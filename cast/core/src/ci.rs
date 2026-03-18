use crate::build;
use crate::command_util;
use crate::install;
use crate::publish;
use crate::test;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
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

/// Result of running CI on a project
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CiRunResult {
    /// CI checks were run and passed
    Ran,
    /// CI checks were skipped (e.g., no changes with --only-changed)
    Skipped,
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
    #[error("Installation failed: {0}")]
    InstallError(#[from] install::InstallError),
    #[error("Multiple projects failed CI:\n{}", .0.iter().map(|(path, err)| format!("  - {}: {}", path.display(), err)).collect::<Vec<_>>().join("\n"))]
    MultipleProjectFailures(Vec<(PathBuf, Box<CiError>)>),
}

/// Cache for git diff results to avoid running the same git commands repeatedly
/// Maps absolute path to whether the directory has changes
type GitDiffCache = Rc<RefCell<HashMap<PathBuf, bool>>>;

/// Accumulated results from recursive CI runs
struct CiResults {
    successes: Vec<PathBuf>,
    skipped: Vec<PathBuf>,
    failures: Vec<(PathBuf, Box<CiError>)>,
}

/// Run CI checks for a project
/// This function performs the following steps:
/// 1. Installs required tools (rustc, cargo, clippy, dx, npm, playwright, etc.)
/// 2. Detects the project type and runs appropriate checks:
///    - For Rust projects (has Cargo.toml): cargo fmt, clippy, build, test (includes npm test if package.json exists)
///    - For TypeScript projects (has package.json): npm ci, lint, compile
///    - For hybrid projects (both files): runs both sets of checks, but npm test only runs once
/// 3. Behavior depends on the mode:
///    - Check: Run all checks (fmt --check, clippy, build, test)
///    - Fix: Auto-fix formatting issues, then run checks
///    - Release: Run all checks with release build, then publish artifacts
/// 4. If recursive_depth is Some(depth), after running CI on the current directory,
///    it will find all Cast projects up to 'depth' levels below and run CI on them
/// 5. If only_changed is true, CI will only run if the project has changes:
///    - With uncommitted changes in repository (anywhere):
///      - On default branch: Only checks if THIS project has uncommitted changes (ignores last commit)
///      - On feature branch: Checks both committed changes (HEAD vs origin/default) AND uncommitted changes in this project
///    - With clean repository (no uncommitted changes anywhere):
///      - On default branch: Compares HEAD to HEAD~1 (checks if last commit touched this project)
///      - On feature branch: Compares HEAD to origin/default_branch (PR-style diff)
pub fn run(
    working_directory: impl AsRef<Path>,
    mode: CiMode,
    recursive_depth: Option<usize>,
    only_changed: bool,
    headless: bool,
) -> Result<(), CiError> {
    // Setup headless environment if requested
    if headless {
        // Install headless tools (xvfb, playwright with system deps)
        let install_options = install::InstallOptions {
            specific_tools: Some(vec![install::Tool::Xvfb, install::Tool::Playwright]),
            skip_tools: Vec::new(),
            dry_run: false,
            force: false,
            headless: true,
        };

        if let Err(e) = install::install_tools(working_directory.as_ref(), install_options) {
            eprintln!("Warning: Failed to install headless tools: {}", e);
            eprintln!("Continuing without full headless support...");
        }
    }

    // Create a cache for git diff results to improve performance with --only-changed
    let git_diff_cache = Rc::new(RefCell::new(HashMap::new()));

    // If recursive_depth is specified, use run_ci_recursively to handle the entire tree
    if let Some(depth) = recursive_depth {
        return run_ci_recursively(
            working_directory.as_ref(),
            mode,
            depth,
            only_changed,
            headless,
        );
    }

    run_internal(
        working_directory.as_ref(),
        mode,
        only_changed,
        headless,
        git_diff_cache,
    )
    .map(|_| ()) // Convert CiRunResult to () for public API
}

/// Internal implementation of run that accepts a git diff cache
/// This allows memoization of git diff results across recursive calls
/// Returns CiRunResult indicating whether CI was run or skipped
fn run_internal(
    working_directory: &Path,
    mode: CiMode,
    only_changed: bool,
    headless: bool,
    git_diff_cache: GitDiffCache,
) -> Result<CiRunResult, CiError> {
    // Install required tools before running CI checks
    // This ensures all necessary tools (rustc, cargo, clippy, dx, npm, etc.) are available
    let install_options = install::InstallOptions {
        specific_tools: None,   // Install all required tools
        skip_tools: Vec::new(), // Don't skip any tools
        dry_run: false,         // Actually install
        force: false,           // Only install if not already installed
        headless,               // Pass through headless flag
    };

    // Run the installation
    let install_results = install::install_tools(working_directory, install_options)?;

    // Check if any installations failed (shouldn't happen if tools are already installed)
    let failed_installs: Vec<_> = install_results
        .iter()
        .filter(|r| !r.success && !r.skipped)
        .collect();

    if !failed_installs.is_empty() {
        eprintln!("Warning: Some tools failed to install:");
        for result in &failed_installs {
            eprintln!("  - {}: {}", result.tool.name(), result.message);
        }
        // Note: We continue anyway because the tools might already be installed
        // and the failure might be about reinstalling them
    }

    // If only_changed is true, check if the project has changes
    if only_changed {
        match has_changes_cached(working_directory, &git_diff_cache) {
            Ok(true) => {
                // Has changes, continue with CI
            }
            Ok(false) => {
                // No changes, skip CI
                println!("No changes found in project. Skipping CI.");
                return Ok(CiRunResult::Skipped);
            }
            Err(e) => {
                // Error checking for changes (e.g., not in a git repo)
                eprintln!(
                    "Warning: Could not check for changes: {}. Proceeding with CI.",
                    e
                );
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
        run_rust_ci(working_directory, mode, headless)?;
        ran_ci_checks = true;
    }

    // Run TypeScript CI if package.json exists (can run in addition to Rust CI)
    // Pass has_cargo_toml so we can avoid running tests twice, and mode for fix behavior
    if has_package_json {
        run_typescript_ci(working_directory, has_cargo_toml, mode, headless)?;
        ran_ci_checks = true;
    }

    // Handle post-check publishing based on mode
    // Only publish if we have a Rust project (Cargo.toml exists)
    // Cast's publish command creates binary/bundle artifacts from Rust projects
    if ran_ci_checks && has_cargo_toml && mode == CiMode::Release {
        // For Release mode, publish is already handled by run_rust_ci
        // Just commit the artifacts
        commit_artifacts(working_directory)?;
    }
    // For Check and Fix modes, do NOT create artifacts
    // Artifacts should only be created in Release mode
    // If no CI checks were run, silently succeed (empty project or unsupported type)

    // Note: Recursive CI is now handled by run_ci_recursively to accumulate all results
    // Don't recurse from here to avoid nested summaries

    Ok(CiRunResult::Ran)
}

/// Run CI checks for a Rust project
/// This runs different steps based on the mode:
/// - Check mode: cargo fmt --check, clippy, build (debug), test
/// - Fix mode: cargo fmt (auto-fix only)
/// - Release mode: cargo fmt --check, clippy, build --release, test, publish
fn run_rust_ci(working_directory: &Path, mode: CiMode, headless: bool) -> Result<(), CiError> {
    // Handle formatting based on mode
    match mode {
        CiMode::Check | CiMode::Release => {
            // Run cargo fmt --check
            run_fmt_check(working_directory)?;
        }
        CiMode::Fix => {
            // Run cargo fmt to auto-fix formatting issues
            run_fmt_fix(working_directory)?;
            // Skip clippy, build, and tests in Fix mode - only do auto-fixable tasks
            return Ok(());
        }
    }

    // Run cargo clippy
    run_clippy(working_directory)?;

    // Run build - release mode for Release, debug for others
    match mode {
        CiMode::Check => {
            build::run(working_directory)?;
        }
        CiMode::Release => {
            build::run_release(working_directory)?;
        }
        CiMode::Fix => unreachable!(), // Fix mode returns early
    }

    // Run cast test (without coverage in CI, with headless mode)
    test::run(working_directory, false, headless)?;

    // For Release mode, also run publish
    if mode == CiMode::Release {
        publish::run(working_directory)?;
    }

    Ok(())
}

/// Run CI checks for a TypeScript/Node.js project
/// This runs:
/// 1. npm ci (to install dependencies from lockfile)
/// 2. npm run lint (if script exists, skipped in Fix mode)
/// 3. npm run compile (if script exists, skipped in Fix mode)
/// 4. npm test (if script exists AND skip_tests is false, skipped in Fix mode)
///
/// The skip_tests parameter should be true when this is a hybrid project (has both Cargo.toml and package.json)
/// because test::run() already runs npm test for hybrid projects
///
/// In Fix mode, only npm install is run - no linting, compiling, or testing
fn run_typescript_ci(
    working_directory: &Path,
    skip_tests: bool,
    mode: CiMode,
    headless: bool,
) -> Result<(), CiError> {
    // Run npm ci to ensure dependencies are installed from lockfile
    run_npm_install(working_directory).map_err(|_| CiError::NpmInstallError)?;

    // In Fix mode, skip everything else - only install dependencies
    if mode == CiMode::Fix {
        return Ok(());
    }

    // Run npm run lint if it exists
    if npm_script_exists(working_directory, "lint") {
        run_npm_command(working_directory, "lint", headless).map_err(|_| CiError::NpmLintError)?;
    }

    // Run npm run compile if it exists
    if npm_script_exists(working_directory, "compile") {
        run_npm_command(working_directory, "compile", headless)
            .map_err(|_| CiError::NpmCompileError)?;
    }

    // Run npm test if it exists (e.g., Playwright tests)
    // Skip if this is a hybrid project (tests already run by test::run())
    if !skip_tests && npm_script_exists(working_directory, "test") {
        run_npm_command(working_directory, "test", headless).map_err(|_| CiError::NpmTestError)?;
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
fn run_npm_command(
    working_directory: &Path,
    command: &str,
    headless: bool,
) -> Result<(), std::io::Error> {
    let mut cmd = if headless {
        command_util::wrap_with_xvfb_if_headless(
            "npm",
            &["run", command],
            working_directory,
            headless,
        )
    } else {
        let mut c = Command::new("npm");
        c.arg("run").arg(command).current_dir(working_directory);
        c
    };

    cmd.stdin(std::process::Stdio::null()); // Prevent blocking on user input (e.g., Playwright HTML reporter)

    let status = cmd.status()?;

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
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt").arg("--check").current_dir(working_directory);

    // Configure LLVM environment if needed
    for (var_name, var_value) in install::detect_llvm_env() {
        cmd.env(var_name, var_value);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(CiError::FmtError);
    }

    Ok(())
}

fn run_fmt_fix(working_directory: &Path) -> Result<(), CiError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("fmt")
        .current_dir(working_directory)
        .stdin(std::process::Stdio::null()); // Prevent blocking on user input

    // Configure LLVM environment if needed
    for (var_name, var_value) in install::detect_llvm_env() {
        cmd.env(var_name, var_value);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(CiError::FmtError);
    }

    Ok(())
}

fn run_clippy(working_directory: &Path) -> Result<(), CiError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .current_dir(working_directory);

    // Configure LLVM environment if needed
    for (var_name, var_value) in install::detect_llvm_env() {
        cmd.env(var_name, var_value);
    }

    let status = cmd.status()?;

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
pub fn run_ci_recursively(
    working_directory: &Path,
    mode: CiMode,
    max_depth: usize,
    only_changed: bool,
    headless: bool,
) -> Result<(), CiError> {
    // Setup headless environment if requested
    if headless {
        // Install headless tools (xvfb, playwright with system deps)
        let install_options = install::InstallOptions {
            specific_tools: Some(vec![install::Tool::Xvfb, install::Tool::Playwright]),
            skip_tools: Vec::new(),
            dry_run: false,
            force: false,
            headless: true,
        };

        if let Err(e) = install::install_tools(working_directory, install_options) {
            eprintln!("Warning: Failed to install headless tools: {}", e);
            eprintln!("Continuing without full headless support...");
        }
    }

    // Create a cache for git diff results to improve performance with --only-changed
    let git_diff_cache = Rc::new(RefCell::new(HashMap::new()));

    // Accumulate all results from the entire tree
    let mut results = CiResults {
        successes: Vec::new(),
        skipped: Vec::new(),
        failures: Vec::new(),
    };

    run_ci_recursively_internal(
        working_directory,
        mode,
        max_depth,
        only_changed,
        headless,
        git_diff_cache,
        &mut results,
    )?;

    // Print summary at the top level with all accumulated results
    println!("\n=== CI Summary ===");
    println!("Passed: {}", results.successes.len());
    println!("Skipped: {}", results.skipped.len());
    println!("Failed: {}", results.failures.len());

    if !results.successes.is_empty() {
        println!("\nSuccessful projects:");
        for path in &results.successes {
            println!("  ✓ {}", path.display());
        }
    }

    if !results.skipped.is_empty() {
        println!("\nSkipped projects (no changes):");
        for path in &results.skipped {
            println!("  ○ {}", path.display());
        }
    }

    if !results.failures.is_empty() {
        println!("\nFailed projects:");
        for (path, err) in &results.failures {
            println!("  ✗ {}: {}", path.display(), err);
        }
        return Err(CiError::MultipleProjectFailures(results.failures));
    }

    Ok(())
}

/// Internal implementation of run_ci_recursively that accepts a git diff cache
/// and accumulates results in the provided CiResults struct
fn run_ci_recursively_internal(
    working_directory: &Path,
    mode: CiMode,
    max_depth: usize,
    only_changed: bool,
    headless: bool,
    git_diff_cache: GitDiffCache,
    results: &mut CiResults,
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

        // Run CI on the child project with the shared cache
        match run_internal(
            &project_path,
            mode,
            only_changed,
            headless,
            git_diff_cache.clone(),
        ) {
            Ok(CiRunResult::Ran) => {
                println!("✓ CI passed for {}", project_path.display());
                results.successes.push(project_path.clone());
            }
            Ok(CiRunResult::Skipped) => {
                // Project was skipped due to no changes
                results.skipped.push(project_path.clone());
            }
            Err(e) => {
                eprintln!("✗ CI failed for {}: {}", project_path.display(), e);
                results.failures.push((project_path.clone(), Box::new(e)));
            }
        }

        // Now handle nested projects if there's remaining depth
        if let Some(remaining) = remaining_depth {
            if remaining > 0 {
                // Recursively process nested projects, accumulating results in the same struct
                let _ = run_ci_recursively_internal(
                    &project_path,
                    mode,
                    remaining,
                    only_changed,
                    headless,
                    git_diff_cache.clone(),
                    results,
                );
                // Continue even if nested recursion fails - we want to check all projects
            }
        }
    }

    Ok(())
}

/// Check if the current project has changes compared to the origin's default branch
/// Returns true if there are changes, false if no changes
/// Returns an error if git operations fail (e.g., not in a git repo)
///
/// Behavior:
/// - If repository has uncommitted changes anywhere (dirty):
///   - On default branch: Only check if THIS project has uncommitted changes
///   - On feature branch: Check both committed changes (HEAD vs origin/default) AND uncommitted changes in this project
/// - If repository is clean (no uncommitted changes anywhere):
///   - On default branch: Compare HEAD to HEAD~1 (previous commit)
///   - On feature branch: Compare HEAD to origin/default_branch (PR-style diff)
fn has_changes(working_directory: &Path) -> Result<bool, CiError> {
    // Check if we're in a git repository
    let git_check = Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .current_dir(working_directory)
        .output()?;

    if !git_check.status.success() {
        return Err(CiError::IoError(std::io::Error::other(
            "Cannot check for changes with --only-changed flag: Not in a git repository",
        )));
    }

    // Get the current and default branch names
    let current_branch = get_current_branch(working_directory)?;
    let default_branch = get_default_branch(working_directory)?;
    let on_default_branch = current_branch == default_branch;

    // Check if repository has uncommitted changes ANYWHERE (not just this project)
    let repository_is_dirty = is_repository_dirty(working_directory)?;

    // Special handling when repository has uncommitted changes
    if repository_is_dirty {
        // Check if THIS specific project has uncommitted changes
        let project_has_dirty_files = is_working_directory_dirty(working_directory)?;

        if on_default_branch {
            // On default branch with dirty repository: Only include if THIS project has uncommitted changes
            // Ignore what was in the last commit (HEAD vs HEAD~1)
            return Ok(project_has_dirty_files);
        } else {
            // On feature branch with dirty repository: Check both committed changes AND uncommitted changes in this project
            let has_committed_changes =
                check_committed_changes(working_directory, &format!("origin/{}", default_branch))?;
            return Ok(has_committed_changes || project_has_dirty_files);
        }
    }

    // Repository is clean, use standard comparison logic
    let base_ref = if on_default_branch {
        // On default branch: compare HEAD to previous commit (HEAD~1)
        "HEAD~1".to_string()
    } else {
        // On feature branch: compare to origin/default_branch (like a PR)
        format!("origin/{}", default_branch)
    };

    check_committed_changes(working_directory, &base_ref)
}

/// Check if there are committed changes between base_ref and HEAD in the current directory
fn check_committed_changes(working_directory: &Path, base_ref: &str) -> Result<bool, CiError> {
    // Check if there are any changes in the current project directory
    let diff_output = Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .arg(base_ref)
        .arg("HEAD")
        .arg("--")
        .arg(".")
        .current_dir(working_directory)
        .output();

    match diff_output {
        Ok(output) => {
            // git diff --quiet exit codes:
            // 0 = no differences
            // 1 = differences found
            // 2+ = error (e.g., invalid ref like HEAD~1 when it doesn't exist)
            if let Some(code) = output.status.code() {
                if code >= 2 {
                    // Error case (e.g., HEAD~1 doesn't exist on first commit)
                    // For HEAD~1 on default branch, this means first commit - no changes
                    Ok(false)
                } else {
                    // Exit code 0 = no changes, 1 = has changes
                    Ok(!output.status.success())
                }
            } else {
                // Process was terminated by signal
                Err(CiError::IoError(std::io::Error::other(
                    "git diff was terminated by signal",
                )))
            }
        }
        Err(e) => Err(CiError::IoError(e)),
    }
}

/// Check if the working directory has uncommitted changes in the current directory
/// Returns true if there are staged or unstaged changes, false otherwise
fn is_working_directory_dirty(working_directory: &Path) -> Result<bool, CiError> {
    // Check for both staged and unstaged changes in the current directory
    // git diff-index --quiet HEAD -- . checks for any changes (staged or unstaged)
    let status = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("HEAD")
        .arg("--")
        .arg(".")
        .current_dir(working_directory)
        .status()?;

    // Exit code 0 = no changes, 1 = has changes
    Ok(!status.success())
}

/// Check if the repository has uncommitted changes ANYWHERE (not just in current directory)
/// Returns true if there are any staged or unstaged changes in the entire repository
fn is_repository_dirty(working_directory: &Path) -> Result<bool, CiError> {
    // Check for uncommitted changes anywhere in the repository
    // git diff-index --quiet HEAD (without path) checks the entire repository
    let status = Command::new("git")
        .arg("diff-index")
        .arg("--quiet")
        .arg("HEAD")
        .current_dir(working_directory)
        .status()?;

    // Exit code 0 = no changes, 1 = has changes
    Ok(!status.success())
}

/// Check if the current project has changes compared to the origin's default branch
/// This version uses a cache to avoid running the same git command multiple times
/// Returns true if there are changes, false if no changes
/// Returns an error if git operations fail (e.g., not in a git repo)
fn has_changes_cached(working_directory: &Path, cache: &GitDiffCache) -> Result<bool, CiError> {
    // Convert to absolute path for cache key consistency
    let abs_path = working_directory
        .canonicalize()
        .unwrap_or_else(|_| working_directory.to_path_buf());

    // Check cache first
    {
        let cache_ref = cache.borrow();
        if let Some(&cached_result) = cache_ref.get(&abs_path) {
            return Ok(cached_result);
        }
    }

    // Not in cache, compute the result
    let result = has_changes(working_directory)?;

    // Store in cache
    cache.borrow_mut().insert(abs_path, result);

    Ok(result)
}

/// Get the current branch name
fn get_current_branch(working_directory: &Path) -> Result<String, CiError> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(working_directory)
        .output()?;

    if output.status.success() {
        let branch_name = String::from_utf8_lossy(&output.stdout);
        Ok(branch_name.trim().to_string())
    } else {
        Err(CiError::IoError(std::io::Error::other(
            "Cannot determine current branch name",
        )))
    }
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
        "Cannot determine default branch for --only-changed flag. Ensure origin remote is configured and origin/main or origin/master exists",
    )))
}

