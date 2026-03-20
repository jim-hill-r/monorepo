//! Audit module for checking standards compliance
//!
//! This module provides functionality to audit projects against defined standards.
//! Each audit function examines a specific aspect of project configuration and
//! returns violations that need to be addressed.

use crate::discovery::{Project, ProjectType};
use crate::{Severity, StandardType};

/// Represents a violation of a standard
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Type of standard that was violated
    pub standard_type: StandardType,
    /// Unique identifier for the violated standard
    pub standard_id: String,
    /// Project where the violation occurred
    pub project_name: String,
    /// Path to the project
    pub project_path: String,
    /// Severity of the violation
    pub severity: Severity,
    /// Human-readable description of the violation
    pub message: String,
}

impl Violation {
    /// Create a new violation
    pub fn new(
        standard_type: StandardType,
        standard_id: String,
        project_name: String,
        project_path: String,
        severity: Severity,
        message: String,
    ) -> Self {
        Self {
            standard_type,
            standard_id,
            project_name,
            project_path,
            severity,
            message,
        }
    }
}

/// Audit results containing all violations found
#[derive(Debug, Clone)]
pub struct AuditResult {
    /// List of all violations found
    pub violations: Vec<Violation>,
}

impl AuditResult {
    /// Create a new empty audit result
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
        }
    }

    /// Add a violation to the result
    pub fn add_violation(&mut self, violation: Violation) {
        self.violations.push(violation);
    }

    /// Check if there are any violations
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Count violations by severity
    pub fn count_by_severity(&self, severity: Severity) -> usize {
        self.violations
            .iter()
            .filter(|v| v.severity == severity)
            .count()
    }
}

impl Default for AuditResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Naming standards audit module
pub mod naming {
    use super::*;

    /// Check if a string is in snake_case format
    ///
    /// Snake case means:
    /// - All lowercase letters
    /// - Words separated by underscores
    /// - May contain numbers
    /// - No consecutive underscores
    /// - Does not start or end with underscore
    pub fn is_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        // Check for leading or trailing underscores
        if s.starts_with('_') || s.ends_with('_') {
            return false;
        }

        // Check for consecutive underscores
        if s.contains("__") {
            return false;
        }

