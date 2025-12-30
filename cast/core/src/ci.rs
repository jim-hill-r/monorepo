use crate::build;
use crate::publish;
use crate::test;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CiError {
    #[error("Cargo fmt check failed")]
    FmtError,
    #[error("Cargo clippy check failed")]
    ClippyError,
    #[error("Cargo build failed: {0}")]
    BuildError(#[from] build::BuildError),
    #[error("Cargo test failed: {0}")]
    TestError(#[from] test::TestError),
    #[error("npm install failed")]
    NpmInstallError,
    #[error("npm lint failed")]
    NpmLintError,
    #[error("npm compile failed")]
    NpmCompileError,
    #[error("npm test failed")]
    NpmTestError,
    #[error("Publish failed: {0}")]
    PublishError(#[from] publish::PublishError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Git LFS command failed: {0}")]
    GitLfsError(String),
    #[error("Git commit failed: {0}")]
    GitCommitError(String),
}

/// Run CI checks for a project
/// This detects the project type and runs appropriate checks:
/// - For Rust projects (has Cargo.toml): cargo fmt, clippy, build, test
/// - For TypeScript projects (has package.json): npm lint, compile, test
/// - Projects can have both Cargo.toml and package.json (e.g., Dioxus web apps with Playwright tests)
/// - If all checks pass, runs publish to create release artifacts
pub fn run(working_directory: impl AsRef<Path>) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

    // Check if this is a Rust project or TypeScript project
    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    // Track if we ran any CI checks
    let mut ran_ci_checks = false;

    // Run Rust CI if Cargo.toml exists
    if has_cargo_toml {
        run_rust_ci(working_directory)?;
        ran_ci_checks = true;
    }

    // Run TypeScript CI if package.json exists (can run in addition to Rust CI)
    if has_package_json {
        run_typescript_ci(working_directory)?;
        ran_ci_checks = true;
    }

    // If we ran CI checks and they all passed, run publish to create release artifacts
    if ran_ci_checks {
        publish::run(working_directory)?;

        // Commit artifacts to git with git LFS
        commit_artifacts(working_directory)?;
    }
    // If no CI checks were run, silently succeed (empty project or unsupported type)

    Ok(())
}

/// Run CI checks for a Rust project
/// This runs:
/// 1. cargo fmt --check
/// 2. cargo clippy -- -D warnings
/// 3. cast build (cargo build)
/// 4. cast test (cargo test)
fn run_rust_ci(working_directory: &Path) -> Result<(), CiError> {
    // Run cargo fmt --check
    run_fmt_check(working_directory)?;

    // Run cargo clippy
    run_clippy(working_directory)?;

    // Run cast build
    build::run(working_directory)?;

    // Run cast test
    test::run(working_directory)?;

    Ok(())
}

/// Run CI checks for a TypeScript/Node.js project
/// This runs:
/// 1. npm ci (to install dependencies from lockfile)
/// 2. npm run lint (if script exists)
/// 3. npm run compile (if script exists)
/// 4. npm test (if script exists)
fn run_typescript_ci(working_directory: &Path) -> Result<(), CiError> {
    // Run npm ci to ensure dependencies are installed from lockfile
    run_npm_install(working_directory).map_err(|_| CiError::NpmInstallError)?;

    // Run npm run lint if it exists
    if npm_script_exists(working_directory, "lint") {
        run_npm_command(working_directory, "lint").map_err(|_| CiError::NpmLintError)?;
    }

    // Run npm run compile if it exists
    if npm_script_exists(working_directory, "compile") {
        run_npm_command(working_directory, "compile").map_err(|_| CiError::NpmCompileError)?;
    }

    // Run npm test if it exists (e.g., Playwright tests)
    if npm_script_exists(working_directory, "test") {
        run_npm_command(working_directory, "test").map_err(|_| CiError::NpmTestError)?;
    }

    Ok(())
}

/// Check if an npm script exists in package.json
fn npm_script_exists(working_directory: &Path, script: &str) -> bool {
    let package_json_path = working_directory.join("package.json");
    if let Ok(content) = std::fs::read_to_string(package_json_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(scripts) = json.get("scripts") {
                return scripts.get(script).is_some();
            }
        }
    }
    false
}

/// Run an npm command
fn run_npm_command(working_directory: &Path, command: &str) -> Result<(), std::io::Error> {
    let status = Command::new("npm")
        .arg("run")
        .arg(command)
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other(format!("npm run {} failed", command)));
    }

    Ok(())
}

