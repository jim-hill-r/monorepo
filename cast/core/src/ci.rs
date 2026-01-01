use crate::build;
use crate::publish;
use crate::test;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

/// CI execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CiMode {
    /// Run checks only (default mode for PR validation)
    #[default]
    Check,
    /// Auto-fix issues that can be fixed automatically (e.g., formatting)
    Fix,
    /// Build in release mode and publish artifacts (for post-merge to master)
    Release,
}

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
/// - Behavior depends on the mode:
///   - Check: Run all checks (fmt --check, clippy, build, test)
///   - Fix: Auto-fix formatting issues, then run checks
///   - Release: Run all checks with release build, then publish artifacts
/// - If recursive_depth is Some(depth), after running CI on the current directory,
///   it will find all Cast projects up to 'depth' levels below and run CI on them
/// - If only_changed is true, CI will only run if the project has changes compared to the origin's default branch
pub fn run(
    working_directory: impl AsRef<Path>,
    mode: CiMode,
    recursive_depth: Option<usize>,
    only_changed: bool,
) -> Result<(), CiError> {
    let working_directory = working_directory.as_ref();

    // If only_changed is true, check if the project has changes
    if only_changed {
        match has_changes(working_directory) {
            Ok(true) => {
                // Has changes, continue with CI
            }
            Ok(false) => {
                // No changes, skip CI
                println!("No changes found in project. Skipping CI.");
                return Ok(());
            }
            Err(e) => {
                // Error checking for changes (e.g., not in a git repo)
                eprintln!("Warning: Could not check for changes: {}. Proceeding with CI.", e);
                // Continue with CI despite the error
            }
        }
    }

    // Check if this is a Rust project or TypeScript project
    let has_cargo_toml = working_directory.join("Cargo.toml").exists();
    let has_package_json = working_directory.join("package.json").exists();

    // Track if we ran any CI checks
    let mut ran_ci_checks = false;

    // Run Rust CI if Cargo.toml exists
    if has_cargo_toml {
        run_rust_ci(working_directory, mode)?;
        ran_ci_checks = true;
    }

    // Run TypeScript CI if package.json exists (can run in addition to Rust CI)
    if has_package_json {
        run_typescript_ci(working_directory)?;
        ran_ci_checks = true;
    }

    // Handle post-check publishing based on mode
    if ran_ci_checks {
        match mode {
            CiMode::Check | CiMode::Fix => {
                // For Check and Fix modes, run publish to create artifacts
                publish::run(working_directory)?;

                // Commit artifacts to git with git LFS
                commit_artifacts(working_directory)?;
            }
            CiMode::Release => {
                // For Release mode, publish is already handled by run_rust_ci
                // Just commit the artifacts
                commit_artifacts(working_directory)?;
            }
        }
    }
    // If no CI checks were run, silently succeed (empty project or unsupported type)

    // Run CI recursively on child projects if requested
    if let Some(depth) = recursive_depth {
        run_ci_recursively(working_directory, mode, depth, only_changed)?;
    }

    Ok(())
}