        // All characters must be lowercase letters, digits, or underscores
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }

    /// Check if a string is in kebab-case format
    ///
    /// Kebab case means:
    /// - All lowercase letters
    /// - Words separated by hyphens
    /// - May contain numbers
    /// - No consecutive hyphens
    /// - Does not start or end with hyphen
    pub fn is_kebab_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        // Check for leading or trailing hyphens
        if s.starts_with('-') || s.ends_with('-') {
            return false;
        }

        // Check for consecutive hyphens
        if s.contains("--") {
            return false;
        }

        // All characters must be lowercase letters, digits, or hyphens
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    /// Audit projects for naming standards compliance
    ///
    /// Checks:
    /// - NAM-001: All projects MUST be snake_case
    /// - NAM-002: Directory name must match package name, or parent_dir + "_" + dir_name must match
    /// - NAM-004: PoC projects must begin with "poc_"
    pub fn audit_naming_standards(projects: &[Project]) -> AuditResult {
        let mut result = AuditResult::new();

        for project in projects {
            // NAM-001: Check if project name is snake_case
            if !is_snake_case(&project.name) {
                result.add_violation(Violation::new(
                    StandardType::Naming,
                    "NAM-001".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Error,
                    format!(
                        "Project name '{}' is not in snake_case format. All projects MUST be snake_case.",
                        project.name
                    ),
                ));
            }

            // NAM-002: Check if directory name matches package name.
            // Allows either an exact match OR parent_dir + "_" + dir_name (e.g., cast/core → cast_core).
            if let Some(dir_name) = project.path.file_name().and_then(|n| n.to_str()) {
                let parent_prefixed_name = project
                    .path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .map(|parent| format!("{}_{}", parent, dir_name));

                let is_valid = dir_name == project.name
                    || parent_prefixed_name.as_deref() == Some(project.name.as_str());

                if !is_valid {
                    result.add_violation(Violation::new(
                        StandardType::Naming,
                        "NAM-002".to_string(),
                        project.name.clone(),
                        project.path.display().to_string(),
                        Severity::Error,
                        format!(
                            "Directory name '{}' does not match package name '{}'. Directory name MUST match the package name, or the package name must equal the parent directory name joined with the directory name using an underscore.",
                            dir_name, project.name
                        ),
                    ));
                }
            }

            // NAM-004: Check if PoC projects begin with "poc_"
            // Only for Rust projects (assumption based on common practice)
            if project.project_type == ProjectType::Rust {
                // Heuristic: if the project name or path contains "poc" or "proof" but doesn't start with "poc_"
                let lower_name = project.name.to_lowercase();
                let path_str = project.path.display().to_string().to_lowercase();

                // If it looks like a PoC but doesn't follow the naming convention
                if (lower_name.contains("poc")
                    || lower_name.contains("proof")
                    || path_str.contains("/poc"))
                    && !project.name.starts_with("poc_")
                {
                    result.add_violation(Violation::new(
                        StandardType::Naming,
                        "NAM-004".to_string(),
                        project.name.clone(),
                        project.path.display().to_string(),
                        Severity::Error,
                        format!(
                            "Proof of Concept project '{}' does not begin with 'poc_'. All PoC projects must begin with poc (ie: poc_project_name).",
                            project.name
                        ),
                    ));
                }
            }
        }

        result
    }

    #[cfg(test)]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod tests {
        use super::*;
        use std::path::PathBuf;

        #[test]
        fn test_is_snake_case_valid() {
            assert!(is_snake_case("valid_name"));
            assert!(is_snake_case("another_valid_name"));
            assert!(is_snake_case("name_with_123"));
            assert!(is_snake_case("simple"));
            assert!(is_snake_case("a"));
        }

        #[test]
        fn test_is_snake_case_invalid() {
            assert!(!is_snake_case("InvalidName")); // PascalCase
            assert!(!is_snake_case("invalid-name")); // kebab-case
            assert!(!is_snake_case("_leading"));
            assert!(!is_snake_case("trailing_"));
            assert!(!is_snake_case("double__underscore"));
            assert!(!is_snake_case("")); // empty
            assert!(!is_snake_case("Has Space"));
            assert!(!is_snake_case("hasUpperCase"));
        }

        #[test]
        fn test_is_kebab_case_valid() {
            assert!(is_kebab_case("valid-name"));
            assert!(is_kebab_case("another-valid-name"));
            assert!(is_kebab_case("name-with-123"));
            assert!(is_kebab_case("simple"));
            assert!(is_kebab_case("a"));
        }

        #[test]
        fn test_is_kebab_case_invalid() {
            assert!(!is_kebab_case("InvalidName")); // PascalCase
            assert!(!is_kebab_case("invalid_name")); // snake_case
            assert!(!is_kebab_case("-leading"));
            assert!(!is_kebab_case("trailing-"));
            assert!(!is_kebab_case("double--hyphen"));
            assert!(!is_kebab_case("")); // empty
            assert!(!is_kebab_case("Has Space"));
            assert!(!is_kebab_case("hasUpperCase"));
        }

        #[test]
        fn test_audit_valid_snake_case_project() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/valid_project"),
                ProjectType::Rust,
                "valid_project".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(!result.has_violations());
        }

        #[test]
        fn test_audit_invalid_snake_case_project() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/InvalidProject"),
                ProjectType::Rust,
                "InvalidProject".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(result.has_violations());
            // Only NAM-001 violation since directory name matches package name (both wrong)
            assert_eq!(result.violations.len(), 1);

            let nam_001 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "NAM-001");
            assert!(nam_001.is_some());
            assert_eq!(nam_001.unwrap().severity, Severity::Error);
        }

        #[test]
        fn test_audit_directory_name_mismatch() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/wrong_dir"),
                ProjectType::Rust,
                "correct_name".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(result.has_violations());

            let violations: Vec<_> = result
                .violations
                .iter()
                .filter(|v| v.standard_id == "NAM-002")
                .collect();
            assert_eq!(violations.len(), 1);
            assert!(violations[0].message.contains("wrong_dir"));
            assert!(violations[0].message.contains("correct_name"));
        }

        #[test]
        fn test_audit_parent_prefixed_name_is_valid() {
            // cast/core with package name cast_core should be valid (parent + "_" + dir)
            let projects = vec![Project::new(
                PathBuf::from("/monorepo/cast/core"),
                ProjectType::Rust,
                "cast_core".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            let nam_002: Vec<_> = result
                .violations
                .iter()
                .filter(|v| v.standard_id == "NAM-002")
                .collect();
            assert!(
                nam_002.is_empty(),
                "cast/core with package cast_core should be valid"
            );
        }

        #[test]
        fn test_audit_parent_prefixed_name_mismatch_still_fails() {
            // cast/web with package name cast_core should still be invalid
            let projects = vec![Project::new(
                PathBuf::from("/monorepo/cast/web"),
                ProjectType::Rust,
                "cast_core".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            let nam_002: Vec<_> = result
                .violations
                .iter()
                .filter(|v| v.standard_id == "NAM-002")
                .collect();
            assert_eq!(
                nam_002.len(),
                1,
                "cast/web with package cast_core should be invalid"
            );
        }

        #[test]
        fn test_audit_poc_project_without_prefix() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/proof_of_concept"),
                ProjectType::Rust,
                "proof_of_concept".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(result.has_violations());

            let violations: Vec<_> = result
                .violations
                .iter()
                .filter(|v| v.standard_id == "NAM-004")
                .collect();
            assert_eq!(violations.len(), 1);
            assert!(violations[0].message.contains("poc_"));
        }

        #[test]
        fn test_audit_poc_project_with_correct_prefix() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/poc_experiment"),
                ProjectType::Rust,
                "poc_experiment".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(!result.has_violations());
        }

        #[test]
        fn test_audit_multiple_projects_with_mixed_violations() {
            let projects = vec![
                Project::new(
                    PathBuf::from("/repo/valid_project"),
                    ProjectType::Rust,
                    "valid_project".to_string(),
                ),
                Project::new(
                    PathBuf::from("/repo/InvalidName"),
                    ProjectType::Rust,
                    "InvalidName".to_string(),
                ),
                Project::new(
                    PathBuf::from("/repo/wrong_dir"),
                    ProjectType::Rust,
                    "correct_name".to_string(),
                ),
            ];

            let result = audit_naming_standards(&projects);
            assert!(result.has_violations());
            assert!(result.violations.len() >= 2); // At least 2 violations
        }

        #[test]
        fn test_audit_typescript_project() {
            let projects = vec![Project::new(
                PathBuf::from("/repo/ts_project"),
                ProjectType::TypeScript,
                "ts_project".to_string(),
            )];

            let result = audit_naming_standards(&projects);
            assert!(!result.has_violations());
        }

        #[test]
        fn test_count_by_severity() {
            let mut result = AuditResult::new();

            result.add_violation(Violation::new(
                StandardType::Naming,
                "NAM-001".to_string(),
                "test".to_string(),
                "/test".to_string(),
                Severity::Error,
                "Error violation".to_string(),
            ));

            result.add_violation(Violation::new(
                StandardType::Naming,
                "NAM-002".to_string(),
                "test".to_string(),
                "/test".to_string(),
                Severity::Warning,
                "Warning violation".to_string(),
            ));

            assert_eq!(result.count_by_severity(Severity::Error), 1);
            assert_eq!(result.count_by_severity(Severity::Warning), 1);
            assert_eq!(result.count_by_severity(Severity::Info), 0);
        }
    }
}

