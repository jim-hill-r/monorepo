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
    /// - NAM-002: Directory name must match package name
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

            // NAM-002: Check if directory name matches package name
            if let Some(dir_name) = project.path.file_name().and_then(|n| n.to_str()) {
                if dir_name != project.name {
                    result.add_violation(Violation::new(
                        StandardType::Naming,
                        "NAM-002".to_string(),
                        project.name.clone(),
                        project.path.display().to_string(),
                        Severity::Error,
                        format!(
                            "Directory name '{}' does not match package name '{}'. Directory name MUST match the package name.",
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

#[cfg(test)]
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
