use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

use crate::config::CastConfig;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewProjectError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
    #[error("No exemplar projects found")]
    NoExemplarProjects,
}

#[derive(Error, Debug)]
pub enum WithChangesError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Git error: {0}")]
    GitError(String),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub fn new(working_directory: impl AsRef<Path>, name: &str) -> Result<(), NewProjectError> {
    let working_directory = working_directory.as_ref();
    let destination = working_directory.join(name);

    // Find all exemplar projects
    let exemplar_projects = find_exemplar_projects(working_directory)?;

    if exemplar_projects.is_empty() {
        return Err(NewProjectError::NoExemplarProjects);
    }

    // Copy each exemplar project to the destination (later ones overwrite earlier ones)
    for exemplar_path in exemplar_projects {
        copy_dir_all(&exemplar_path, &destination)?;
    }

    // Delete unnecessary .gitignore files (empty placeholder files)
    delete_empty_gitignores(&destination)?;

    // Remove exemplar flag from the new project's Cast.toml
    remove_exemplar_flag(&destination)?;

    Ok(())
}

/// Find all exemplar projects by scanning for Cast.toml files with exemplar = true
/// Searches recursively through the entire monorepo starting from working_directory.
/// Any project can be an exemplar - it's not limited to specific directories.
fn find_exemplar_projects(working_directory: &Path) -> Result<Vec<PathBuf>, NewProjectError> {
    let mut exemplar_projects = Vec::new();

    // Recursively search the entire monorepo for exemplar projects
    find_exemplars_recursive(working_directory, &mut exemplar_projects)?;

    // Sort to ensure consistent ordering (alphabetical by path name)
    exemplar_projects.sort();

    Ok(exemplar_projects)
}

/// Recursively find exemplar projects in a directory tree
/// This searches the entire directory tree, not just immediate subdirectories.
/// Skips common directories that shouldn't contain projects (target, node_modules, .git, etc.)
fn find_exemplars_recursive(dir: &Path, exemplars: &mut Vec<PathBuf>) -> io::Result<()> {
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
        {
            return Ok(());
        }
    }

    // Check if this directory is an exemplar project
    // CastConfig::load_from_dir already handles checking for both Cast.toml and Cargo.toml
    if let Ok(config) = CastConfig::load_from_dir(dir) {
        if config.exemplar == Some(true) {
            exemplars.push(dir.to_path_buf());
        }
    }

    // Recursively search subdirectories (skip build directories)
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // The recursive call will check if this is a build directory
            find_exemplars_recursive(&path, exemplars)?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn delete_empty_gitignores(dir: impl AsRef<Path>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if entry.file_type()?.is_dir() {
            delete_empty_gitignores(&path)?;
        } else if path.file_name() == Some(std::ffi::OsStr::new(".gitignore")) {
            // Check if the file is empty
            if fs::metadata(&path)?.len() == 0 {
                fs::remove_file(&path)?;
            }
        }
    }
    Ok(())
}

/// Remove the exemplar flag from a project's configuration file
/// Note: We only modify Cast.toml if it exists. If the project uses Cargo.toml
/// with [package.metadata.cast], the exemplar flag will remain in Cargo.toml.
/// This is intentional to avoid modifying Cargo.toml, which may contain other
/// important package information. Users should manually remove the exemplar flag
/// from Cargo.toml if desired, or create a Cast.toml file in the new project.
fn remove_exemplar_flag(project_dir: &Path) -> Result<(), NewProjectError> {
    // Prioritize Cast.toml for writing (simpler format)
    let cast_toml_path = project_dir.join("Cast.toml");

    if cast_toml_path.exists() {
        let mut config = CastConfig::load(&cast_toml_path)?;
        config.exemplar = None;
        config.save(&cast_toml_path)?;
    }

    Ok(())
}

