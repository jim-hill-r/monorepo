//! Standards enforcement library for the monorepo
//!
//! This library provides utilities and checks to ensure all projects in the
//! monorepo follow consistent standards and best practices.
//!
//! # Architecture
//!
//! The standards framework consists of:
//! - `Standard` trait: Core trait for representing individual standard rules
//! - `StandardType`: Enumeration of different standard categories
//! - `parser`: Module for parsing standards from markdown documentation
//! - `loader`: Module for loading standards from the docs/ directory

#![warn(missing_docs)]

use std::fmt;

/// Type of standard being enforced
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardType {
    /// Naming conventions (e.g., snake_case for projects)
    Naming,
    /// Configuration requirements (e.g., royalty.toml presence)
    Configuration,
    /// Documentation requirements (e.g., README.md presence)
    Documentation,
    /// Testing requirements
    Testing,
    /// Rust-specific standards
    Rust,
    /// TypeScript-specific standards
    TypeScript,
    /// Toolchain management standards
    Toolchain,
    /// Issue management standards
    IssueManagement,
}

impl fmt::Display for StandardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StandardType::Naming => write!(f, "Naming"),
            StandardType::Configuration => write!(f, "Configuration"),
            StandardType::Documentation => write!(f, "Documentation"),
            StandardType::Testing => write!(f, "Testing"),
            StandardType::Rust => write!(f, "Rust"),
            StandardType::TypeScript => write!(f, "TypeScript"),
            StandardType::Toolchain => write!(f, "Toolchain"),
            StandardType::IssueManagement => write!(f, "Issue Management"),
        }
    }
}

/// Severity level of a standard violation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    /// Must be fixed - blocks merging
    Error,
    /// Should be fixed - doesn't block merging
    Warning,
    /// Nice to have - informational only
    Info,
}

/// Represents a single standard rule
pub trait Standard: fmt::Debug {
    /// Returns the type of this standard
    fn standard_type(&self) -> StandardType;

    /// Returns a unique identifier for this standard
    fn id(&self) -> String;

    /// Returns a human-readable description of the standard
    fn description(&self) -> String;

    /// Returns the severity level of violations
    fn severity(&self) -> Severity;
}

/// A concrete implementation of a standard parsed from markdown
#[derive(Debug, Clone)]
pub struct ParsedStandard {
    /// Type of standard
    pub standard_type: StandardType,
    /// Unique identifier
    pub id: String,
    /// Description of the standard
    pub description: String,
    /// Severity level
    pub severity: Severity,
}

impl Standard for ParsedStandard {
    fn standard_type(&self) -> StandardType {
        self.standard_type.clone()
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn severity(&self) -> Severity {
        self.severity.clone()
    }
}

/// Parser for extracting standards from markdown documentation
pub mod parser {
    use super::{ParsedStandard, Severity, StandardType};

    /// Parse standards from markdown content
    ///
    /// # Arguments
    ///
    /// * `content` - The markdown content to parse
    /// * `standard_type` - The type of standard being parsed
    ///
    /// # Returns
    ///
    /// A vector of parsed standards
    pub fn parse_markdown(content: &str, standard_type: StandardType) -> Vec<ParsedStandard> {
        let mut standards = Vec::new();
        let mut counter = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Look for lines starting with "- " followed by "MUST" or "SHOULD"
            if trimmed.starts_with("- ") {
                let description = trimmed.strip_prefix("- ").unwrap_or(trimmed).to_string();

                // Determine severity based on keywords
                let severity = if description.contains("MUST") {
                    Severity::Error
                } else if description.contains("SHOULD") {
                    Severity::Warning
                } else {
                    Severity::Info
                };

                counter += 1;
                let id = format!("{}-{:03}", standard_type_to_prefix(&standard_type), counter);

                standards.push(ParsedStandard {
                    standard_type: standard_type.clone(),
                    id,
                    description,
                    severity,
                });
            }
        }

        standards
    }

    fn standard_type_to_prefix(st: &StandardType) -> &str {
        match st {
            StandardType::Naming => "NAM",
            StandardType::Configuration => "CFG",
            StandardType::Documentation => "DOC",
            StandardType::Testing => "TST",
            StandardType::Rust => "RST",
            StandardType::TypeScript => "TSC",
            StandardType::Toolchain => "TLC",
            StandardType::IssueManagement => "ISS",
        }
    }
}

/// Loader for reading standards from the docs/ directory
pub mod loader {
    use super::{parser, ParsedStandard, StandardType};
    use std::path::Path;

    /// Load standards from a markdown file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the markdown file
    /// * `standard_type` - The type of standard in the file
    ///
    /// # Returns
    ///
    /// Result containing vector of standards or IO error
    pub fn load_from_file(
        file_path: &Path,
        standard_type: StandardType,
    ) -> Result<Vec<ParsedStandard>, std::io::Error> {
        let content = std::fs::read_to_string(file_path)?;
        Ok(parser::parse_markdown(&content, standard_type))
    }