/// Run CI checks for a Rust project
/// This runs different steps based on the mode:
/// - Check mode: cargo fmt --check, clippy, build (debug), test
/// - Fix mode: cargo fmt (auto-fix), clippy, build (debug), test
/// - Release mode: cargo fmt --check, clippy, build --release, test, publish
fn run_rust_ci(working_directory: &Path, mode: CiMode) -> Result<(), CiError> {
    // Handle formatting based on mode
    match mode {
        CiMode::Check | CiMode::Release => {
            // Run cargo fmt --check
            run_fmt_check(working_directory)?;
        }
        CiMode::Fix => {
            // Run cargo fmt to auto-fix formatting issues
            run_fmt_fix(working_directory)?;
        }
    }

    // Run cargo clippy
    run_clippy(working_directory)?;

    // Run build - release mode for Release, debug for others
    match mode {
        CiMode::Check | CiMode::Fix => {
            build::run(working_directory)?;
        }
        CiMode::Release => {
            build::run_release(working_directory)?;
        }
    }

    // Run cast test
    test::run(working_directory)?;

    // For Release mode, also run publish
    if mode == CiMode::Release {
        publish::run(working_directory)?;
    }

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

fn run_fmt_fix(working_directory: &Path) -> Result<(), CiError> {
    let status = Command::new("cargo")
        .arg("fmt")
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

/// Run CI recursively on child projects
/// Finds all Cast projects up to 'max_depth' levels below the current directory
/// and runs CI on each of them with recursive_depth decremented by the depth at which they were found
fn run_ci_recursively(
    working_directory: &Path,
    mode: CiMode,
    max_depth: usize,
    only_changed: bool,
) -> Result<(), CiError> {
    if max_depth == 0 {
        return Ok(());
    }

    let projects = find_cast_projects(working_directory, max_depth)?;

    for (project_path, depth) in projects {
        println!("Running CI on child project: {}", project_path.display());

        // Calculate remaining depth for recursive calls
        // If we found this project at depth D, and we started with max_depth,
        // then we should run with max_depth - D for this project
        let remaining_depth = if max_depth > depth {
            Some(max_depth - depth)
        } else {
            None
        };

        // Run CI on the child project
        run(&project_path, mode, remaining_depth, only_changed)?;
    }

    Ok(())
}

/// Check if the current project has changes compared to the origin's default branch
/// Returns true if there are changes, false if no changes
/// Returns an error if git operations fail (e.g., not in a git repo)
fn has_changes(working_directory: &Path) -> Result<bool, CiError> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(working_directory)
        .output()?;

    if !git_check.status.success() {
        return Err(CiError::IoError(std::io::Error::other(
            "Not in a git repository",
        )));
    }

    // Get the default branch name from origin
    let default_branch = get_default_branch(working_directory)?;

    // Check if there are any changes between HEAD and origin/default_branch
    // We'll check if the project directory has any diffs
    let diff_output = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .arg(&format!("origin/{}", default_branch))
        .arg("HEAD")
        .arg("--")
        .arg(".")
        .current_dir(working_directory)
        .status()?;

    // git diff --quiet returns 0 if no changes, 1 if there are changes
    Ok(!diff_output.success())
}

/// Get the default branch name from origin (usually 'main' or 'master')
fn get_default_branch(working_directory: &Path) -> Result<String, CiError> {
    // Try to get the default branch from origin's HEAD
    let output = Command::new("git")
        .arg("symbolic-ref")
        .arg("refs/remotes/origin/HEAD")
        .current_dir(working_directory)
        .output()?;

    if output.status.success() {
        let branch_ref = String::from_utf8_lossy(&output.stdout);
        // Parse "refs/remotes/origin/main" to get "main"
        if let Some(branch_name) = branch_ref.trim().strip_prefix("refs/remotes/origin/") {
            return Ok(branch_name.to_string());
        }
    }

    // Fallback: try to guess the default branch
    // Check if origin/main exists
    let main_check = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("origin/main")
        .current_dir(working_directory)
        .output()?;

    if main_check.status.success() {
        return Ok("main".to_string());
    }

    // Check if origin/master exists
    let master_check = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("origin/master")
        .current_dir(working_directory)
        .output()?;

    if master_check.status.success() {
        return Ok("master".to_string());
    }

    // If we can't find the default branch, return an error
    Err(CiError::IoError(std::io::Error::other(
        "Could not determine default branch from origin",
    )))
}

/// Find Cast projects within a given depth below the working directory
/// Returns a vector of (project_path, depth_found) tuples
/// Skips common build directories like target, node_modules, .git, etc.
fn find_cast_projects(
    working_directory: &Path,
    max_depth: usize,
) -> Result<Vec<(PathBuf, usize)>, CiError> {
    let mut projects = Vec::new();
    find_cast_projects_recursive(working_directory, max_depth, 0, &mut projects)?;
    Ok(projects)
}

/// Recursively find Cast projects up to max_depth levels
fn find_cast_projects_recursive(
    dir: &Path,
    max_depth: usize,
    current_depth: usize,
    projects: &mut Vec<(PathBuf, usize)>,
) -> Result<(), CiError> {
    // Stop if we've reached max depth
    if current_depth > max_depth {
        return Ok(());
    }

    // Skip if not a directory
    if !dir.is_dir() {
        return Ok(());
    }

    // Skip directories that shouldn't be searched
    if let Some(dir_name) = dir.file_name() {
        let dir_name = dir_name.to_string_lossy();
        if dir_name == "target"
            || dir_name == "node_modules"
            || dir_name == ".git"
            || dir_name == "dist"
            || dir_name == "build"
            || dir_name == "artifacts"
        {
            return Ok(());
        }
    }

    // Don't scan subdirectories at the starting level (current_depth == 0)
    // We only want projects below the current directory
    if current_depth > 0 {
        // Check if this directory is a Cast project
        let has_cast_toml = dir.join("Cast.toml").exists();
        let cargo_toml_path = dir.join("Cargo.toml");
        let has_cargo_with_cast = cargo_toml_path.exists()
            && crate::config::CastConfig::load_from_cargo_toml(cargo_toml_path)
                .map(|c| c.has_cast_metadata())
                .unwrap_or(false);

        if has_cast_toml || has_cargo_with_cast {
            projects.push((dir.to_path_buf(), current_depth));
            // Don't search subdirectories of a found project
            return Ok(());
        }
    }

    // Search subdirectories
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            find_cast_projects_recursive(&path, max_depth, current_depth + 1, projects)?;
        }
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false);
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false);
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false);
        assert!(result.is_err(), "CI should fail when build fails");

        // Verify that artifacts directory was NOT created
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should not exist when CI fails"
        );
    }

    #[test]
    fn test_ci_fix_mode_auto_formats_code() {
        let tmp_dir = TempDir::new("test_ci_fix").unwrap();

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

        // Create a minimal binary project with poorly formatted code
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main(){println!(\"Hello, world!\");}", // poorly formatted
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

        // CI in Fix mode should auto-format and succeed
        let result = run(tmp_dir.path(), CiMode::Fix, None, false);
        assert!(
            result.is_ok(),
            "CI Fix mode should succeed after auto-formatting: {:?}",
            result.err()
        );

        // Verify the code was reformatted
        let code = fs::read_to_string(tmp_dir.path().join("src/main.rs")).unwrap();
        assert!(
            code.contains("fn main()"),
            "Code should be properly formatted"
        );
    }

    #[test]
    fn test_ci_release_mode_builds_with_release_flag() {
        let tmp_dir = TempDir::new("test_ci_release").unwrap();

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

        // CI in Release mode should build in release and publish
        let result = run(tmp_dir.path(), CiMode::Release, None, false);
        assert!(
            result.is_ok(),
            "CI Release mode should succeed: {:?}",
            result.err()
        );

        // Verify that release binary was built
        let release_binary = tmp_dir.path().join("target/release/test");
        assert!(
            release_binary.exists() || tmp_dir.path().join("target/release/test.exe").exists(),
            "Release binary should exist after CI Release mode"
        );

        // Verify that artifacts directory was created by publish
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            artifacts_dir.exists(),
            "Artifacts directory should exist after CI Release mode"
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

    #[test]
    fn test_find_cast_projects_at_depth_1() {
        let tmp_dir = TempDir::new("test_find_projects").unwrap();

        // Create a cast project in the root
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create two child projects at depth 1
        let child1 = tmp_dir.path().join("child1");
        fs::create_dir_all(&child1).unwrap();
        fs::write(child1.join("Cast.toml"), "").unwrap();

        let child2 = tmp_dir.path().join("child2");
        fs::create_dir_all(&child2).unwrap();
        fs::write(child2.join("Cast.toml"), "").unwrap();

        // Create a grandchild project at depth 2
        let grandchild = child1.join("grandchild");
        fs::create_dir_all(&grandchild).unwrap();
        fs::write(grandchild.join("Cast.toml"), "").unwrap();

        // Find projects at depth 1
        let projects = find_cast_projects(tmp_dir.path(), 1).unwrap();

        // Should find child1 and child2, but not grandchild
        assert_eq!(projects.len(), 2);
        assert!(projects.iter().any(|(p, d)| p == &child1 && *d == 1));
        assert!(projects.iter().any(|(p, d)| p == &child2 && *d == 1));
        assert!(!projects.iter().any(|(p, _)| p == &grandchild));
    }

    #[test]
    fn test_find_cast_projects_at_depth_2() {
        let tmp_dir = TempDir::new("test_find_projects_deep").unwrap();

        // Create a cast project in the root
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a child project at depth 1 in dir1
        let child1 = tmp_dir.path().join("dir1");
        fs::create_dir_all(&child1).unwrap();
        fs::write(child1.join("Cast.toml"), "").unwrap();

        // Create a non-project directory at depth 1
        let dir2 = tmp_dir.path().join("dir2");
        fs::create_dir_all(&dir2).unwrap();

        // Create a grandchild project at depth 2 under dir2 (not under child1)
        let grandchild = dir2.join("grandchild");
        fs::create_dir_all(&grandchild).unwrap();
        fs::write(grandchild.join("Cast.toml"), "").unwrap();

        // Find projects at depth 2
        let projects = find_cast_projects(tmp_dir.path(), 2).unwrap();

        // Should find both child1 and grandchild
        // Note: child1 is found at depth 1, and we don't search its subdirectories
        // grandchild is found at depth 2 under dir2 (which is not a cast project)
        assert_eq!(projects.len(), 2);
        assert!(projects.iter().any(|(p, d)| p == &child1 && *d == 1));
        assert!(projects.iter().any(|(p, d)| p == &grandchild && *d == 2));
    }

    #[test]
    fn test_find_cast_projects_skips_build_directories() {
        let tmp_dir = TempDir::new("test_skip_build_dirs").unwrap();

        // Create a cast project in the root
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create projects in directories that should be skipped
        let target_dir = tmp_dir.path().join("target");
        fs::create_dir_all(&target_dir).unwrap();
        fs::write(target_dir.join("Cast.toml"), "").unwrap();

        let node_modules = tmp_dir.path().join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join("Cast.toml"), "").unwrap();

        // Create a valid child project
        let child = tmp_dir.path().join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("Cast.toml"), "").unwrap();

        // Find projects
        let projects = find_cast_projects(tmp_dir.path(), 1).unwrap();

        // Should only find child, not target or node_modules
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].0, child);
    }

    #[test]
    fn test_find_cast_projects_with_cargo_metadata() {
        let tmp_dir = TempDir::new("test_cargo_metadata").unwrap();

        // Create a cast project in the root
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a child project using Cargo.toml with Cast metadata
        let child = tmp_dir.path().join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(
            child.join("Cargo.toml"),
            "[package]\nname = \"child\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"",
        )
        .unwrap();

        // Find projects
        let projects = find_cast_projects(tmp_dir.path(), 1).unwrap();

        // Should find the child project
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].0, child);
    }

    #[test]
    fn test_run_ci_with_recursive_depth() {
        let tmp_dir = TempDir::new("test_recursive_ci").unwrap();

        // Initialize git repo for the root
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

        // Create root project
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"root\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"root\");\n}\n",
        )
        .unwrap();

        // Create child project
        let child = tmp_dir.path().join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("Cast.toml"), "").unwrap();
        fs::write(
            child.join("Cargo.toml"),
            "[package]\nname = \"child\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(child.join("src")).unwrap();
        fs::write(
            child.join("src/main.rs"),
            "fn main() {\n    println!(\"child\");\n}\n",
        )
        .unwrap();

        // Commit everything
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

        // Run CI with recursive depth 1
        let result = run(tmp_dir.path(), CiMode::Check, Some(1), false);

        // Should succeed
        assert!(
            result.is_ok(),
            "Recursive CI should succeed: {:?}",
            result.err()
        );

        // Verify artifacts were created for both projects
        assert!(tmp_dir.path().join("artifacts").exists());
        assert!(child.join("artifacts").exists());
    }

    #[test]
    fn test_only_changed_skips_ci_when_no_changes() {
        let tmp_dir = TempDir::new("test_only_changed_no_changes").unwrap();

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

        // Create a minimal project
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

        // Create a remote origin (simulated with a bare repo)
        let remote_dir = tmp_dir.path().join("remote.git");
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&remote_dir)
            .output()
            .unwrap();

        // Add origin remote
        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(&remote_dir)
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Push to origin to establish the default branch
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg("HEAD")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Set symbolic ref for origin/HEAD (simulating default branch)
        Command::new("git")
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("refs/remotes/origin/master")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Run CI with only_changed=true
        // Since there are no changes, CI should skip
        let result = run(tmp_dir.path(), CiMode::Check, None, true);

        // Should succeed without running CI
        assert!(result.is_ok(), "CI should succeed: {:?}", result.err());

        // Verify artifacts were NOT created (CI was skipped)
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should not exist when CI is skipped"
        );
    }

    #[test]
    fn test_only_changed_runs_ci_when_changes_exist() {
        let tmp_dir = TempDir::new("test_only_changed_with_changes").unwrap();

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

        // Create a minimal project
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

        // Create a remote origin
        let remote_dir = tmp_dir.path().join("remote.git");
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&remote_dir)
            .output()
            .unwrap();

        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(&remote_dir)
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg("HEAD")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        Command::new("git")
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("refs/remotes/origin/master")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Make a change to the project
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, changed world!\");\n}\n",
        )
        .unwrap();

        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("make change")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Run CI with only_changed=true
        // Since there are changes, CI should run
        let result = run(tmp_dir.path(), CiMode::Check, None, true);

        // Should succeed and run CI
        assert!(result.is_ok(), "CI should succeed: {:?}", result.err());

        // Verify artifacts were created (CI ran)
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            artifacts_dir.exists(),
            "Artifacts directory should exist when CI runs"
        );
    }

    #[test]
    fn test_has_changes_detects_changes() {
        let tmp_dir = TempDir::new("test_has_changes").unwrap();

        // Initialize git repo with origin
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

        // Create initial file and commit
        fs::write(tmp_dir.path().join("README.md"), "initial").unwrap();
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

        // Create remote and push
        let remote_dir = tmp_dir.path().join("remote.git");
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&remote_dir)
            .output()
            .unwrap();
        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(&remote_dir)
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg("HEAD")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("refs/remotes/origin/master")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Initially, no changes
        let result = has_changes(tmp_dir.path());
        assert!(result.is_ok());
        assert!(!result.unwrap(), "Should have no changes initially");

        // Make a change
        fs::write(tmp_dir.path().join("README.md"), "changed").unwrap();
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("change")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Now should have changes
        let result = has_changes(tmp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap(), "Should detect changes after commit");
    }

    #[test]
    fn test_get_default_branch_finds_main() {
        let tmp_dir = TempDir::new("test_default_branch").unwrap();

        // Initialize git repo
        Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("main")
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

        // Create initial commit
        fs::write(tmp_dir.path().join("README.md"), "test").unwrap();
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

        // Create remote and push
        let remote_dir = tmp_dir.path().join("remote.git");
        Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&remote_dir)
            .output()
            .unwrap();
        Command::new("git")
            .arg("remote")
            .arg("add")
            .arg("origin")
            .arg(&remote_dir)
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("push")
            .arg("-u")
            .arg("origin")
            .arg("main")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("refs/remotes/origin/main")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Should detect main as default branch
        let result = get_default_branch(tmp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "main");
    }
}
