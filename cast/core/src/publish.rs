use crate::config::CastConfig;
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PublishError {
    #[error("Cargo build --release failed")]
    BuildFailed,
    #[error("Dioxus bundle failed")]
    DxBundleFailed,
    #[error("Zip creation failed: {0}")]
    ZipFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to determine target triple")]
    TargetTripleFailed,
    #[error("Failed to find built artifact in target/release directory")]
    ArtifactNotFound,
    #[error("Failed to parse Cargo.toml: {0}")]
    CargoTomlParseError(String),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("Git command failed: {0}")]
    GitError(String),
    #[error("Bundle output directory not found")]
    BundleOutputNotFound,
    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
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

/// Get the current git SHA
fn get_git_sha(working_directory: &Path) -> Result<String, PublishError> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("HEAD")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(PublishError::GitError("Failed to get git SHA".to_string()));
    }

    let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(sha)
}

/// Check if git working directory is dirty
fn is_git_dirty(working_directory: &Path) -> Result<bool, PublishError> {
    let output = Command::new("git")
        .arg("status")
        .arg("-s")
        .current_dir(working_directory)
        .output()?;

    if !output.status.success() {
        return Err(PublishError::GitError(
            "Failed to check git status".to_string(),
        ));
    }

    Ok(!output.stdout.is_empty())
}

/// Get version from Cargo.toml
fn get_version_from_cargo_toml(working_directory: &Path) -> Result<String, PublishError> {
    let cargo_toml_path = working_directory.join("Cargo.toml");
    let contents = fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value =
        toml::from_str(&contents).map_err(|e| PublishError::CargoTomlParseError(e.to_string()))?;

    let version = cargo_toml
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            PublishError::CargoTomlParseError("version not found in Cargo.toml".to_string())
        })?;

    Ok(version.to_string())
}

/// Get and increment build counter for the current day
/// Returns the build counter to use for this build
fn get_and_increment_build_counter(working_directory: &Path) -> Result<u32, PublishError> {
    // Create .cast directory if it doesn't exist
    let cast_dir = working_directory.join(".cast");
    fs::create_dir_all(&cast_dir)?;

    // Get current date for the counter file name
    let now = chrono::Utc::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let counter_file = cast_dir.join(format!("build_counter_{}.txt", date_str));

    // Read current counter or start at 0
    // If the file is corrupted or contains invalid data, we default to 0 and restart counting
    // This is safe because build counters are per-day and not critical data
    let current_counter = if counter_file.exists() {
        fs::read_to_string(&counter_file)?
            .trim()
            .parse::<u32>()
            .unwrap_or(0)
    } else {
        0
    };

    // Increment counter
    let new_counter = current_counter + 1;

    // Write new counter back to file
    fs::write(&counter_file, new_counter.to_string())?;

    Ok(new_counter)
}

/// Generate versioned filename for bundle
fn generate_bundle_filename(working_directory: &Path) -> Result<String, PublishError> {
    let version = get_version_from_cargo_toml(working_directory)?;
    let sha = get_git_sha(working_directory)?;
    let dirty = if is_git_dirty(working_directory)? {
        "-dirty"
    } else {
        ""
    };

    // Get current date
    let now = chrono::Utc::now();
    let year = now.format("%Y");
    let month = now.format("%m");
    let day = now.format("%d");

    // Get and increment build counter
    let counter = get_and_increment_build_counter(working_directory)?;

    // Truncate SHA to 7 characters for better filename readability
    // If SHA is shorter than 7 characters (shouldn't happen with git), use what we have
    let sha_short = if sha.len() >= 7 {
        &sha[..7]
    } else if !sha.is_empty() {
        &sha
    } else {
        return Err(PublishError::GitError(
            "Git SHA is empty or invalid".to_string(),
        ));
    };

    Ok(format!(
        "{}+{}-{}-{}.{}.{}{}.zip",
        version, year, month, day, counter, sha_short, dirty
    ))
}