/// Run npm ci to install dependencies from lockfile
/// Uses 'npm ci' for faster, more reliable installs in CI environments
fn run_npm_install(working_directory: &Path) -> Result<(), std::io::Error> {
    let status = Command::new("npm")
        .arg("ci")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("npm ci failed"));
    }

    Ok(())
}

fn run_fmt_check(working_directory: &Path) -> Result<(), CiError> {
    let status = Command::new("cargo")
        .arg("fmt")
        .arg("--check")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::FmtError);
    }

    Ok(())
}

fn run_clippy(working_directory: &Path) -> Result<(), CiError> {
    let status = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::ClippyError);
    }

    Ok(())
}

/// Commit artifacts to git with git LFS
/// This function:
/// 1. Checks if git LFS is installed
/// 2. Stages the artifacts directory (git LFS tracking should be configured via .gitattributes)
/// 3. Commits the artifacts with a descriptive message
///
/// Note: This assumes the repository's .gitattributes is already configured to track
/// artifact files (e.g., *.zip) with git LFS. It does not modify .gitattributes.
///
/// If there are no new artifacts or artifacts haven't changed, this is a no-op.
/// If we're not in a git repository, this silently succeeds.
fn commit_artifacts(working_directory: &Path) -> Result<(), CiError> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(working_directory)
        .output()?;

    if !git_check.status.success() {
        // Not in a git repository, silently succeed
        return Ok(());
    }

    // Check if git LFS is installed
    let lfs_check = Command::new("git")
        .arg("lfs")
        .arg("version")
        .current_dir(working_directory)
        .output()?;

    if !lfs_check.status.success() {
        return Err(CiError::GitLfsError(
            "git-lfs is not installed or not available in PATH".to_string(),
        ));
    }

    // Check if artifacts directory exists
    let artifacts_dir = working_directory.join("artifacts");
    if !artifacts_dir.exists() {
        // No artifacts to commit
        return Ok(());
    }

    // Stage the artifacts directory
    let status = Command::new("git")
        .arg("add")
        .arg("artifacts")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::GitCommitError(
            "Failed to stage artifacts directory".to_string(),
        ));
    }

    // Check if there are any changes to commit
    let diff_check = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--quiet")
        .arg("artifacts")
        .current_dir(working_directory)
        .status()?;

    // diff --quiet returns 0 if no changes, 1 if there are changes
    if diff_check.success() {
        // No changes to commit
        return Ok(());
    }

    // Commit the artifacts
    let status = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("chore: update CI artifacts")
        .current_dir(working_directory)
        .status()?;

    if !status.success() {
        return Err(CiError::GitCommitError(
            "Failed to commit artifacts".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use tempdir::TempDir;

    #[test]
    fn test_run_ci_succeeds_without_cargo_or_package_json() {
        let tmp_dir = TempDir::new("test_ci").unwrap();

        let result = run(tmp_dir.path());
        // Should succeed silently for directories without Cargo.toml or package.json
        // Publish should not run since no CI checks were performed
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_fmt_check_passes_with_formatted_code() {
        let tmp_dir = TempDir::new("test_fmt").unwrap();

        // Create a simple Cargo project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = run_fmt_check(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_ci_runs_publish_after_successful_checks() {
        let tmp_dir = TempDir::new("test_ci_publish").unwrap();

        // Initialize git repo (required by publish)
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

        // Create a minimal binary project with properly formatted code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .unwrap();

        // Commit the project
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let result = run(tmp_dir.path());
        assert!(
            result.is_ok(),
            "CI should succeed and run publish: {:?}",
            result.err()
        );

        // Verify that artifacts directory was created by publish
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            artifacts_dir.exists(),
            "Artifacts directory should exist after CI runs publish"
        );
    }

    #[test]
    fn test_ci_does_not_run_publish_when_build_fails() {
        let tmp_dir = TempDir::new("test_ci_no_publish").unwrap();

        // Initialize git repo (required by publish)
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

        // Create a project with invalid code (will fail build)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        // Write properly formatted but semantically invalid code
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    undefined_function();\n}\n",
        )
        .unwrap();

        let result = run(tmp_dir.path());
        assert!(result.is_err(), "CI should fail when build fails");

        // Verify that artifacts directory was NOT created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should not exist when CI fails"
        );
    }

    #[test]
    fn test_commit_artifacts_succeeds_without_git_repo() {
        let tmp_dir = TempDir::new("test_commit_no_git").unwrap();

        // Create artifacts directory without git repo
        fs::create_dir_all(tmp_dir.path().join("artifacts")).unwrap();
        fs::write(tmp_dir.path().join("artifacts").join("test.zip"), b"test").unwrap();

        // Should succeed silently when not in a git repo
        let result = commit_artifacts(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_commit_artifacts_succeeds_without_artifacts_dir() {
        let tmp_dir = TempDir::new("test_commit_no_artifacts").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Should succeed when there's no artifacts directory
        let result = commit_artifacts(tmp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_commit_artifacts_commits_new_artifacts() {
        let tmp_dir = TempDir::new("test_commit_artifacts").unwrap();

        // Initialize git repo
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

        // Create an initial commit (required before we can check status)
        fs::write(tmp_dir.path().join("README.md"), "test").unwrap();
        Command::new("git")
            .arg("add")
            .arg("README.md")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create artifacts directory with a file
        fs::create_dir_all(tmp_dir.path().join("artifacts")).unwrap();
        fs::write(
            tmp_dir.path().join("artifacts").join("test.zip"),
            b"test content",
        )
        .unwrap();

        // Commit artifacts
        let result = commit_artifacts(tmp_dir.path());
        assert!(
            result.is_ok(),
            "Failed to commit artifacts: {:?}",
            result.err()
        );

        // Verify the commit was created
        let log_output = Command::new("git")
            .arg("log")
            .arg("--oneline")
            .arg("-1")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let log = String::from_utf8_lossy(&log_output.stdout);
        assert!(log.contains("chore: update CI artifacts"));

        // Verify the artifacts are in the commit
        let ls_output = Command::new("git")
            .arg("ls-tree")
            .arg("-r")
            .arg("HEAD")
            .arg("--name-only")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let files = String::from_utf8_lossy(&ls_output.stdout);
        assert!(files.contains("artifacts/test.zip"));
    }

    #[test]
    fn test_commit_artifacts_no_op_when_no_changes() {
        let tmp_dir = TempDir::new("test_commit_no_changes").unwrap();

        // Initialize git repo
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

        // Create artifacts and commit them
        fs::create_dir_all(tmp_dir.path().join("artifacts")).unwrap();
        fs::write(tmp_dir.path().join("artifacts").join("test.zip"), b"test").unwrap();
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Get commit count before
        let log_before = Command::new("git")
            .arg("rev-list")
            .arg("--count")
            .arg("HEAD")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        let count_before = String::from_utf8_lossy(&log_before.stdout)
            .trim()
            .parse::<u32>()
            .unwrap();

        // Try to commit again - should be no-op
        let result = commit_artifacts(tmp_dir.path());
        assert!(result.is_ok());

        // Verify no new commit was created
        let log_after = Command::new("git")
            .arg("rev-list")
            .arg("--count")
            .arg("HEAD")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        let count_after = String::from_utf8_lossy(&log_after.stdout)
            .trim()
            .parse::<u32>()
            .unwrap();

        assert_eq!(count_before, count_after, "No new commit should be created");
    }

    #[test]
    fn test_commit_artifacts_requires_git_lfs() {
        let tmp_dir = TempDir::new("test_commit_lfs").unwrap();

        // Initialize git repo without git-lfs
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

        // Create artifacts directory
        fs::create_dir_all(tmp_dir.path().join("artifacts")).unwrap();
        fs::write(tmp_dir.path().join("artifacts").join("test.zip"), b"test").unwrap();

        // Try to commit - should check for git-lfs
        let result = commit_artifacts(tmp_dir.path());

        // If git-lfs is not installed in the test environment, we expect an error
        // If it is installed, we expect success
        // We can't reliably test this without knowing the environment, so we just verify
        // the function handles both cases properly
        match result {
            Ok(_) => {
                // git-lfs is available, verify commit was created
                let log_output = Command::new("git")
                    .arg("log")
                    .arg("--oneline")
                    .current_dir(tmp_dir.path())
                    .output()
                    .unwrap();
                assert!(!log_output.stdout.is_empty());
            }
            Err(CiError::GitLfsError(_)) => {
                // git-lfs is not available, which is expected in some environments
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