    /// Load all standards from the docs/ directory
    ///
    /// # Arguments
    ///
    /// * `docs_path` - Path to the docs directory
    ///
    /// # Returns
    ///
    /// Result containing vector of all standards or IO error
    pub fn load_all_standards(docs_path: &Path) -> Result<Vec<ParsedStandard>, std::io::Error> {
        let mut all_standards = Vec::new();

        // Define the mapping of files to standard types
        let file_mappings = vec![
            ("naming.md", StandardType::Naming),
            ("configuration.md", StandardType::Configuration),
            ("documentation.md", StandardType::Documentation),
            ("testing.md", StandardType::Testing),
            ("rust.md", StandardType::Rust),
            ("typescript.md", StandardType::TypeScript),
            ("toolchain.md", StandardType::Toolchain),
            ("issue-management.md", StandardType::IssueManagement),
        ];

        for (filename, standard_type) in file_mappings {
            let file_path = docs_path.join(filename);
            if file_path.exists() {
                match load_from_file(&file_path, standard_type) {
                    Ok(mut standards) => all_standards.append(&mut standards),
                    Err(_) => {
                        // Skip files that can't be read
                        continue;
                    }
                }
            }
        }

        Ok(all_standards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_standard_type_display() {
        assert_eq!(StandardType::Naming.to_string(), "Naming");
        assert_eq!(StandardType::Configuration.to_string(), "Configuration");
        assert_eq!(StandardType::Documentation.to_string(), "Documentation");
    }

    #[test]
    fn test_parsed_standard_implements_trait() {
        let standard = ParsedStandard {
            standard_type: StandardType::Naming,
            id: "NAM-001".to_string(),
            description: "All projects MUST be snake_case.".to_string(),
            severity: Severity::Error,
        };

        assert_eq!(standard.standard_type(), StandardType::Naming);
        assert_eq!(standard.id(), "NAM-001");
        assert_eq!(standard.description(), "All projects MUST be snake_case.");
        assert_eq!(standard.severity(), Severity::Error);
    }

    #[test]
    fn test_parse_naming_standards() {
        let content = r#"- All projects MUST be snake_case.
- All projects MUST have the same directory name as their Cargo package name.
- All deployments MUST be kebab-case."#;

        let standards = parser::parse_markdown(content, StandardType::Naming);

        assert_eq!(standards.len(), 3);
        assert_eq!(standards[0].id, "NAM-001");
        assert_eq!(standards[0].severity, Severity::Error);
        assert!(standards[0].description.contains("snake_case"));

        assert_eq!(standards[1].id, "NAM-002");
        assert!(standards[1].description.contains("directory name"));

        assert_eq!(standards[2].id, "NAM-003");
        assert!(standards[2].description.contains("kebab-case"));
    }

    #[test]
    fn test_parse_configuration_standards() {
        let content =
            "- All projects MUST include a royalty.toml for use with the royalty project.";

        let standards = parser::parse_markdown(content, StandardType::Configuration);

        assert_eq!(standards.len(), 1);
        assert_eq!(standards[0].id, "CFG-001");
        assert_eq!(standards[0].severity, Severity::Error);
        assert!(standards[0].description.contains("royalty.toml"));
    }

    #[test]
    fn test_parse_with_should_keyword() {
        let content = "- Projects SHOULD include unit tests.";

        let standards = parser::parse_markdown(content, StandardType::Testing);

        assert_eq!(standards.len(), 1);
        assert_eq!(standards[0].severity, Severity::Warning);
    }

    #[test]
    fn test_parse_with_info_level() {
        let content = "- Projects may include benchmarks for performance testing.";

        let standards = parser::parse_markdown(content, StandardType::Testing);

        assert_eq!(standards.len(), 1);
        assert_eq!(standards[0].severity, Severity::Info);
    }

    #[test]
    fn test_parse_ignores_non_list_items() {
        let content = r#"# Title
Some paragraph text
- All projects MUST be snake_case.
More text
## Heading
- Another MUST rule."#;

        let standards = parser::parse_markdown(content, StandardType::Naming);

        assert_eq!(standards.len(), 2);
        assert_eq!(standards[0].id, "NAM-001");
        assert_eq!(standards[1].id, "NAM-002");
    }

    #[test]
    fn test_load_naming_standards_from_file() {
        let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
        let naming_file = docs_path.join("naming.md");

        let result = loader::load_from_file(&naming_file, StandardType::Naming);
        assert!(result.is_ok());

        let standards = result.unwrap();
        assert!(!standards.is_empty());
        assert!(standards
            .iter()
            .all(|s| matches!(s.standard_type, StandardType::Naming)));
    }

    #[test]
    fn test_load_configuration_standards_from_file() {
        let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
        let config_file = docs_path.join("configuration.md");

        let result = loader::load_from_file(&config_file, StandardType::Configuration);
        assert!(result.is_ok());

        let standards = result.unwrap();
        assert!(!standards.is_empty());
        assert!(standards
            .iter()
            .all(|s| matches!(s.standard_type, StandardType::Configuration)));
    }

    #[test]
    fn test_load_all_standards() {
        let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");

        let result = loader::load_all_standards(&docs_path);
        assert!(result.is_ok());

        let all_standards = result.unwrap();
        assert!(!all_standards.is_empty());

        // Verify we have standards from multiple types
        let types: std::collections::HashSet<_> = all_standards
            .iter()
            .map(|s| format!("{:?}", s.standard_type))
            .collect();
        assert!(
            types.len() > 1,
            "Should have loaded multiple standard types"
        );
    }

    #[test]
    fn test_load_nonexistent_file_returns_error() {
        let nonexistent = PathBuf::from("/nonexistent/path/file.md");
        let result = loader::load_from_file(&nonexistent, StandardType::Naming);
        assert!(result.is_err());
    }
}
