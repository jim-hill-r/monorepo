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
    #[error("Failed to determine package name from Cargo.toml")]
    PackageNameNotFound,
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

/// Get the package name from Cargo.toml
fn get_package_name(working_directory: &Path) -> Result<String, PublishError> {
    let cargo_toml_path = working_directory.join("Cargo.toml");
    let contents = fs::read_to_string(cargo_toml_path)?;

    // Simple TOML parsing to extract package name
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") && trimmed.contains('=') {
            if let Some(name_part) = trimmed.split('=').nth(1) {
                let name = name_part.trim().trim_matches('"').trim_matches('\'');
                return Ok(name.to_string());
            }
        }
    }

    Err(PublishError::PackageNameNotFound)
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

    // Get the package name to find the artifact
    let package_name = get_package_name(working_directory)?;

    // Get target triple
    let target_triple = get_target_triple()?;

    // Find the built artifact in target/release
    let release_dir = working_directory.join("target").join("release");
    let artifact_path = release_dir.join(&package_name);

    // Check if artifact exists (might need .exe extension on Windows)
    let artifact_path = if artifact_path.exists() {
        artifact_path
    } else {
        let with_exe = release_dir.join(format!("{}.exe", package_name));
        if with_exe.exists() {
            with_exe
        } else {
            return Err(PublishError::ArtifactNotFound);
        }
    };

    // Create artifacts directory structure: artifacts/<target-triple>/
    let artifacts_dir = working_directory.join("artifacts").join(&target_triple);
    fs::create_dir_all(&artifacts_dir)?;

    // Copy the artifact to the artifacts directory
    let destination = artifacts_dir.join(
        artifact_path
            .file_name()
            .ok_or(PublishError::ArtifactNotFound)?,
    );
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
    fn test_get_package_name() {
        let tmp_dir = TempDir::new("test_package_name").unwrap();

        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let name = get_package_name(tmp_dir.path()).unwrap();
        assert_eq!(name, "test_project");
    }

    #[test]
    fn test_get_package_name_with_quotes() {
        let tmp_dir = TempDir::new("test_package_name_quotes").unwrap();

        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test-quotes"
version = "0.1.0"
"#,
        )
        .unwrap();

        let name = get_package_name(tmp_dir.path()).unwrap();
        assert_eq!(name, "test-quotes");
    }

    #[test]
    fn test_get_package_name_fails_without_cargo_toml() {
        let tmp_dir = TempDir::new("test_no_cargo").unwrap();

        let result = get_package_name(tmp_dir.path());
        assert!(result.is_err());
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