/// Configuration standards audit module
pub mod configuration {
    use super::*;

    /// Audit projects for configuration standards compliance
    ///
    /// Checks:
    /// - CFG-001: All projects MUST include a royalty.toml
    pub fn audit_configuration_standards(projects: &[Project]) -> AuditResult {
        let mut result = AuditResult::new();

        for project in projects {
            // CFG-001: Check for royalty.toml presence
            let royalty_toml = project.path.join("royalty.toml");
            if !royalty_toml.exists() {
                result.add_violation(Violation::new(
                    StandardType::Configuration,
                    "CFG-001".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Error,
                    format!(
                        "Project '{}' is missing royalty.toml. All projects MUST include a royalty.toml for use with the royalty project.",
                        project.name
                    ),
                ));
            } else {
                // Validate that the file can be read (basic format check)
                if let Err(e) = std::fs::read_to_string(&royalty_toml) {
                    result.add_violation(Violation::new(
                        StandardType::Configuration,
                        "CFG-001".to_string(),
                        project.name.clone(),
                        project.path.display().to_string(),
                        Severity::Error,
                        format!(
                            "Project '{}' has royalty.toml but it cannot be read: {}",
                            project.name, e
                        ),
                    ));
                }
            }
        }

        result
    }

    #[cfg(test)]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod tests {
        use super::*;

        #[test]
        fn test_audit_project_with_royalty_toml() {
            // Create a temporary project with royalty.toml
            let temp_dir = std::env::temp_dir().join("test_config_audit_with_royalty");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create royalty.toml
            let royalty_toml = temp_dir.join("royalty.toml");
            std::fs::write(&royalty_toml, "# Sample royalty.toml")
                .expect("Failed to write royalty.toml");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_configuration_standards(&projects);

            // Should have no violations
            assert!(!result.has_violations());

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_without_royalty_toml() {
            let temp_dir = std::env::temp_dir().join("test_config_audit_without_royalty");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_configuration_standards(&projects);

            // Should have CFG-001 violation
            assert!(result.has_violations());
            assert_eq!(result.violations.len(), 1);

            let violation = &result.violations[0];
            assert_eq!(violation.standard_id, "CFG-001");
            assert_eq!(violation.standard_type, StandardType::Configuration);
            assert_eq!(violation.severity, Severity::Error);
            assert!(violation.message.contains("royalty.toml"));
            assert!(violation.message.contains("test_project"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_with_unreadable_royalty_toml() {
            let temp_dir = std::env::temp_dir().join("test_config_audit_unreadable");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create royalty.toml
            let royalty_toml = temp_dir.join("royalty.toml");
            std::fs::write(&royalty_toml, "# Sample").expect("Failed to write royalty.toml");

            // Make it unreadable on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&royalty_toml)
                    .expect("Failed to get metadata")
                    .permissions();
                perms.set_mode(0o000); // Remove all permissions
                std::fs::set_permissions(&royalty_toml, perms).expect("Failed to set permissions");
            }

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_configuration_standards(&projects);

            // On Unix, should have a violation for unreadable file
            // On Windows, this test may not work as expected due to different permission model
            #[cfg(unix)]
            {
                assert!(result.has_violations());
                assert_eq!(result.violations.len(), 1);
                let violation = &result.violations[0];
                assert!(violation.message.contains("cannot be read"));
            }

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_multiple_projects_mixed() {
            let temp_dir_root = std::env::temp_dir().join("test_config_audit_mixed");
            std::fs::remove_dir_all(&temp_dir_root).ok();
            std::fs::create_dir_all(&temp_dir_root).expect("Failed to create test directory");

            // Project 1: has royalty.toml
            let proj1_dir = temp_dir_root.join("project1");
            std::fs::create_dir_all(&proj1_dir).expect("Failed to create project1");
            std::fs::write(proj1_dir.join("royalty.toml"), "# Project 1")
                .expect("Failed to write royalty.toml");

            // Project 2: missing royalty.toml
            let proj2_dir = temp_dir_root.join("project2");
            std::fs::create_dir_all(&proj2_dir).expect("Failed to create project2");

            let projects = vec![
                Project::new(proj1_dir.clone(), ProjectType::Rust, "project1".to_string()),
                Project::new(proj2_dir.clone(), ProjectType::Rust, "project2".to_string()),
            ];

            let result = audit_configuration_standards(&projects);

            // Should have 1 violation (project2)
            assert!(result.has_violations());
            assert_eq!(result.violations.len(), 1);
            assert_eq!(result.violations[0].project_name, "project2");

            std::fs::remove_dir_all(&temp_dir_root).ok();
        }

        #[test]
        fn test_audit_typescript_project() {
            let temp_dir = std::env::temp_dir().join("test_config_audit_typescript");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // TypeScript projects also need royalty.toml
            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::TypeScript,
                "ts_project".to_string(),
            )];

            let result = audit_configuration_standards(&projects);

            // Should have CFG-001 violation
            assert!(result.has_violations());
            assert_eq!(result.violations.len(), 1);
            assert_eq!(result.violations[0].project_name, "ts_project");

            std::fs::remove_dir_all(&temp_dir).ok();
        }
    }
}

/// Documentation standards audit module
pub mod documentation {
    use super::*;