/// Create a zip file from a directory
fn create_zip_from_directory(source_dir: &Path, output_path: &Path) -> Result<(), PublishError> {
    use walkdir::WalkDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let file = fs::File::create(output_path)?;
    let mut zip = ZipWriter::new(file);

    // Walk through all files in the source directory
    let walkdir = WalkDir::new(source_dir);
    let it = walkdir.into_iter();

    for entry in it.filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path
            .strip_prefix(source_dir)
            .map_err(|_| {
                PublishError::IoError(std::io::Error::other("Failed to strip prefix from path"))
            })?
            .to_string_lossy();

        // Skip .DS_Store files (case-insensitive check)
        if name
            .split('/')
            .any(|component| component.eq_ignore_ascii_case(".DS_Store"))
        {
            continue;
        }

        if path.is_file() {
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);

            zip.start_file(name.to_string(), options)?;
            let mut f = fs::File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        } else if !name.is_empty() {
            // Add directory entry
            let options = SimpleFileOptions::default().unix_permissions(0o755);
            zip.add_directory(name.to_string(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}

/// Publish a Dioxus web project
fn publish_dioxus(working_directory: &Path) -> Result<(), PublishError> {
    // Run dx bundle --platform web --release
    let status = Command::new("dx")
        .arg("bundle")
        .arg("--platform")
        .arg("web")
        .arg("--release")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(PublishError::DxBundleFailed);
    }

    // The bundle output is in target/dx/web/release/web/public
    let bundle_dir = working_directory
        .join("target")
        .join("dx")
        .join("web")
        .join("release")
        .join("web")
        .join("public");

    if !bundle_dir.exists() {
        return Err(PublishError::BundleOutputNotFound);
    }

    // Generate versioned filename
    let filename = generate_bundle_filename(working_directory)?;

    // Create artifacts directory
    let artifacts_dir = working_directory.join("artifacts");
    fs::create_dir_all(&artifacts_dir)?;

    // Create zip file using Rust zip crate
    let zip_path = artifacts_dir.join(&filename);
    create_zip_from_directory(&bundle_dir, &zip_path)?;

    Ok(())
}

/// Publish a Rust binary project
fn publish_binary(working_directory: &Path) -> Result<(), PublishError> {
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

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

    // Generate versioned filename
    let filename = generate_bundle_filename(working_directory)?;

    // Create artifacts directory
    let artifacts_dir = working_directory.join("artifacts");
    fs::create_dir_all(&artifacts_dir)?;

    // Create zip file with target triple in the structure
    let zip_path = artifacts_dir.join(&filename);
    let file = fs::File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    // Add target triple directory in the zip
    let target_dir_in_zip = format!("{}/", target_triple);
    let options = SimpleFileOptions::default().unix_permissions(0o755);
    zip.add_directory(&target_dir_in_zip, options)?;

    // Add the binary to the zip under the target triple directory
    let file_in_zip = format!("{}/{}", target_triple, artifact_name);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    zip.start_file(file_in_zip, options)?;

    // Copy binary content to zip
    let mut binary_file = fs::File::open(&artifact_path)?;
    std::io::copy(&mut binary_file, &mut zip)?;

    zip.finish()?;

    Ok(())
}

/// Check if a Cargo.toml is a workspace root (has [workspace] section)
fn is_workspace_root(working_directory: &Path) -> Result<bool, PublishError> {
    let cargo_toml_path = working_directory.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Ok(false);
    }

    let contents = fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value =
        toml::from_str(&contents).map_err(|e| PublishError::CargoTomlParseError(e.to_string()))?;

    // Check if there's a [workspace] section
    Ok(cargo_toml.get("workspace").is_some())
}