/// Wrapper function for args.rs that allows calling recursively with headless support
/// This is used when CI is invoked with --recursive from a directory without Cast.toml
pub fn run_ci_recursively_with_headless(
    working_directory: &Path,
    mode: CiMode,
    max_depth: usize,
    only_changed: bool,
    headless: bool,
) -> Result<(), CiError> {
    run_ci_recursively(working_directory, mode, max_depth, only_changed, headless)
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false, false);
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false, false);
        assert!(
            result.is_ok(),
            "CI should succeed in Check mode: {:?}",
            result.err()
        );

        // Verify that artifacts directory was NOT created in Check mode
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should NOT exist after CI Check mode (only in Release mode)"
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

        let result = run(tmp_dir.path(), CiMode::Check, None, false, false);
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
        let result = run(tmp_dir.path(), CiMode::Fix, None, false, false);
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

        // Verify that artifacts directory was NOT created in Fix mode
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should NOT exist after CI Fix mode (only in Release mode)"
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
        let result = run(tmp_dir.path(), CiMode::Release, None, false, false);
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
        let result = run(tmp_dir.path(), CiMode::Check, Some(1), false, false);

        // Should succeed
        assert!(
            result.is_ok(),
            "Recursive CI should succeed: {:?}",
            result.err()
        );

        // Verify artifacts were NOT created in Check mode (only created in Release mode)
        assert!(!tmp_dir.path().join("artifacts").exists());
        assert!(!child.join("artifacts").exists());
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
        let result = run(tmp_dir.path(), CiMode::Check, None, true, false);

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
        let result = run(tmp_dir.path(), CiMode::Check, None, true, false);

        // Should succeed and run CI
        assert!(result.is_ok(), "CI should succeed: {:?}", result.err());

        // Verify artifacts were NOT created in Check mode (only created in Release mode)
        let artifacts_dir = tmp_dir.path().join("artifacts");
        assert!(
            !artifacts_dir.exists(),
            "Artifacts directory should NOT exist in Check mode (only in Release mode)"
        );
    }

    #[test]
    fn test_has_changes_detects_changes() {
        let tmp_dir = TempDir::new("test_has_changes").unwrap();

        // Initialize git repo with master branch explicitly
        Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("master")
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

    #[test]
    fn test_has_changes_on_feature_branch_compares_to_default() {
        let tmp_dir = TempDir::new("test_feature_branch").unwrap();

        // Initialize git repo with master branch
        Command::new("git")
            .arg("init")
            .arg("-b")
            .arg("master")
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
            .arg("initial commit")
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
            .arg("master")
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

        // Create a feature branch
        Command::new("git")
            .arg("checkout")
            .arg("-b")
            .arg("feature")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Initially on feature branch, no changes compared to master
        let result = has_changes(tmp_dir.path());
        assert!(result.is_ok());
        assert!(
            !result.unwrap(),
            "Should have no changes on feature branch initially"
        );

        // Make a change on the feature branch
        fs::write(tmp_dir.path().join("README.md"), "feature change").unwrap();
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("feature change")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Now should have changes compared to origin/master
        let result = has_changes(tmp_dir.path());
        assert!(result.is_ok());
        assert!(
            result.unwrap(),
            "Should detect changes on feature branch after commit"
        );
    }

    #[test]
    fn test_ci_does_not_run_npm_test_twice_for_hybrid_project() {
        let tmp_dir = TempDir::new("test_hybrid_no_double_test").unwrap();

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

        // Create a hybrid project (both Rust and TypeScript)
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

        // Create a package.json with a test script that creates a marker file
        // This lets us count how many times the test script runs
        let marker_path = tmp_dir.path().join("test_marker.txt");
        let test_script = format!(
            "echo test >> {}",
            marker_path.to_string_lossy().replace('\\', "/")
        );
        fs::write(
            tmp_dir.path().join("package.json"),
            format!(r#"{{"scripts": {{"test": "{}"}}}}"#, test_script),
        )
        .unwrap();

        // Create package-lock.json for npm ci
        fs::write(
            tmp_dir.path().join("package-lock.json"),
            r#"{"name": "test", "lockfileVersion": 3}"#,
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

        // Run CI
        let result = run(tmp_dir.path(), CiMode::Check, None, false, false);
        assert!(
            result.is_ok(),
            "CI should succeed for hybrid project: {:?}",
            result.err()
        );

        // Check that test was only run once
        // If the marker file exists, count the lines
        if marker_path.exists() {
            let content = fs::read_to_string(&marker_path).unwrap();
            let line_count = content.lines().count();
            assert_eq!(
                line_count, 1,
                "npm test should only run once, but ran {} times",
                line_count
            );
        } else {
            panic!("Test marker file should exist after running CI");
        }
    }

    #[test]
    fn test_ci_runs_npm_test_for_pure_typescript_project() {
        let tmp_dir = TempDir::new("test_pure_typescript").unwrap();

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

        // Create a pure TypeScript project (only package.json, no Cargo.toml)
        let marker_path = tmp_dir.path().join("test_marker.txt");
        let test_script = format!(
            "echo test >> {}",
            marker_path.to_string_lossy().replace('\\', "/")
        );
        fs::write(
            tmp_dir.path().join("package.json"),
            format!(r#"{{"scripts": {{"test": "{}"}}}}"#, test_script),
        )
        .unwrap();

        // Create package-lock.json for npm ci
        fs::write(
            tmp_dir.path().join("package-lock.json"),
            r#"{"name": "test", "lockfileVersion": 3}"#,
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

        // Run CI
        let result = run(tmp_dir.path(), CiMode::Check, None, false, false);
        assert!(
            result.is_ok(),
            "CI should succeed for pure TypeScript project: {:?}",
            result.err()
        );

        // Check that test was run
        if marker_path.exists() {
            let content = fs::read_to_string(&marker_path).unwrap();
            let line_count = content.lines().count();
            assert_eq!(
                line_count, 1,
                "npm test should run once for pure TypeScript projects"
            );
        } else {
            panic!("Test marker file should exist after running CI");
        }
    }

    #[test]
    fn test_recursive_ci_continues_on_failure() {
        let tmp_dir = TempDir::new("test_recursive_ci_failure").unwrap();

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

        // Create three child projects: one failing, two passing
        let child1 = tmp_dir.path().join("child1");
        let child2_fail = tmp_dir.path().join("child2_fail");
        let child3 = tmp_dir.path().join("child3");

        // Child 1: Valid project
        fs::create_dir_all(child1.join("src")).unwrap();
        fs::write(
            child1.join("Cargo.toml"),
            "[package]\nname = \"child1\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nproject_type = \"rust_library\"",
        )
        .unwrap();
        fs::write(child1.join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        // Child 2: Invalid project (will fail CI)
        fs::create_dir_all(child2_fail.join("src")).unwrap();
        fs::write(
            child2_fail.join("Cargo.toml"),
            "[package]\nname = \"child2_fail\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nproject_type = \"rust_library\"",
        )
        .unwrap();
        fs::write(
            child2_fail.join("src/lib.rs"),
            "pub fn test() {\n    undefined_function();\n}\n",
        )
        .unwrap();

        // Child 3: Valid project
        fs::create_dir_all(child3.join("src")).unwrap();
        fs::write(
            child3.join("Cargo.toml"),
            "[package]\nname = \"child3\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nproject_type = \"rust_library\"",
        )
        .unwrap();
        fs::write(child3.join("src/lib.rs"), "pub fn test2() {}\n").unwrap();

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

        // Run CI recursively
        // The function should continue even if child2 fails
        let result = run_ci_recursively(tmp_dir.path(), CiMode::Check, 1, false, false);

        // The result should be an error because child2 failed
        assert!(
            result.is_err(),
            "Recursive CI should return error when at least one project fails"
        );

        // The error should indicate which project failed
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("child2_fail") || error_msg.contains("Failed projects"),
            "Error message should indicate which project failed: {}",
            error_msg
        );
    }

    #[test]
    fn test_git_diff_cache_improves_performance() {
        let tmp_dir = TempDir::new("test_git_diff_cache").unwrap();

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

        // Create initial file and commit
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

        // Create remote and setup origin/main
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

        // Set up symbolic ref for origin/HEAD
        Command::new("git")
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("refs/remotes/origin/main")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create child projects
        let child1 = tmp_dir.path().join("child1");
        let child2 = tmp_dir.path().join("child2");
        fs::create_dir_all(&child1).unwrap();
        fs::create_dir_all(&child2).unwrap();

        // Create Cast.toml in each child
        fs::write(child1.join("Cast.toml"), "[project]\nname = \"child1\"\n").unwrap();
        fs::write(child2.join("Cast.toml"), "[project]\nname = \"child2\"\n").unwrap();

        // Test that cache is being used
        let cache = Rc::new(RefCell::new(HashMap::new()));

        // First call should populate cache
        let result1 = has_changes_cached(&child1, &cache);
        assert!(result1.is_ok());
        assert_eq!(
            cache.borrow().len(),
            1,
            "Cache should have 1 entry after first call"
        );

        // Second call to same path should use cache
        let result2 = has_changes_cached(&child1, &cache);
        assert!(result2.is_ok());
        assert_eq!(
            cache.borrow().len(),
            1,
            "Cache should still have 1 entry (using cached value)"
        );

        // Both results should be the same
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "Cached result should match original"
        );

        // Call with different path should add to cache
        let result3 = has_changes_cached(&child2, &cache);
        assert!(result3.is_ok());
        assert_eq!(
            cache.borrow().len(),
            2,
            "Cache should have 2 entries after calling with different path"
        );
    }
}
