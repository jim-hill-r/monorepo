//! ISSUES.md file modification module
//!
//! This module provides functionality for parsing and modifying ISSUES.md files
//! in the monorepo. It can:
//! - Parse existing ISSUES.md files
//! - Add new TODO entries without creating duplicates
//! - Maintain proper formatting and structure
//! - Create ISSUES.md files if they don't exist

use std::fs;
use std::io;
use std::path::Path;

/// Represents the structure of an ISSUES.md file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuesFile {
    /// Path to the ISSUES.md file
    pub path: String,
    /// Content of the Priority Issues section
    pub priority_issues: Vec<String>,
    /// Content of the Backlog section
    pub backlog: Vec<String>,
    /// Raw content of other sections (preserved as-is)
    pub other_content: String,
}

impl IssuesFile {
    /// Create a new empty IssuesFile with default sections
    pub fn new(path: String) -> Self {
        Self {
            path,
            priority_issues: Vec::new(),
            backlog: Vec::new(),
            other_content: String::new(),
        }
    }

    /// Parse an ISSUES.md file from a path
    pub fn parse(path: &Path) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();

        let mut issues_file = Self::new(path_str);
        let mut current_section = Section::None;
        let mut priority_lines = Vec::new();
        let mut backlog_lines = Vec::new();
        let mut other_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section headers
            if trimmed == "# Priority Issues" {
                current_section = Section::Priority;
                continue;
            } else if trimmed == "# Backlog" {
                current_section = Section::Backlog;
                continue;
            } else if trimmed.starts_with('#') {
                // Any other section goes into other_content
                current_section = Section::Other;
                other_lines.push(line.to_string());
                continue;
            }