/// Run release build and copy artifacts to the artifacts directory
/// Supports both Rust binaries and Dioxus web projects
/// Skips workspace roots (directories with [workspace] in Cargo.toml)
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), PublishError> {
    let working_directory = working_directory.as_ref();

    // Skip workspace roots - they don't have publishable artifacts
    if is_workspace_root(working_directory)? {
        return Ok(());
    }

    // Load Cast configuration to determine project type
    let config = CastConfig::load_from_dir(working_directory)?;

    // Check if this is a Dioxus project
    if let Some(framework) = &config.framework {
        if framework == "dioxus" {
            return publish_dioxus(working_directory);
        }
    }

    // Default to binary publish for non-Dioxus projects
    publish_binary(working_directory)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

        // Initialize git repository (required for generate_bundle_filename)
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

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

        // Commit to have a valid git SHA
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Run publish
        let result = run(tmp_dir.path());
        assert!(result.is_ok(), "Publish failed: {:?}", result.err());

        // Check that artifacts directory was created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(artifacts_dir.exists());

        // Check that a zip file was created (not the raw binary)
        let entries = fs::read_dir(&artifacts_dir).unwrap();
        let zip_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "zip")
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(
            zip_files.len(),
            1,
            "Expected exactly one zip file in artifacts directory"
        );

        // Verify the zip file contains the binary in the target triple directory
        let zip_path = zip_files[0].path();
        let file = fs::File::open(&zip_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        // Check that the zip contains files
        assert!(archive.len() > 0, "Zip file should not be empty");

        // Find the binary in the zip
        let target_triple = get_target_triple().unwrap();
        let artifact_name = if cfg!(windows) {
            "test_binary.exe"
        } else {
            "test_binary"
        };
        let expected_path = format!("{}/{}", target_triple, artifact_name);

        let found = (0..archive.len()).any(|i| {
            let file = archive.by_index(i).unwrap();
            file.name() == expected_path
        });

        assert!(found, "Expected to find {} in zip file", expected_path);
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

    #[test]
    fn test_get_version_from_cargo_toml() {
        let tmp_dir = TempDir::new("test_version").unwrap();
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "1.2.3"
edition = "2021"
"#,
        )
        .unwrap();

        let version = get_version_from_cargo_toml(tmp_dir.path()).unwrap();
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn test_get_git_sha() {
        // This test requires a git repository
        let tmp_dir = TempDir::new("test_git_sha").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Configure git user for the test
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a file and commit
        fs::write(tmp_dir.path().join("test.txt"), "test").unwrap();
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let sha = get_git_sha(tmp_dir.path()).unwrap();
        assert!(!sha.is_empty());
        assert_eq!(sha.len(), 40); // Git SHA is 40 characters
    }

    #[test]
    fn test_is_git_dirty() {
        let tmp_dir = TempDir::new("test_git_dirty").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Configure git user
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Should be clean initially (no files)
        let dirty = is_git_dirty(tmp_dir.path()).unwrap();
        assert!(!dirty);

        // Add a file - should be dirty
        fs::write(tmp_dir.path().join("test.txt"), "test").unwrap();
        let dirty = is_git_dirty(tmp_dir.path()).unwrap();
        assert!(dirty);

        // Commit - should be clean again
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        let dirty = is_git_dirty(tmp_dir.path()).unwrap();
        assert!(!dirty);
    }

    #[test]
    fn test_generate_bundle_filename() {
        let tmp_dir = TempDir::new("test_bundle_filename").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Configure git user
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        // Commit to have a SHA
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let filename = generate_bundle_filename(tmp_dir.path()).unwrap();

        // Should start with version
        assert!(filename.starts_with("0.1.0+"));
        // Should end with .zip
        assert!(filename.ends_with(".zip"));
        // Should contain date components (year-month-day)
        assert!(filename.contains('-'));
    }

    #[test]
    fn test_get_and_increment_build_counter() {
        let tmp_dir = TempDir::new("test_build_counter").unwrap();

        // First call should return 1
        let counter1 = get_and_increment_build_counter(tmp_dir.path()).unwrap();
        assert_eq!(counter1, 1);

        // Second call should return 2
        let counter2 = get_and_increment_build_counter(tmp_dir.path()).unwrap();
        assert_eq!(counter2, 2);

        // Third call should return 3
        let counter3 = get_and_increment_build_counter(tmp_dir.path()).unwrap();
        assert_eq!(counter3, 3);

        // Verify the counter file exists in .cast directory
        let now = chrono::Utc::now();
        let date_str = now.format("%Y-%m-%d").to_string();
        let counter_file = tmp_dir
            .path()
            .join(".cast")
            .join(format!("build_counter_{}.txt", date_str));
        assert!(counter_file.exists());

        // Verify file contains the correct value
        let contents = fs::read_to_string(&counter_file).unwrap();
        assert_eq!(contents, "3");
    }

    #[test]
    fn test_generate_bundle_filename_increments_counter() {
        let tmp_dir = TempDir::new("test_bundle_counter").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Configure git user
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        // Commit to have a SHA
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("test")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Generate first filename - should contain ".1."
        let filename1 = generate_bundle_filename(tmp_dir.path()).unwrap();
        assert!(filename1.contains(".1."));

        // Generate second filename - should contain ".2."
        let filename2 = generate_bundle_filename(tmp_dir.path()).unwrap();
        assert!(filename2.contains(".2."));

        // Ensure they're different
        assert_ne!(filename1, filename2);
    }

    #[test]
    fn test_is_workspace_root_detects_workspace() {
        let tmp_dir = TempDir::new("test_workspace").unwrap();

        // Create a workspace Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["member1", "member2"]
"#,
        )
        .unwrap();

        let is_workspace = is_workspace_root(tmp_dir.path()).unwrap();
        assert!(is_workspace, "Should detect workspace root");
    }

    #[test]
    fn test_is_workspace_root_detects_non_workspace() {
        let tmp_dir = TempDir::new("test_non_workspace").unwrap();

        // Create a regular package Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();

        let is_workspace = is_workspace_root(tmp_dir.path()).unwrap();
        assert!(
            !is_workspace,
            "Should not detect non-workspace as workspace"
        );
    }

    #[test]
    fn test_publish_skips_workspace_root() {
        let tmp_dir = TempDir::new("test_workspace_publish").unwrap();

        // Create a workspace Cargo.toml
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            r#"[workspace]
members = ["member1"]

[workspace.package]
version = "0.1.0"
"#,
        )
        .unwrap();

        // Publish should succeed but do nothing for workspace roots
        let result = run(tmp_dir.path());
        assert!(result.is_ok(), "Publish should succeed for workspace roots");

        // Verify no artifacts directory was created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "No artifacts should be created for workspace roots"
        );
    }
}
