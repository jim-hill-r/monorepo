use std::path::{Path, PathBuf};
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

    // Search in the templates directory for backwards compatibility
    let templates_dir = working_directory.join("templates");
    if templates_dir.exists() {
        find_exemplars_in_directory(&templates_dir, &mut exemplar_projects)?;
    }

    // Search in the projects directory for new exemplar projects
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
                if let Ok(config) = CastConfig::load(&cast_toml)
                    && config.exemplar == Some(true)
                {
                    exemplars.push(path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new_creates_project_from_templates() {
        let tmp_dir = TempDir::new("test_new_project").unwrap();

        // Create mock templates directory structure
        let templates_base = tmp_dir.path().join("templates/base");
        let templates_library = tmp_dir.path().join("templates/library");

        // Create base template with some files and directories
        fs::create_dir_all(&templates_base.join("src")).unwrap();
        fs::create_dir_all(&templates_base.join("docs")).unwrap();
        fs::write(templates_base.join("README.md"), "# Base README").unwrap();
        fs::write(templates_base.join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(templates_base.join("Cast.toml"), "exemplar = true").unwrap();

        // Create library template with Cargo.toml
        fs::create_dir_all(&templates_library.join("src")).unwrap();
        fs::write(
            templates_library.join("Cargo.toml"),
            "[package]\nname = \"test\"",
        )
        .unwrap();
        fs::write(templates_library.join("src/lib.rs"), "// lib").unwrap();
        fs::write(templates_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Call the new function
        let result = new(tmp_dir.path(), "my_project");
        assert!(result.is_ok());

        // Verify the project was created
        let project_path = tmp_dir.path().join("my_project");
        assert!(project_path.exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src").exists());

        // Verify content from base template
        let readme_content = fs::read_to_string(project_path.join("README.md")).unwrap();
        assert_eq!(readme_content, "# Base README");

        // Verify content from library template (should exist)
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
        assert_eq!(cargo_content, "[package]\nname = \"test\"");
    }

    #[test]
    fn test_new_overwrites_files_from_library_template() {
        let tmp_dir = TempDir::new("test_overwrite").unwrap();

        // Create templates
        let templates_base = tmp_dir.path().join("templates/base");
        let templates_library = tmp_dir.path().join("templates/library");

        // Create the same file in both templates
        fs::create_dir_all(&templates_base.join("src")).unwrap();
        fs::create_dir_all(&templates_library.join("src")).unwrap();
        fs::write(templates_base.join("src/lib.rs"), "// base version").unwrap();
        fs::write(templates_base.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(templates_library.join("src/lib.rs"), "// library version").unwrap();
        fs::write(templates_library.join("Cast.toml"), "exemplar = true").unwrap();

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

        // Create templates with empty .gitignore files
        let templates_base = tmp_dir.path().join("templates/base");
        fs::create_dir_all(&templates_base.join("src")).unwrap();
        fs::create_dir_all(&templates_base.join("docs")).unwrap();

        // Create empty .gitignore files
        fs::write(templates_base.join("src/.gitignore"), "").unwrap();
        fs::write(templates_base.join("docs/.gitignore"), "").unwrap();

        // Create a non-empty .gitignore
        fs::write(templates_base.join(".gitignore"), "target/\n").unwrap();

        // Create Cast.toml to mark as exemplar
        fs::write(templates_base.join("Cast.toml"), "exemplar = true").unwrap();

        let templates_library = tmp_dir.path().join("templates/library");
        fs::create_dir_all(&templates_library).unwrap();
        fs::write(templates_library.join("Cast.toml"), "exemplar = true").unwrap();

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
    fn test_new_returns_error_when_templates_missing() {
        let tmp_dir = TempDir::new("test_error").unwrap();

        // Don't create templates directories or exemplar projects
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
    fn test_find_exemplar_projects_in_templates_directory() {
        let tmp_dir = TempDir::new("test_exemplar").unwrap();

        // Create templates directory with exemplar projects
        let templates_base = tmp_dir.path().join("templates/base");
        let templates_library = tmp_dir.path().join("templates/library");
        fs::create_dir_all(&templates_base).unwrap();
        fs::create_dir_all(&templates_library).unwrap();

        fs::write(templates_base.join("Cast.toml"), "exemplar = true").unwrap();
        fs::write(templates_library.join("Cast.toml"), "exemplar = true").unwrap();

        // Find exemplar projects
        let result = find_exemplar_projects(tmp_dir.path());
        assert!(result.is_ok());

        let exemplars = result.unwrap();
        assert_eq!(exemplars.len(), 2);

        // Verify both templates are found (sorted order)
        assert!(exemplars[0].ends_with("base"));
        assert!(exemplars[1].ends_with("library"));
    }

    #[test]
    fn test_find_exemplar_projects_in_projects_directory() {
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
    fn test_find_exemplar_projects_searches_both_templates_and_projects() {
        let tmp_dir = TempDir::new("test_both").unwrap();

        // Create both templates and projects with exemplars
        let templates_base = tmp_dir.path().join("templates/base");
        let projects_example = tmp_dir.path().join("projects/example");

        fs::create_dir_all(&templates_base).unwrap();
        fs::create_dir_all(&projects_example).unwrap();

        fs::write(templates_base.join("Cast.toml"), "exemplar = true").unwrap();
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

        // Create template with exemplar flag
        let templates_base = tmp_dir.path().join("templates/base");
        fs::create_dir_all(&templates_base).unwrap();
        fs::write(
            templates_base.join("Cast.toml"),
            "exemplar = true\nproof_of_concept = false",
        )
        .unwrap();
        fs::write(templates_base.join("README.md"), "# Test").unwrap();

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
}
