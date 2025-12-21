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
fn find_exemplar_projects(working_directory: &Path) -> Result<Vec<PathBuf>, NewProjectError> {
    let mut exemplar_projects = Vec::new();

    // Search in the projects directory for exemplar projects
    let projects_dir = working_directory.join("projects");
    if projects_dir.exists() {
        find_exemplars_in_directory(&projects_dir, &mut exemplar_projects)?;
    }

    // Sort to ensure consistent ordering (alphabetical by path name)
    exemplar_projects.sort();

    Ok(exemplar_projects)
}

/// Recursively find exemplar projects in a directory
fn find_exemplars_in_directory(dir: &Path, exemplars: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let cast_toml = path.join("Cast.toml");
            if cast_toml.exists() {
                // Try to load the Cast.toml and check if it's an exemplar
                if let Ok(config) = CastConfig::load(&cast_toml) {
                    if config.exemplar == Some(true) {
                        exemplars.push(path);
                    }
                }
            }
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

/// Remove the exemplar flag from a project's Cast.toml file
fn remove_exemplar_flag(project_dir: &Path) -> Result<(), NewProjectError> {
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

/// Find the project directory containing a Cast.toml for a given file path
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

    // Walk up the directory tree looking for Cast.toml
    while current.starts_with(repo_root) {
        let cast_toml = current.join("Cast.toml");
        if cast_toml.exists() {
            // Return relative path from repo_root
            let relative = current.strip_prefix(repo_root).ok()?;
            // If relative path is empty, return "."
            if relative.as_os_str().is_empty() {
                return Some(PathBuf::from("."));
            }
            return Some(relative.to_path_buf());
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
        fs::create_dir_all(&projects_base.join("src")).unwrap();
        fs::create_dir_all(&projects_base.join("docs")).unwrap();
        fs::write(projects_base.join("README.md"), "# Base README").unwrap();
        fs::write(projects_base.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(projects_base.join("Cast.toml"), "exemplar = true").unwrap();

        // Create library exemplar with Cargo.toml
        fs::create_dir_all(&projects_library.join("src")).unwrap();
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
        fs::create_dir_all(&projects_base.join("src")).unwrap();
        fs::create_dir_all(&projects_library.join("src")).unwrap();
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
        fs::create_dir_all(&projects_base.join("src")).unwrap();
        fs::create_dir_all(&projects_base.join("docs")).unwrap();

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
        fs::create_dir_all(&src.join("a/b/c")).unwrap();
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
        let config = CastConfig::load(&project_path.join("Cast.toml")).unwrap();
        assert_eq!(config.exemplar, None);
        // Other flags should be preserved
        assert_eq!(config.proof_of_concept, Some(false));
    }

    #[test]
    fn test_find_project_dir_finds_cast_toml() {
        let tmp_dir = TempDir::new("test_find_project").unwrap();

        // Create a project with Cast.toml
        let project_dir = tmp_dir.path().join("projects/my_project");
        fs::create_dir_all(&project_dir.join("src")).unwrap();
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
}
