use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Cargo build --release failed")]
    BuildFailed,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to determine target triple")]
    TargetTripleFailed,
    #[error("Failed to find built artifact in target/release directory")]
    ArtifactNotFound,
}

/// Get the target triple for the current platform
fn get_target_triple() -> Result<String, PublishError> {
    let output = Command::new("rustc")
        .arg("--version")
        .arg("--verbose")
        .output()?;

    if !output.status.success() {
        return Err(PublishError::TargetTripleFailed);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse the output to find the "host:" line
    for line in stdout.lines() {
        if line.starts_with("host:") {
            if let Some(triple) = line.split_whitespace().nth(1) {
                return Ok(triple.to_string());
            }
        }
    }

    Err(PublishError::TargetTripleFailed)
}

/// Find the executable binary in the target/release directory
fn find_binary_artifact(working_directory: &Path) -> Result<String, PublishError> {
    let release_dir = working_directory.join("target").join("release");

    if !release_dir.exists() {
        return Err(PublishError::ArtifactNotFound);
    }

    // Read the directory and find executable files
    let entries = fs::read_dir(&release_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Skip directories and non-files
        if !path.is_file() {
            continue;
        }

        // Get the file name
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip files with extensions (like .d, .rlib, etc.) except .exe on Windows
        if file_name.contains('.') && !file_name.ends_with(".exe") {
            continue;
        }

        // Check if the file is executable (on Unix) or is an .exe (on Windows)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = match path.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let permissions = metadata.permissions();
            // Check if file has execute permission
            if permissions.mode() & 0o111 != 0 {
                return Ok(file_name);
            }
        }

        #[cfg(windows)]
        {
            if file_name.ends_with(".exe") {
                return Ok(file_name);
            }
        }
    }

    Err(PublishError::ArtifactNotFound)
}

/// Run release build and copy artifacts to the artifacts directory
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), PublishError> {
    let working_directory = working_directory.as_ref();

    // Run cargo build --release
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(PublishError::BuildFailed);
    }

    // Find the built binary artifact
    let artifact_name = find_binary_artifact(working_directory)?;

    // Get target triple
    let target_triple = get_target_triple()?;

    // Get the full path to the artifact
    let release_dir = working_directory.join("target").join("release");
    let artifact_path = release_dir.join(&artifact_name);

    // Create artifacts directory structure: artifacts/<target-triple>/
    let artifacts_dir = working_directory.join("artifacts").join(&target_triple);
    fs::create_dir_all(&artifacts_dir)?;

    // Copy the artifact to the artifacts directory
    let destination = artifacts_dir.join(&artifact_name);
    fs::copy(&artifact_path, &destination)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_get_target_triple() {
        let triple = get_target_triple().unwrap();
        // Should return something like "x86_64-unknown-linux-gnu"
        assert!(triple.contains('-'));
        assert!(!triple.is_empty());
    }

    #[test]
    fn test_publish_fails_without_cargo_project() {
        let tmp_dir = TempDir::new("test_publish_no_project").unwrap();
        let result = run(tmp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_publish_creates_artifacts_directory() {
        let tmp_dir = TempDir::new("test_publish_artifacts").unwrap();

        // Create a minimal binary project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test_binary"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        // Run publish
        let result = run(tmp_dir.path());
        assert!(result.is_ok(), "Publish failed: {:?}", result.err());

        // Check that artifacts directory was created
        let target_triple = get_target_triple().unwrap();
        let artifacts_dir = tmp_dir.path().join("artifacts").join(target_triple);
        assert!(artifacts_dir.exists());

        // Check that the artifact was copied
        let artifact_name = if cfg!(windows) {
            "test_binary.exe"
        } else {
            "test_binary"
        };
        assert!(artifacts_dir.join(artifact_name).exists());
    }

    #[test]
    fn test_publish_fails_with_invalid_code() {
        let tmp_dir = TempDir::new("test_publish_invalid").unwrap();

        // Create a project with invalid code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test_invalid"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { this_does_not_compile }\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err());
        if let Err(PublishError::BuildFailed) = result {
            // Expected error type
        } else {
            panic!("Expected BuildFailed error");
        }
    }
}
