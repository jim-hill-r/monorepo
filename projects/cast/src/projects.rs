use std::path::Path;
use std::{fs, io};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewProjectError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub fn new(working_directory: impl AsRef<Path>, name: &str) -> Result<(), NewProjectError> {
    let working_directory = working_directory.as_ref();
    let templates_base = working_directory.join("templates/base");
    let templates_library = working_directory.join("templates/library");
    let destination = working_directory.join(name);

    // Copy templates/base to the new project directory
    copy_dir_all(&templates_base, &destination)?;
    
    // Copy templates/library (overwriting files from base)
    copy_dir_all(&templates_library, &destination)?;
    
    // Delete unnecessary .gitignore files (empty placeholder files)
    delete_empty_gitignores(&destination)?;
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
        
        // Create library template with Cargo.toml
        fs::create_dir_all(&templates_library.join("src")).unwrap();
        fs::write(templates_library.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(templates_library.join("src/lib.rs"), "// lib").unwrap();
        
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
        fs::write(templates_library.join("src/lib.rs"), "// library version").unwrap();
        
        // Call new
        let result = new(tmp_dir.path(), "test_project");
        assert!(result.is_ok());
        
        // Verify the library version overwrote the base version
        let lib_content = fs::read_to_string(tmp_dir.path().join("test_project/src/lib.rs")).unwrap();
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
        
        let templates_library = tmp_dir.path().join("templates/library");
        fs::create_dir_all(&templates_library).unwrap();
        
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
        
        // Don't create templates directories
        let result = new(tmp_dir.path(), "test_project");
        
        // Should return an error since templates don't exist
        assert!(result.is_err());
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