    /// Helper function to check if a markdown header with specific text exists
    ///
    /// Searches for a markdown heading (# Header, ## Header, etc.) with the given text (case-insensitive)
    fn has_markdown_header(content: &str, expected_text: &str) -> bool {
        let expected_lower = expected_text.to_lowercase();

        content.lines().any(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let heading_text = trimmed.trim_start_matches('#').trim().to_lowercase();
                heading_text == expected_lower
            } else {
                false
            }
        })
    }

    /// Check if a section header exists in README content
    ///
    /// Looks for a header like "# {project_name}" at the start of a line
    fn has_project_name_section(content: &str, project_name: &str) -> bool {
        has_markdown_header(content, project_name)
    }

    /// Check if a section header exists in CONTRIBUTING content
    ///
    /// Looks for a "Getting Started" section (case-insensitive)
    fn has_getting_started_section(content: &str) -> bool {
        has_markdown_header(content, "getting started")
    }

    /// Audit projects for documentation standards compliance
    ///
    /// Checks:
    /// - DOC-001: All projects MUST include a README.md
    /// - DOC-002: README.md should include a project name section with description
    /// - DOC-003: All projects MUST include a CONTRIBUTING.md
    /// - DOC-004: CONTRIBUTING.md should include a Getting Started section
    pub fn audit_documentation_standards(projects: &[Project]) -> AuditResult {
        let mut result = AuditResult::new();

        for project in projects {
            // DOC-001: Check for README.md presence
            let readme_path = project.path.join("README.md");
            if !readme_path.exists() {
                result.add_violation(Violation::new(
                    StandardType::Documentation,
                    "DOC-001".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Error,
                    format!(
                        "Project '{}' is missing README.md. All projects MUST include a README.md.",
                        project.name
                    ),
                ));
            } else {
                // DOC-002: Validate README.md sections
                if let Ok(readme_content) = std::fs::read_to_string(&readme_path) {
                    if !has_project_name_section(&readme_content, &project.name) {
                        result.add_violation(Violation::new(
                            StandardType::Documentation,
                            "DOC-002".to_string(),
                            project.name.clone(),
                            project.path.display().to_string(),
                            Severity::Warning,
                            format!(
                                "Project '{}' README.md should include a section with the project name ({}) which includes a short description.",
                                project.name, project.name
                            ),
                        ));
                    }
                }
            }

            // DOC-003: Check for CONTRIBUTING.md presence
            let contributing_path = project.path.join("CONTRIBUTING.md");
            if !contributing_path.exists() {
                result.add_violation(Violation::new(
                    StandardType::Documentation,
                    "DOC-003".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Error,
                    format!(
                        "Project '{}' is missing CONTRIBUTING.md. All projects MUST include a CONTRIBUTING.md.",
                        project.name
                    ),
                ));
            } else {
                // DOC-004: Validate CONTRIBUTING.md sections
                if let Ok(contributing_content) = std::fs::read_to_string(&contributing_path) {
                    if !has_getting_started_section(&contributing_content) {
                        result.add_violation(Violation::new(
                            StandardType::Documentation,
                            "DOC-004".to_string(),
                            project.name.clone(),
                            project.path.display().to_string(),
                            Severity::Warning,
                            format!(
                                "Project '{}' CONTRIBUTING.md should include a 'Getting Started' section which describes how to install the project's toolchain, how to build, and how to test.",
                                project.name
                            ),
                        ));
                    }
                }
            }
        }

        result
    }

    #[cfg(test)]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod tests {
        use super::*;

        #[test]
        fn test_has_project_name_section_found() {
            let content = "# my_project\n\nThis is the description.";
            assert!(has_project_name_section(content, "my_project"));
        }

        #[test]
        fn test_has_project_name_section_case_insensitive() {
            let content = "# My_Project\n\nDescription here.";
            assert!(has_project_name_section(content, "my_project"));
        }

        #[test]
        fn test_has_project_name_section_with_multiple_hashes() {
            let content = "## my_project\n\nDescription.";
            assert!(has_project_name_section(content, "my_project"));
        }

        #[test]
        fn test_has_project_name_section_not_found() {
            let content = "# Different Header\n\nSome text.";
            assert!(!has_project_name_section(content, "my_project"));
        }

        #[test]
        fn test_has_getting_started_section_found() {
            let content = "# Contributing\n\n## Getting Started\n\nInstructions here.";
            assert!(has_getting_started_section(content));
        }

        #[test]
        fn test_has_getting_started_section_case_insensitive() {
            let content = "# GETTING STARTED\n\nInstructions.";
            assert!(has_getting_started_section(content));
        }

        #[test]
        fn test_has_getting_started_section_not_found() {
            let content = "# Different Section\n\nSome text.";
            assert!(!has_getting_started_section(content));
        }

        #[test]
        fn test_audit_project_with_all_docs() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_with_all");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create README.md with proper section
            std::fs::write(
                temp_dir.join("README.md"),
                "# test_project\n\nThis is a test project.",
            )
            .expect("Failed to write README.md");

            // Create CONTRIBUTING.md with Getting Started section
            std::fs::write(
                temp_dir.join("CONTRIBUTING.md"),
                "# Contributing\n\n## Getting Started\n\nBuild instructions here.",
            )
            .expect("Failed to write CONTRIBUTING.md");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have no violations
            assert!(!result.has_violations());

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_without_readme() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_no_readme");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have DOC-001 and DOC-003 violations
            assert!(result.has_violations());

            let doc_001 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "DOC-001");
            assert!(doc_001.is_some());
            assert_eq!(doc_001.unwrap().severity, Severity::Error);
            assert!(doc_001.unwrap().message.contains("README.md"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_without_contributing() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_no_contributing");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create README.md
            std::fs::write(temp_dir.join("README.md"), "# test_project\n\nDescription.")
                .expect("Failed to write README.md");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have DOC-003 violation
            assert!(result.has_violations());

            let doc_003 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "DOC-003");
            assert!(doc_003.is_some());
            assert_eq!(doc_003.unwrap().severity, Severity::Error);
            assert!(doc_003.unwrap().message.contains("CONTRIBUTING.md"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_readme_missing_project_name_section() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_readme_no_section");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create README.md without project name section
            std::fs::write(
                temp_dir.join("README.md"),
                "# Wrong Header\n\nSome description.",
            )
            .expect("Failed to write README.md");

            // Create CONTRIBUTING.md
            std::fs::write(
                temp_dir.join("CONTRIBUTING.md"),
                "# Contributing\n\n## Getting Started\n\nInstructions.",
            )
            .expect("Failed to write CONTRIBUTING.md");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have DOC-002 violation (warning)
            assert!(result.has_violations());

            let doc_002 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "DOC-002");
            assert!(doc_002.is_some());
            assert_eq!(doc_002.unwrap().severity, Severity::Warning);
            assert!(doc_002.unwrap().message.contains("project name"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_project_contributing_missing_getting_started() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_no_getting_started");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // Create README.md
            std::fs::write(temp_dir.join("README.md"), "# test_project\n\nDescription.")
                .expect("Failed to write README.md");

            // Create CONTRIBUTING.md without Getting Started section
            std::fs::write(
                temp_dir.join("CONTRIBUTING.md"),
                "# Contributing\n\n## Other Section\n\nSome text.",
            )
            .expect("Failed to write CONTRIBUTING.md");

            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have DOC-004 violation (warning)
            assert!(result.has_violations());

            let doc_004 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "DOC-004");
            assert!(doc_004.is_some());
            assert_eq!(doc_004.unwrap().severity, Severity::Warning);
            assert!(doc_004.unwrap().message.contains("Getting Started"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_audit_multiple_projects_mixed_violations() {
            let temp_dir_root = std::env::temp_dir().join("test_doc_audit_mixed");
            std::fs::remove_dir_all(&temp_dir_root).ok();
            std::fs::create_dir_all(&temp_dir_root).expect("Failed to create test directory");

            // Project 1: Has all documentation
            let proj1_dir = temp_dir_root.join("project1");
            std::fs::create_dir_all(&proj1_dir).expect("Failed to create project1");
            std::fs::write(proj1_dir.join("README.md"), "# project1\n\nDescription.")
                .expect("Failed to write README.md");
            std::fs::write(
                proj1_dir.join("CONTRIBUTING.md"),
                "# Getting Started\n\nInstructions.",
            )
            .expect("Failed to write CONTRIBUTING.md");

            // Project 2: Missing documentation
            let proj2_dir = temp_dir_root.join("project2");
            std::fs::create_dir_all(&proj2_dir).expect("Failed to create project2");

            let projects = vec![
                Project::new(proj1_dir.clone(), ProjectType::Rust, "project1".to_string()),
                Project::new(proj2_dir.clone(), ProjectType::Rust, "project2".to_string()),
            ];

            let result = audit_documentation_standards(&projects);

            // Should have violations only for project2
            assert!(result.has_violations());
            assert!(result
                .violations
                .iter()
                .all(|v| v.project_name == "project2"));

            std::fs::remove_dir_all(&temp_dir_root).ok();
        }

        #[test]
        fn test_audit_typescript_project() {
            let temp_dir = std::env::temp_dir().join("test_doc_audit_typescript");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create test directory");

            // TypeScript projects also need documentation
            let projects = vec![Project::new(
                temp_dir.clone(),
                ProjectType::TypeScript,
                "ts_project".to_string(),
            )];

            let result = audit_documentation_standards(&projects);

            // Should have DOC-001 and DOC-003 violations
            assert!(result.has_violations());
            assert!(result.violations.iter().any(|v| v.standard_id == "DOC-001"));
            assert!(result.violations.iter().any(|v| v.standard_id == "DOC-003"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }
    }
}

/// Linting standards audit module
pub mod linting {
    use super::*;
    use std::path::Path;

    /// Lint configuration found in a Cargo.toml file
    #[derive(Debug, Default)]
    struct CargoLintConfig {
        /// Whether `[lints] workspace = true` is set
        inherits_workspace: bool,
        /// Whether `unwrap_used = "warn"` is present in clippy lints
        has_unwrap_used_warn: bool,
        /// Whether `expect_used = "warn"` is present in clippy lints
        has_expect_used_warn: bool,
        /// Whether `unsafe_code = "forbid"` is present in rust lints
        has_unsafe_code_forbid: bool,
    }

    /// Sections that can appear in a Cargo.toml relevant to lints
    #[derive(Debug, PartialEq)]
    enum LintSection {
        None,
        LintsWorkspace,
        LintsClippy,
        LintsRust,
        WorkspaceLintsClippy,
        WorkspaceLintsRust,
    }

    /// Parse a Cargo.toml file and extract lint configuration
    fn parse_cargo_lint_config(path: &Path) -> CargoLintConfig {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return CargoLintConfig::default(),
        };

        let mut config = CargoLintConfig::default();
        let mut current_section = LintSection::None;

        for line in content.lines() {
            let trimmed = line.trim();

            // Detect section headers
            if trimmed == "[lints]" {
                current_section = LintSection::LintsWorkspace;
                continue;
            }
            if trimmed == "[lints.clippy]" {
                current_section = LintSection::LintsClippy;
                continue;
            }
            if trimmed == "[lints.rust]" {
                current_section = LintSection::LintsRust;
                continue;
            }
            if trimmed == "[workspace.lints.clippy]" {
                current_section = LintSection::WorkspaceLintsClippy;
                continue;
            }
            if trimmed == "[workspace.lints.rust]" {
                current_section = LintSection::WorkspaceLintsRust;
                continue;
            }
            // Any other section header resets
            if trimmed.starts_with('[') {
                current_section = LintSection::None;
                continue;
            }

            match current_section {
                LintSection::LintsWorkspace => {
                    if trimmed.starts_with("workspace") && trimmed.contains('=') {
                        let value_part =
                            trimmed.split_once('=').map(|(_, v)| v).unwrap_or("").trim();
                        if value_part == "true" {
                            config.inherits_workspace = true;
                        }
                    }
                }
                LintSection::LintsClippy | LintSection::WorkspaceLintsClippy => {
                    if trimmed.starts_with("unwrap_used") && trimmed.contains('=') {
                        let value_part =
                            trimmed.split_once('=').map(|(_, v)| v).unwrap_or("").trim();
                        let value = value_part.trim_matches('"').trim_matches('\'');
                        if value == "warn" {
                            config.has_unwrap_used_warn = true;
                        }
                    }
                    if trimmed.starts_with("expect_used") && trimmed.contains('=') {
                        let value_part =
                            trimmed.split_once('=').map(|(_, v)| v).unwrap_or("").trim();
                        let value = value_part.trim_matches('"').trim_matches('\'');
                        if value == "warn" {
                            config.has_expect_used_warn = true;
                        }
                    }
                }
                LintSection::LintsRust | LintSection::WorkspaceLintsRust => {
                    if trimmed.starts_with("unsafe_code") && trimmed.contains('=') {
                        let value_part =
                            trimmed.split_once('=').map(|(_, v)| v).unwrap_or("").trim();
                        let value = value_part.trim_matches('"').trim_matches('\'');
                        if value == "forbid" {
                            config.has_unsafe_code_forbid = true;
                        }
                    }
                }
                LintSection::None => {}
            }
        }

        config
    }

    /// Find the workspace Cargo.toml for a given project by searching parent directories
    fn find_workspace_cargo_toml(project_path: &Path) -> Option<std::path::PathBuf> {
        let mut current = project_path.parent()?;
        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists() {
                // Check if this is a workspace Cargo.toml (contains [workspace])
                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                    if content.lines().any(|l| l.trim() == "[workspace]") {
                        return Some(cargo_toml);
                    }
                }
            }
            current = current.parent()?;
        }
    }

    /// Audit projects for linting standards compliance
    ///
    /// Checks:
    /// - LIN-001: All Rust projects MUST configure clippy lints: `unwrap_used = "warn"` and
    ///   `expect_used = "warn"`, either directly or via workspace lint inheritance.
    /// - LIN-002: All Rust projects MUST configure `unsafe_code = "forbid"` in rust lints,
    ///   either directly or via workspace lint inheritance.
    pub fn audit_linting_standards(projects: &[Project]) -> AuditResult {
        let mut result = AuditResult::new();

        for project in projects {
            if project.project_type != ProjectType::Rust {
                continue;
            }

            let cargo_toml = project.path.join("Cargo.toml");
            if !cargo_toml.exists() {
                continue;
            }

            let config = parse_cargo_lint_config(&cargo_toml);

            // Resolve effective lint config: if workspace inheritance is used,
            // look up the workspace Cargo.toml
            let effective = if config.inherits_workspace {
                if let Some(ws_path) = find_workspace_cargo_toml(&project.path) {
                    let ws_config = parse_cargo_lint_config(&ws_path);
                    CargoLintConfig {
                        inherits_workspace: true,
                        has_unwrap_used_warn: ws_config.has_unwrap_used_warn,
                        has_expect_used_warn: ws_config.has_expect_used_warn,
                        has_unsafe_code_forbid: ws_config.has_unsafe_code_forbid,
                    }
                } else {
                    // Workspace not found; treat as if no lints configured
                    CargoLintConfig::default()
                }
            } else {
                CargoLintConfig {
                    inherits_workspace: false,
                    has_unwrap_used_warn: config.has_unwrap_used_warn,
                    has_expect_used_warn: config.has_expect_used_warn,
                    has_unsafe_code_forbid: config.has_unsafe_code_forbid,
                }
            };

            // LIN-001: Check clippy lints
            if !effective.has_unwrap_used_warn || !effective.has_expect_used_warn {
                let missing: Vec<&str> = [
                    (!effective.has_unwrap_used_warn).then_some("unwrap_used = \"warn\""),
                    (!effective.has_expect_used_warn).then_some("expect_used = \"warn\""),
                ]
                .into_iter()
                .flatten()
                .collect();

                result.add_violation(Violation::new(
                    StandardType::Rust,
                    "LIN-001".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Warning,
                    format!(
                        "Project '{}' Cargo.toml is missing required clippy lints: {}. \
                        Add these to [lints.clippy] or use workspace lint inheritance.",
                        project.name,
                        missing.join(", ")
                    ),
                ));
            }

            // LIN-002: Check rust lints
            if !effective.has_unsafe_code_forbid {
                result.add_violation(Violation::new(
                    StandardType::Rust,
                    "LIN-002".to_string(),
                    project.name.clone(),
                    project.path.display().to_string(),
                    Severity::Warning,
                    format!(
                        "Project '{}' Cargo.toml is missing required rust lint: \
                        unsafe_code = \"forbid\". \
                        Add this to [lints.rust] or use workspace lint inheritance.",
                        project.name
                    ),
                ));
            }
        }

        result
    }

    #[cfg(test)]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod tests {
        use super::*;
        use crate::discovery::ProjectType;

        fn make_project(dir: &std::path::Path) -> Project {
            Project::new(
                dir.to_path_buf(),
                ProjectType::Rust,
                "test_project".to_string(),
            )
        }

        #[test]
        fn test_project_with_direct_lints_passes() {
            let temp_dir = std::env::temp_dir().join("test_lin_direct_lints");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints.clippy]\nunwrap_used = \"warn\"\nexpect_used = \"warn\"\n\n\
                 [lints.rust]\nunsafe_code = \"forbid\"\n",
            )
            .expect("Failed to write Cargo.toml");

            let result = audit_linting_standards(&[make_project(&temp_dir)]);
            assert!(
                !result.has_violations(),
                "Expected no violations but got: {:?}",
                result.violations
            );

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_project_missing_clippy_lints_fails() {
            let temp_dir = std::env::temp_dir().join("test_lin_missing_clippy");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints.rust]\nunsafe_code = \"forbid\"\n",
            )
            .expect("Failed to write Cargo.toml");

            let result = audit_linting_standards(&[make_project(&temp_dir)]);
            assert!(result.has_violations());
            assert!(result.violations.iter().any(|v| v.standard_id == "LIN-001"));
            assert!(!result.violations.iter().any(|v| v.standard_id == "LIN-002"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_project_missing_rust_lints_fails() {
            let temp_dir = std::env::temp_dir().join("test_lin_missing_rust");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints.clippy]\nunwrap_used = \"warn\"\nexpect_used = \"warn\"\n",
            )
            .expect("Failed to write Cargo.toml");

            let result = audit_linting_standards(&[make_project(&temp_dir)]);
            assert!(result.has_violations());
            assert!(!result.violations.iter().any(|v| v.standard_id == "LIN-001"));
            assert!(result.violations.iter().any(|v| v.standard_id == "LIN-002"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_project_missing_all_lints_fails() {
            let temp_dir = std::env::temp_dir().join("test_lin_missing_all");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n",
            )
            .expect("Failed to write Cargo.toml");

            let result = audit_linting_standards(&[make_project(&temp_dir)]);
            assert!(result.has_violations());
            assert_eq!(result.violations.len(), 2);
            assert!(result.violations.iter().any(|v| v.standard_id == "LIN-001"));
            assert!(result.violations.iter().any(|v| v.standard_id == "LIN-002"));

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_project_with_workspace_lints_passes() {
            let temp_dir = std::env::temp_dir().join("test_lin_workspace_lints");
            std::fs::remove_dir_all(&temp_dir).ok();

            // Create workspace Cargo.toml
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");
            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[workspace]\nmembers = [\"member\"]\n\n\
                 [workspace.lints.clippy]\nunwrap_used = \"warn\"\nexpect_used = \"warn\"\n\n\
                 [workspace.lints.rust]\nunsafe_code = \"forbid\"\n",
            )
            .expect("Failed to write workspace Cargo.toml");

            // Create member project with workspace lint inheritance
            let member_dir = temp_dir.join("member");
            std::fs::create_dir_all(&member_dir).expect("Failed to create member dir");
            std::fs::write(
                member_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints]\nworkspace = true\n",
            )
            .expect("Failed to write member Cargo.toml");

            let project = Project::new(
                member_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            );
            let result = audit_linting_standards(&[project]);
            assert!(
                !result.has_violations(),
                "Expected no violations but got: {:?}",
                result.violations
            );

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_project_with_workspace_lints_missing_in_workspace_fails() {
            let temp_dir = std::env::temp_dir().join("test_lin_workspace_missing");
            std::fs::remove_dir_all(&temp_dir).ok();

            // Create workspace Cargo.toml without required lints
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");
            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[workspace]\nmembers = [\"member\"]\n",
            )
            .expect("Failed to write workspace Cargo.toml");

            // Create member project with workspace lint inheritance
            let member_dir = temp_dir.join("member");
            std::fs::create_dir_all(&member_dir).expect("Failed to create member dir");
            std::fs::write(
                member_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints]\nworkspace = true\n",
            )
            .expect("Failed to write member Cargo.toml");

            let project = Project::new(
                member_dir.clone(),
                ProjectType::Rust,
                "test_project".to_string(),
            );
            let result = audit_linting_standards(&[project]);
            assert!(result.has_violations());
            assert_eq!(result.violations.len(), 2);

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_typescript_project_skipped() {
            let temp_dir = std::env::temp_dir().join("test_lin_typescript_skip");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            let project = Project::new(
                temp_dir.clone(),
                ProjectType::TypeScript,
                "ts_project".to_string(),
            );
            let result = audit_linting_standards(&[project]);
            assert!(!result.has_violations());

            std::fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_only_unwrap_used_missing() {
            let temp_dir = std::env::temp_dir().join("test_lin_only_unwrap_missing");
            std::fs::remove_dir_all(&temp_dir).ok();
            std::fs::create_dir_all(&temp_dir).expect("Failed to create dir");

            std::fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"\n\n\
                 [lints.clippy]\nexpect_used = \"warn\"\n\n\
                 [lints.rust]\nunsafe_code = \"forbid\"\n",
            )
            .expect("Failed to write Cargo.toml");

            let result = audit_linting_standards(&[make_project(&temp_dir)]);
            assert!(result.has_violations());
            let lin001 = result
                .violations
                .iter()
                .find(|v| v.standard_id == "LIN-001")
                .expect("Should have LIN-001");
            assert!(
                lin001.message.contains("unwrap_used"),
                "Message should mention unwrap_used"
            );
            assert!(
                !lin001.message.contains("expect_used"),
                "Message should not mention expect_used"
            );

            std::fs::remove_dir_all(&temp_dir).ok();
        }
    }
}

/// Convert a violation to a TODO string for ISSUES.md
///
/// Creates a formatted TODO entry with the standard ID, project name, and message.
/// The format is: `TODO (agent-generated): [STANDARD-ID] project_name - message`
pub fn violation_to_todo(violation: &Violation) -> String {
    format!(
        "TODO (agent-generated): [{}] {} - {}",
        violation.standard_id, violation.project_name, violation.message
    )
}

/// Write audit violations to a project's ISSUES.md file
///
/// # Arguments
///
/// * `audit_result` - The audit results containing violations
/// * `issues_path` - Path to the ISSUES.md file to write to
///
/// # Returns
///
/// * `Ok(usize)` - Number of TODOs written (excluding duplicates)
/// * `Err(String)` - Error message if writing fails
pub fn write_violations_to_issues(
    audit_result: &AuditResult,
    issues_path: &std::path::Path,
) -> Result<usize, String> {
    use crate::issues::IssuesFile;

    // Try to parse existing file, or create new one if it doesn't exist
    let mut issues_file = if issues_path.exists() {
        IssuesFile::parse(issues_path).map_err(|e| format!("Failed to parse ISSUES.md: {}", e))?
    } else {
        IssuesFile::new(issues_path.to_string_lossy().to_string())
    };

    // Convert violations to TODOs and add them
    let mut added_count = 0;
    for violation in &audit_result.violations {
        let todo = violation_to_todo(violation);
        if issues_file.add_priority_todo(todo) {
            added_count += 1;
        }
    }

    // Write the updated file
    issues_file
        .write()
        .map_err(|e| format!("Failed to write ISSUES.md: {}", e))?;

    Ok(added_count)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let violation = Violation::new(
            StandardType::Naming,
            "NAM-001".to_string(),
            "test_project".to_string(),
            "/repo/test_project".to_string(),
            Severity::Error,
            "Test message".to_string(),
        );

        assert_eq!(violation.standard_type, StandardType::Naming);
        assert_eq!(violation.standard_id, "NAM-001");
        assert_eq!(violation.project_name, "test_project");
        assert_eq!(violation.severity, Severity::Error);
    }

    #[test]
    fn test_audit_result_empty() {
        let result = AuditResult::new();
        assert!(!result.has_violations());
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_audit_result_with_violations() {
        let mut result = AuditResult::new();
        result.add_violation(Violation::new(
            StandardType::Naming,
            "NAM-001".to_string(),
            "test".to_string(),
            "/test".to_string(),
            Severity::Error,
            "Test".to_string(),
        ));

        assert!(result.has_violations());
        assert_eq!(result.violations.len(), 1);
    }
}
