use std::path::{Path, PathBuf};
use std::{fs, io};

use thiserror::Error;

use crate::config::CastConfig;

#[derive(Error, Debug)]
pub enum NewProjectError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("No exemplar projects found")]
    NoExemplarFound,
    #[error("Config error: {0}")]
    ConfigError(#[from] crate::config::ConfigError),
}

pub fn new(working_directory: impl AsRef<Path>, name: &str) -> Result<(), NewProjectError> {
    let working_directory = working_directory.as_ref();
    let destination = working_directory.join(name);

    // Find an exemplar project to use as a template
    let exemplar_path = find_exemplar_project(working_directory)?;
    
    // Copy the exemplar project to the new project directory
    copy_dir_all(&exemplar_path, &destination)?;
    
    // Delete unnecessary .gitignore files (empty placeholder files)
    delete_empty_gitignores(&destination)?;
    
    Ok(())
}

/// Find the first exemplar project in the projects directory
fn find_exemplar_project(working_directory: &Path) -> Result<PathBuf, NewProjectError> {
    let projects_dir = working_directory.join("projects");
    
    if !projects_dir.exists() {
        return Err(NewProjectError::NoExemplarFound);
    }
    
    // Read all directories in projects/
    for entry in fs::read_dir(projects_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            let cast_toml = path.join("Cast.toml");
            if cast_toml.exists() {
                // Try to load the config
                if let Ok(config) = CastConfig::load(&cast_toml) {
                    if config.exemplar == Some(true) {
                        return Ok(path);
                    }
                }
            }
        }
    }
    
    Err(NewProjectError::NoExemplarFound)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new_creates_project_from_exemplar() {
        let tmp_dir = TempDir::new("test_new_project").unwrap();
        
        // Create mock projects directory with an exemplar project
        let projects_dir = tmp_dir.path().join("projects");
        let exemplar_dir = projects_dir.join("library_exemplar");
        
        // Create exemplar project with Cast.toml having exemplar = true
        fs::create_dir_all(&exemplar_dir.join("src")).unwrap();
        fs::create_dir_all(&exemplar_dir.join("docs")).unwrap();
        fs::write(exemplar_dir.join("README.md"), "# Exemplar README").unwrap();
        fs::write(exemplar_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(exemplar_dir.join("src/lib.rs"), "// lib").unwrap();
        fs::write(exemplar_dir.join("Cast.toml"), "exemplar = true").unwrap();
        
        // Call the new function
        let result = new(tmp_dir.path(), "my_project");
        assert!(result.is_ok());
        
        // Verify the project was created
        let project_path = tmp_dir.path().join("my_project");
        assert!(project_path.exists());
        assert!(project_path.join("README.md").exists());
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src").exists());
        
        // Verify content from exemplar
        let readme_content = fs::read_to_string(project_path.join("README.md")).unwrap();
        assert_eq!(readme_content, "# Exemplar README");
        
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
        assert_eq!(cargo_content, "[package]\nname = \"test\"");
    }

    #[test]
    fn test_new_returns_error_when_no_exemplar_found() {
        let tmp_dir = TempDir::new("test_error").unwrap();
        
        // Create projects directory but no exemplar projects
        let projects_dir = tmp_dir.path().join("projects");
        fs::create_dir_all(&projects_dir).unwrap();
        
        let regular_project = projects_dir.join("regular_project");
        fs::create_dir_all(&regular_project).unwrap();
        fs::write(regular_project.join("Cast.toml"), "# No exemplar flag").unwrap();
        
        let result = new(tmp_dir.path(), "test_project");
        
        // Should return an error since no exemplar projects exist
        assert!(result.is_err());
        match result {
            Err(NewProjectError::NoExemplarFound) => {},
            _ => panic!("Expected NoExemplarFound error"),
        }
    }

    #[test]
    fn test_new_deletes_empty_gitignores() {
        let tmp_dir = TempDir::new("test_gitignore").unwrap();
        
        // Create an exemplar project with empty .gitignore files
        let projects_dir = tmp_dir.path().join("projects");
        let exemplar_dir = projects_dir.join("library_exemplar");
        fs::create_dir_all(&exemplar_dir.join("src")).unwrap();
        fs::create_dir_all(&exemplar_dir.join("docs")).unwrap();
        
        // Create empty .gitignore files
        fs::write(exemplar_dir.join("src/.gitignore"), "").unwrap();
        fs::write(exemplar_dir.join("docs/.gitignore"), "").unwrap();
        
        // Create a non-empty .gitignore
        fs::write(exemplar_dir.join(".gitignore"), "target/\n").unwrap();
        
        fs::write(exemplar_dir.join("Cast.toml"), "exemplar = true").unwrap();
        
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
    fn test_find_exemplar_project_returns_first_exemplar() {
        let tmp_dir = TempDir::new("test_find_exemplar").unwrap();
        
        let projects_dir = tmp_dir.path().join("projects");
        
        // Create multiple exemplar projects
        let exemplar1 = projects_dir.join("aaa_exemplar");
        let exemplar2 = projects_dir.join("zzz_exemplar");
        
        fs::create_dir_all(&exemplar1).unwrap();
        fs::write(exemplar1.join("Cast.toml"), "exemplar = true").unwrap();
        
        fs::create_dir_all(&exemplar2).unwrap();
        fs::write(exemplar2.join("Cast.toml"), "exemplar = true").unwrap();
        
        // Find exemplar
        let result = find_exemplar_project(tmp_dir.path());
        assert!(result.is_ok());
        
        // The result should be one of the exemplars (order may vary)
        let exemplar_path = result.unwrap();
        assert!(exemplar_path.ends_with("aaa_exemplar") || exemplar_path.ends_with("zzz_exemplar"));
    }

    #[test]
    fn test_find_exemplar_project_ignores_non_exemplar() {
        let tmp_dir = TempDir::new("test_find_exemplar").unwrap();
        
        let projects_dir = tmp_dir.path().join("projects");
        
        // Create a non-exemplar project
        let regular_project = projects_dir.join("regular_project");
        fs::create_dir_all(&regular_project).unwrap();
        fs::write(regular_project.join("Cast.toml"), "exemplar = false").unwrap();
        
        // Create an exemplar project
        let exemplar = projects_dir.join("exemplar_project");
        fs::create_dir_all(&exemplar).unwrap();
        fs::write(exemplar.join("Cast.toml"), "exemplar = true").unwrap();
        
        // Find exemplar - should find the exemplar, not the regular project
        let result = find_exemplar_project(tmp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("exemplar_project"));
    }

    #[test]
    fn test_find_exemplar_project_returns_error_when_projects_dir_missing() {
        let tmp_dir = TempDir::new("test_find_exemplar").unwrap();
        
        // Don't create projects directory
        let result = find_exemplar_project(tmp_dir.path());
        
        assert!(result.is_err());
        match result {
            Err(NewProjectError::NoExemplarFound) => {},
            _ => panic!("Expected NoExemplarFound error"),
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
        assert_eq!(fs::read_to_string(dst.join("a/file1.txt")).unwrap(), "content1");
        assert_eq!(fs::read_to_string(dst.join("a/b/file2.txt")).unwrap(), "content2");
        assert_eq!(fs::read_to_string(dst.join("a/b/c/file3.txt")).unwrap(), "content3");
    }
}