/// Find projects with Cast.toml that have changes between two git refs
pub fn with_changes(
    working_directory: impl AsRef<Path>,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<PathBuf>, WithChangesError> {
    let working_directory = working_directory.as_ref();

    // Get changed files using git diff
    let changed_files = get_changed_files(working_directory, base_ref, head_ref)?;

    // Find projects with Cast.toml that contain these changed files
    let mut changed_projects = HashSet::new();

    for relative_path in changed_files {
        let file_path = working_directory.join(&relative_path);

        // Walk up the directory tree to find a Cast.toml
        if let Some(project_dir) = find_project_dir(&file_path, working_directory) {
            changed_projects.insert(project_dir);
        }
    }

    // Convert to sorted vector for consistent output
    let mut projects: Vec<PathBuf> = changed_projects.into_iter().collect();
    projects.sort();

    Ok(projects)
}

/// Get list of changed files between two git refs
fn get_changed_files(
    repo_dir: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<Vec<String>, WithChangesError> {
    // Validate refs to prevent command injection
    if !is_valid_git_ref(base_ref) || !is_valid_git_ref(head_ref) {
        return Err(WithChangesError::GitError(
            "Invalid git ref format".to_string(),
        ));
    }

    let output = Command::new("git")
        .arg("diff")
        .arg("--name-only")
        .arg(base_ref)
        .arg(head_ref)
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(WithChangesError::GitError(format!(
            "git diff failed: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let files: Vec<String> = stdout
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    Ok(files)
}

/// Validate that a string is a valid git ref format
/// Allows: alphanumeric, /, -, _, ., ^, ~, and SHA hashes
fn is_valid_git_ref(git_ref: &str) -> bool {
    if git_ref.is_empty() || git_ref.len() > 256 {
        return false;
    }

    // Allow common git ref patterns: branch names, tags, SHAs, HEAD, etc.
    git_ref.chars().all(|c| {
        c.is_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.' || c == '^' || c == '~'
    })
}

/// Find the project directory containing a Cast.toml or Cargo.toml for a given file path
fn find_project_dir(file_path: &Path, repo_root: &Path) -> Option<PathBuf> {
    let mut current = file_path;

    // If the file path doesn't exist (might be deleted), try its parent
    if !current.exists() {
        current = file_path.parent()?;
    }

    // If it's a file, start from its directory
    if current.is_file() {
        current = current.parent()?;
    }

    // Walk up the directory tree looking for Cast.toml or Cargo.toml with Cast metadata
    while current.starts_with(repo_root) {
        let cast_toml = current.join("Cast.toml");
        let cargo_toml = current.join("Cargo.toml");

        // Cast.toml always marks a Cast project
        if cast_toml.exists() {
            let relative = current.strip_prefix(repo_root).ok()?;
            if relative.as_os_str().is_empty() {
                return Some(PathBuf::from("."));
            }
            return Some(relative.to_path_buf());
        }

        // Cargo.toml only marks a Cast project if it has Cast metadata
        if cargo_toml.exists() {
            if let Ok(config) = CastConfig::load_from_cargo_toml(&cargo_toml) {
                if config.has_cast_metadata() {
                    let relative = current.strip_prefix(repo_root).ok()?;
                    if relative.as_os_str().is_empty() {
                        return Some(PathBuf::from("."));
                    }
                    return Some(relative.to_path_buf());
                }
            }
        }

        // Stop if we've reached the repo root
        if current == repo_root {
            break;
        }

        current = current.parent()?;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new_creates_project_from_exemplars() {
        let tmp_dir = TempDir::new("test_new_project").unwrap();

        // Create mock projects directory structure with exemplar projects
        let projects_base = tmp_dir.path().join("projects/base");
        let projects_library = tmp_dir.path().join("projects/library");

        // Create base exemplar with some files and directories
        fs::create_dir_all(projects_base.join("src")).unwrap();
        fs::create_dir_all(projects_base.join("docs")).unwrap();
        fs::write(projects_base.join("README.md"), "# Base README").unwrap();
        fs::write(projects_base.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();

        // Create library exemplar with Cargo.toml
        fs::create_dir_all(projects_library.join("src")).unwrap();
        fs::write(
            projects_library.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(projects_library.join("src/lib.rs"), "// lib").unwrap();
        fs::write(projects_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Call the new function
        let result = new(tmp_dir.path(), "my_project");
        assert!(result.is_ok());

        // Verify the project was created
        let project_path = tmp_dir.path().join("my_project");
        assert!(project_path.exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src").exists());

        // Verify content from base exemplar
        let readme_content = fs::read_to_string(project_path.join("README.md")).unwrap();
        assert_eq!(readme_content, "# Base README");

        // Verify content from library exemplar (should exist)
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
        assert_eq!(cargo_content, "[package]\nname = \"test\"");
    }

    #[test]
    fn test_new_overwrites_files_from_library_exemplar() {
        let tmp_dir = TempDir::new("test_overwrite").unwrap();

        // Create exemplar projects
        let projects_base = tmp_dir.path().join("projects/base");
        let projects_library = tmp_dir.path().join("projects/library");

        // Create the same file in both exemplars
        fs::create_dir_all(projects_base.join("src")).unwrap();
        fs::create_dir_all(projects_library.join("src")).unwrap();
        fs::write(projects_base.join("src/lib.rs"), "// base version").unwrap();
        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(projects_library.join("src/lib.rs"), "// library version").unwrap();
        fs::write(projects_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Call new
        let result = new(tmp_dir.path(), "test_project");
        assert!(result.is_ok());

        // Verify the library version overwrote the base version
        let lib_content =
            fs::read_to_string(tmp_dir.path().join("test_project/src/lib.rs")).unwrap();
        assert_eq!(lib_content, "// library version");
    }

    #[test]
    fn test_new_deletes_empty_gitignores() {
        let tmp_dir = TempDir::new("test_gitignore").unwrap();

        // Create exemplar projects with empty .gitignore files
        let projects_base = tmp_dir.path().join("projects/base");
        fs::create_dir_all(projects_base.join("src")).unwrap();
        fs::create_dir_all(projects_base.join("docs")).unwrap();

        // Create empty .gitignore files
        fs::write(projects_base.join("src/.gitignore"), "").unwrap();
        fs::write(projects_base.join("docs/.gitignore"), "").unwrap();

        // Create a non-empty .gitignore
        fs::write(projects_base.join(".gitignore"), "target/\n").unwrap();

        // Create Cast.toml to mark as exemplar
        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();

        let projects_library = tmp_dir.path().join("projects/library");
        fs::create_dir_all(&projects_library).unwrap();
        fs::write(projects_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Call new
        let result = new(tmp_dir.path(), "test_project");
        assert!(result.is_ok());

        let project_path = tmp_dir.path().join("test_project");

        // Verify empty .gitignore files were deleted
        assert!(!project_path.join("src/.gitignore").exists());
        assert!(!project_path.join("docs/.gitignore").exists());

        // Verify non-empty .gitignore still exists
        assert!(project_path.join(".gitignore").exists());
        let gitignore_content = fs::read_to_string(project_path.join(".gitignore")).unwrap();
        assert_eq!(gitignore_content, "target/\n");
    }

    #[test]
    fn test_new_returns_error_when_exemplars_missing() {
        let tmp_dir = TempDir::new("test_error").unwrap();

        // Don't create projects directories or exemplar projects
        let result = new(tmp_dir.path(), "test_project");

        // Should return an error since no exemplar projects exist
        assert!(result.is_err());
        match result {
            Err(NewProjectError::NoExemplarProjects) => {}
            _ => panic!("Expected NoExemplarProjects error"),
        }
    }

    #[test]
    fn test_copy_dir_all_copies_nested_directories() {
        let tmp_dir = TempDir::new("test_copy").unwrap();

        let src = tmp_dir.path().join("src");
        let dst = tmp_dir.path().join("dst");

        // Create nested directory structure
        fs::create_dir_all(src.join("a/b/c")).unwrap();
        fs::write(src.join("a/file1.txt"), "content1").unwrap();
        fs::write(src.join("a/b/file2.txt"), "content2").unwrap();
        fs::write(src.join("a/b/c/file3.txt"), "content3").unwrap();

        // Copy
        let result = copy_dir_all(&src, &dst);
        assert!(result.is_ok());

        // Verify structure
        assert!(dst.join("a/file1.txt").exists());
        assert!(dst.join("a/b/file2.txt").exists());
        assert!(dst.join("a/b/c/file3.txt").exists());

        // Verify content
        assert_eq!(
            fs::read_to_string(dst.join("a/file1.txt")).unwrap(),
            "content1"
        );
        assert_eq!(
            fs::read_to_string(dst.join("a/b/file2.txt")).unwrap(),
            "content2"
        );
        assert_eq!(
            fs::read_to_string(dst.join("a/b/c/file3.txt")).unwrap(),
            "content3"
        );
    }

    #[test]
    fn test_find_exemplar_projects_in_projects_directory() {
        let tmp_dir = TempDir::new("test_exemplar").unwrap();

        // Create projects directory with exemplar projects
        let projects_base = tmp_dir.path().join("projects/base");
        let projects_library = tmp_dir.path().join("projects/library");
        fs::create_dir_all(&projects_base).unwrap();
        fs::create_dir_all(&projects_library).unwrap();

        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(projects_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 2);

        // Verify both exemplars are found (sorted order)
        assert!(exemplars[0].ends_with("base"));
        assert!(exemplars[1].ends_with("library"));
    }

    #[test]
    fn test_find_exemplar_projects_filters_non_exemplars() {
        let tmp_dir = TempDir::new("test_exemplar_proj").unwrap();

        // Create projects directory with one exemplar
        let projects_dir = tmp_dir.path().join("projects");
        let exemplar_proj = projects_dir.join("example_lib");
        let normal_proj = projects_dir.join("normal_lib");

        fs::create_dir_all(&exemplar_proj).unwrap();
        fs::create_dir_all(&normal_proj).unwrap();

        // Mark one as exemplar
        fs::write(exemplar_proj.join("Cast.toml"), "exemplar = true").unwrap();
        // The other has no exemplar flag
        fs::write(normal_proj.join("Cast.toml"), "").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 1);
        assert!(exemplars[0].ends_with("example_lib"));
    }

    #[test]
    fn test_find_exemplar_projects_returns_empty_when_none_marked() {
        let tmp_dir = TempDir::new("test_no_exemplar").unwrap();

        // Create projects but don't mark them as exemplars
        let projects_dir = tmp_dir.path().join("projects");
        let proj1 = projects_dir.join("proj1");
        let proj2 = projects_dir.join("proj2");

        fs::create_dir_all(&proj1).unwrap();
        fs::create_dir_all(&proj2).unwrap();

        fs::write(proj1.join("Cast.toml"), "").unwrap();
        fs::write(proj2.join("Cast.toml"), "exemplar = false").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 0);
    }

    #[test]
    fn test_find_exemplar_projects_with_nested_structure() {
        let tmp_dir = TempDir::new("test_nested").unwrap();

        // Create projects with nested exemplars
        let projects_base = tmp_dir.path().join("projects/base");
        let projects_example = tmp_dir.path().join("projects/example");

        fs::create_dir_all(&projects_base).unwrap();
        fs::create_dir_all(&projects_example).unwrap();

        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(projects_example.join("Cast.toml"), "exemplar = true").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 2);
    }

    #[test]
    fn test_new_removes_exemplar_flag_from_created_project() {
        let tmp_dir = TempDir::new("test_exemplar_removal").unwrap();

        // Create exemplar project with exemplar flag
        let projects_base = tmp_dir.path().join("projects/base");
        fs::create_dir_all(&projects_base).unwrap();
        fs::write(
            projects_base.join("Cast.toml"),
            "exemplar = true\nproof_of_concept = false",
        )
        .unwrap();
        fs::write(projects_base.join("README.md"), "# Test").unwrap();

        // Call new
        let result = new(tmp_dir.path(), "test_project");
        assert!(result.is_ok());

        // Verify the new project was created
        let project_path = tmp_dir.path().join("test_project");
        assert!(project_path.exists());

        // Load the Cast.toml and verify exemplar flag is removed
        let config = CastConfig::load(project_path.join("Cast.toml")).unwrap();
        assert_eq!(config.exemplar, None);
        // Other flags should be preserved
        assert_eq!(config.proof_of_concept, Some(false));
    }

    #[test]
    fn test_find_project_dir_finds_cast_toml() {
        let tmp_dir = TempDir::new("test_find_project").unwrap();

        // Create a project with Cast.toml
        let project_dir = tmp_dir.path().join("projects/my_project");
        fs::create_dir_all(project_dir.join("src")).unwrap();
        fs::write(project_dir.join("Cast.toml"), "").unwrap();
        fs::write(project_dir.join("src/lib.rs"), "// test").unwrap();

        // Test finding the project from a file inside it
        let file_path = project_dir.join("src/lib.rs");
        let result = find_project_dir(&file_path, tmp_dir.path());

        assert!(result.is_some());
        let found_project = result.unwrap();
        assert_eq!(found_project, PathBuf::from("projects/my_project"));
    }

    #[test]
    fn test_find_project_dir_returns_none_for_files_without_cast_toml() {
        let tmp_dir = TempDir::new("test_no_project").unwrap();

        // Create a directory structure without Cast.toml
        let dir = tmp_dir.path().join("some_dir");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("file.txt"), "test").unwrap();

        // Test finding the project from a file
        let file_path = dir.join("file.txt");
        let result = find_project_dir(&file_path, tmp_dir.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_find_project_dir_handles_root_cast_toml() {
        let tmp_dir = TempDir::new("test_root_project").unwrap();

        // Create Cast.toml in root
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        fs::write(tmp_dir.path().join("README.md"), "test").unwrap();

        // Test finding the project from a file in root
        let file_path = tmp_dir.path().join("README.md");
        let result = find_project_dir(&file_path, tmp_dir.path());

        assert!(result.is_some());
        let found_project = result.unwrap();
        assert_eq!(found_project, PathBuf::from("."));
    }

    #[test]
    fn test_find_project_dir_finds_cargo_toml() {
        let tmp_dir = TempDir::new("test_find_cargo").unwrap();

        // Create a project with Cargo.toml that has Cast metadata
        let project_dir = tmp_dir.path().join("projects/my_project");
        fs::create_dir_all(project_dir.join("src")).unwrap();
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[package.metadata.cast]\nexemplar = true",
        )
        .unwrap();
        fs::write(project_dir.join("src/lib.rs"), "// test").unwrap();

        // Test finding the project from a file inside it
        let file_path = project_dir.join("src/lib.rs");
        let result = find_project_dir(&file_path, tmp_dir.path());

        assert!(result.is_some());
        let found_project = result.unwrap();
        assert_eq!(found_project, PathBuf::from("projects/my_project"));
    }

    #[test]
    fn test_find_project_dir_prefers_closer_cargo_toml() {
        let tmp_dir = TempDir::new("test_nested_cargo").unwrap();

        // Create nested projects with Cargo.toml that have Cast metadata
        let outer_project = tmp_dir.path().join("outer");
        let inner_project = outer_project.join("inner");

        fs::create_dir_all(inner_project.join("src")).unwrap();
        fs::write(
            outer_project.join("Cargo.toml"),
            "[package]\nname = \"outer\"\n\n[package.metadata.cast]\nexemplar = true",
        )
        .unwrap();
        fs::write(
            inner_project.join("Cargo.toml"),
            "[package]\nname = \"inner\"\n\n[package.metadata.cast]\nframework = \"rust-library\"",
        )
        .unwrap();
        fs::write(inner_project.join("src/lib.rs"), "// test").unwrap();

        // Test finding the project from a file in inner project
        let file_path = inner_project.join("src/lib.rs");
        let result = find_project_dir(&file_path, tmp_dir.path());

        assert!(result.is_some());
        let found_project = result.unwrap();
        assert_eq!(found_project, PathBuf::from("outer/inner"));
    }

    #[test]
    fn test_find_project_dir_ignores_cargo_toml_without_cast_metadata() {
        let tmp_dir = TempDir::new("test_no_cast_metadata").unwrap();

        // Create a regular Rust project without Cast metadata
        let project_dir = tmp_dir.path().join("regular_rust_project");
        fs::create_dir_all(project_dir.join("src")).unwrap();
        fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"regular_project\"\nversion = \"0.1.0\"",
        )
        .unwrap();
        fs::write(project_dir.join("src/lib.rs"), "// regular rust code").unwrap();

        // Test finding the project from a file inside it
        let file_path = project_dir.join("src/lib.rs");
        let result = find_project_dir(&file_path, tmp_dir.path());

        // Should return None because Cargo.toml has no Cast metadata
        assert!(result.is_none());
    }

    #[test]
    fn test_is_valid_git_ref_accepts_valid_refs() {
        assert!(is_valid_git_ref("main"));
        assert!(is_valid_git_ref("feature/my-branch"));
        assert!(is_valid_git_ref("v1.0.0"));
        assert!(is_valid_git_ref("HEAD"));
        assert!(is_valid_git_ref("HEAD~1"));
        assert!(is_valid_git_ref("abc123def456"));
        assert!(is_valid_git_ref("origin/main"));
        assert!(is_valid_git_ref("refs/tags/v1.0.0"));
    }

    #[test]
    fn test_is_valid_git_ref_rejects_invalid_refs() {
        assert!(!is_valid_git_ref(""));
        assert!(!is_valid_git_ref("branch with spaces"));
        assert!(!is_valid_git_ref("branch;rm -rf /"));
        assert!(!is_valid_git_ref("branch&whoami"));
        assert!(!is_valid_git_ref("branch|cat /etc/passwd"));
        assert!(!is_valid_git_ref(&"a".repeat(300))); // Too long
    }

    #[test]
    fn test_find_exemplars_searches_entire_monorepo_recursively() {
        let tmp_dir = TempDir::new("test_recursive").unwrap();

        // Create a nested structure with exemplars at various levels
        // root/workspace1/proj1 (exemplar)
        // root/workspace2/nested/proj2 (exemplar)
        // root/standalone_proj (exemplar)
        // root/workspace3/proj3 (not exemplar)

        let proj1 = tmp_dir.path().join("workspace1/proj1");
        let proj2 = tmp_dir.path().join("workspace2/nested/proj2");
        let standalone = tmp_dir.path().join("standalone_proj");
        let proj3 = tmp_dir.path().join("workspace3/proj3");

        fs::create_dir_all(&proj1).unwrap();
        fs::create_dir_all(&proj2).unwrap();
        fs::create_dir_all(&standalone).unwrap();
        fs::create_dir_all(&proj3).unwrap();

        // Mark first three as exemplars
        fs::write(proj1.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(proj2.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(standalone.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(proj3.join("Cast.toml"), "").unwrap(); // Not an exemplar

        // Find all exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 3);

        // Verify all three exemplars are found (sorted alphabetically by full path)
        // The sorting is by full path: standalone_proj, workspace1/proj1, workspace2/nested/proj2
        assert!(exemplars[0].ends_with("standalone_proj"));
        assert!(exemplars[1].ends_with("proj1"));
        assert!(exemplars[2].ends_with("proj2"));
    }

    #[test]
    fn test_find_exemplars_skips_build_directories() {
        let tmp_dir = TempDir::new("test_skip_dirs").unwrap();

        // Create exemplars in normal directories
        let proj1 = tmp_dir.path().join("workspace/proj1");
        fs::create_dir_all(&proj1).unwrap();
        fs::write(proj1.join("Cast.toml"), "exemplar = true").unwrap();

        // Create exemplars in directories that should be skipped
        let target_proj = tmp_dir.path().join("workspace/target/proj2");
        let node_proj = tmp_dir.path().join("workspace/node_modules/proj3");
        let git_proj = tmp_dir.path().join("workspace/.git/proj4");

        fs::create_dir_all(&target_proj).unwrap();
        fs::create_dir_all(&node_proj).unwrap();
        fs::create_dir_all(&git_proj).unwrap();

        fs::write(target_proj.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(node_proj.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(git_proj.join("Cast.toml"), "exemplar = true").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        // Should only find proj1, not the ones in target/node_modules/.git
        assert_eq!(exemplars.len(), 1);
        assert!(exemplars[0].ends_with("proj1"));
    }

    #[test]
    fn test_find_exemplars_works_with_cargo_toml_metadata() {
        let tmp_dir = TempDir::new("test_cargo_metadata").unwrap();

        // Create a project with exemplar flag in Cargo.toml metadata
        let proj = tmp_dir.path().join("proj_with_metadata");
        fs::create_dir_all(&proj).unwrap();

        fs::write(
            proj.join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[package.metadata.cast]\nexemplar = true",
        )
        .unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 1);
        assert!(exemplars[0].ends_with("proj_with_metadata"));
    }

    #[test]
    fn test_find_exemplars_in_deeply_nested_structure() {
        let tmp_dir = TempDir::new("test_deep_nest").unwrap();

        // Create a deeply nested exemplar
        let deep_proj = tmp_dir.path().join("a/b/c/d/e/deep_exemplar");
        fs::create_dir_all(&deep_proj).unwrap();
        fs::write(deep_proj.join("Cast.toml"), "exemplar = true").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 1);
        assert!(exemplars[0].ends_with("deep_exemplar"));
    }

    #[test]
    fn test_find_exemplars_handles_mixed_cast_and_cargo_toml() {
        let tmp_dir = TempDir::new("test_mixed").unwrap();

        // Create three projects:
        // 1. Only Cast.toml with exemplar = true
        // 2. Only Cargo.toml with cast metadata exemplar = true
        // 3. Both Cast.toml and Cargo.toml, only Cast.toml has exemplar = true

        let proj1 = tmp_dir.path().join("proj1_cast_only");
        let proj2 = tmp_dir.path().join("proj2_cargo_only");
        let proj3 = tmp_dir.path().join("proj3_both");

        fs::create_dir_all(&proj1).unwrap();
        fs::create_dir_all(&proj2).unwrap();
        fs::create_dir_all(&proj3).unwrap();

        fs::write(proj1.join("Cast.toml"), "exemplar = true").unwrap();

        fs::write(
            proj2.join("Cargo.toml"),
            "[package]\nname = \"test\"\n\n[package.metadata.cast]\nexemplar = true",
        )
        .unwrap();

        fs::write(proj3.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(proj3.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 3);
    }
}