            // Add content to appropriate section
            match current_section {
                Section::Priority => {
                    if !trimmed.is_empty() {
                        priority_lines.push(line.to_string());
                    }
                }
                Section::Backlog => {
                    if !trimmed.is_empty() {
                        backlog_lines.push(line.to_string());
                    }
                }
                Section::Other => {
                    other_lines.push(line.to_string());
                }
                Section::None => {
                    // Content before any section header
                    if !trimmed.is_empty() {
                        other_lines.push(line.to_string());
                    }
                }
            }
        }

        issues_file.priority_issues = priority_lines;
        issues_file.backlog = backlog_lines;
        issues_file.other_content = other_lines.join("\n");

        Ok(issues_file)
    }

    /// Add a TODO item to the Priority Issues section
    /// Returns true if the TODO was added, false if it already exists
    pub fn add_priority_todo(&mut self, todo: String) -> bool {
        if self.has_duplicate(&self.priority_issues, &todo) {
            return false;
        }
        self.priority_issues.push(todo);
        true
    }

    /// Add a TODO item to the Backlog section
    /// Returns true if the TODO was added, false if it already exists
    pub fn add_backlog_todo(&mut self, todo: String) -> bool {
        if self.has_duplicate(&self.backlog, &todo) {
            return false;
        }
        self.backlog.push(todo);
        true
    }

    /// Check if a TODO already exists in the given list (case-insensitive)
    fn has_duplicate(&self, list: &[String], todo: &str) -> bool {
        let normalized_todo = todo.trim().to_lowercase();
        list.iter()
            .any(|existing| existing.trim().to_lowercase() == normalized_todo)
    }

    /// Write the IssuesFile to disk, creating the file if it doesn't exist
    pub fn write(&self) -> Result<(), io::Error> {
        let mut content = String::new();

        // Priority Issues section
        content.push_str("# Priority Issues\n");
        if self.priority_issues.is_empty() {
            content.push('\n');
        } else {
            for todo in &self.priority_issues {
                content.push('\n');
                content.push_str(todo);
                content.push('\n');
            }
        }

        // Backlog section
        content.push_str("# Backlog\n");
        if self.backlog.is_empty() {
            content.push('\n');
        } else {
            for todo in &self.backlog {
                content.push('\n');
                content.push_str(todo);
                content.push('\n');
            }
        }

        // Other sections (if any)
        if !self.other_content.is_empty() {
            content.push_str(&self.other_content);
            if !self.other_content.ends_with('\n') {
                content.push('\n');
            }
        }

        fs::write(&self.path, content)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Priority,
    Backlog,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_new_issues_file() {
        let issues = IssuesFile::new("/tmp/test/ISSUES.md".to_string());
        assert_eq!(issues.path, "/tmp/test/ISSUES.md");
        assert!(issues.priority_issues.is_empty());
        assert!(issues.backlog.is_empty());
        assert!(issues.other_content.is_empty());
    }

    #[test]
    fn test_add_priority_todo() {
        let mut issues = IssuesFile::new("/tmp/test/ISSUES.md".to_string());

        // Add first TODO
        assert!(issues.add_priority_todo("TODO: Fix bug".to_string()));
        assert_eq!(issues.priority_issues.len(), 1);

        // Try to add duplicate (should fail)
        assert!(!issues.add_priority_todo("TODO: Fix bug".to_string()));
        assert_eq!(issues.priority_issues.len(), 1);

        // Case-insensitive duplicate check
        assert!(!issues.add_priority_todo("todo: fix bug".to_string()));
        assert_eq!(issues.priority_issues.len(), 1);

        // Add different TODO
        assert!(issues.add_priority_todo("TODO: Add feature".to_string()));
        assert_eq!(issues.priority_issues.len(), 2);
    }

    #[test]
    fn test_add_backlog_todo() {
        let mut issues = IssuesFile::new("/tmp/test/ISSUES.md".to_string());

        assert!(issues.add_backlog_todo("TODO: Future work".to_string()));
        assert_eq!(issues.backlog.len(), 1);

        assert!(!issues.add_backlog_todo("TODO: Future work".to_string()));
        assert_eq!(issues.backlog.len(), 1);
    }

    #[test]
    fn test_write_empty_issues_file() {
        let temp_dir = std::env::temp_dir().join("test_write_empty");
        fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let issues = IssuesFile::new(issues_path.to_string_lossy().to_string());

        issues.write().expect("Failed to write file");

        let content = fs::read_to_string(&issues_path).expect("Failed to read file");
        assert!(content.contains("# Priority Issues"));
        assert!(content.contains("# Backlog"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_write_issues_with_todos() {
        let temp_dir = std::env::temp_dir().join("test_write_with_todos");
        fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let mut issues = IssuesFile::new(issues_path.to_string_lossy().to_string());

        issues.add_priority_todo("TODO: Implement feature X".to_string());
        issues.add_priority_todo("TODO: Fix bug Y".to_string());
        issues.add_backlog_todo("TODO: Consider optimization Z".to_string());

        issues.write().expect("Failed to write file");

        let content = fs::read_to_string(&issues_path).expect("Failed to read file");
        assert!(content.contains("TODO: Implement feature X"));
        assert!(content.contains("TODO: Fix bug Y"));
        assert!(content.contains("TODO: Consider optimization Z"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_parse_existing_issues_file() {
        let temp_dir = std::env::temp_dir().join("test_parse_existing");
        fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let content = r#"# Priority Issues

TODO: First priority item
TODO: Second priority item

# Backlog

TODO: First backlog item
"#;

        fs::write(&issues_path, content).expect("Failed to write test file");

        let issues = IssuesFile::parse(&issues_path).expect("Failed to parse file");
        assert_eq!(issues.priority_issues.len(), 2);
        assert_eq!(issues.backlog.len(), 1);

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_parse_and_add_todo() {
        let temp_dir = std::env::temp_dir().join("test_parse_and_add");
        fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let content = r#"# Priority Issues

TODO: Existing item

# Backlog
"#;

        fs::write(&issues_path, content).expect("Failed to write test file");

        let mut issues = IssuesFile::parse(&issues_path).expect("Failed to parse file");

        // Try to add duplicate
        assert!(!issues.add_priority_todo("TODO: Existing item".to_string()));

        // Add new item
        assert!(issues.add_priority_todo("TODO: New item".to_string()));

        issues.write().expect("Failed to write file");

        let new_content = fs::read_to_string(&issues_path).expect("Failed to read file");
        assert!(new_content.contains("TODO: Existing item"));
        assert!(new_content.contains("TODO: New item"));

        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_parse_with_other_sections() {
        let temp_dir = std::env::temp_dir().join("test_parse_other_sections");
        fs::create_dir_all(&temp_dir).ok();

        let issues_path = temp_dir.join("ISSUES.md");
        let content = r#"# Priority Issues

TODO: Priority item

# Backlog

TODO: Backlog item

# Priority Projects
- cast
- cookbook
"#;

        fs::write(&issues_path, content).expect("Failed to write test file");

        let issues = IssuesFile::parse(&issues_path).expect("Failed to parse file");
        assert_eq!(issues.priority_issues.len(), 1);
        assert_eq!(issues.backlog.len(), 1);
        assert!(issues.other_content.contains("# Priority Projects"));
        assert!(issues.other_content.contains("- cast"));

        fs::remove_dir_all(&temp_dir).ok();
    }
}
