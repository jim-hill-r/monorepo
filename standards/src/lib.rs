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
//! - `discovery`: Module for discovering projects in the monorepo

#![warn(missing_docs)]

use std::fmt;

/// Type of standard being enforced
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Build and CI standards
    BuildAndCI,
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
            StandardType::BuildAndCI => write!(f, "Build and CI"),
        }
    }
}

/// Severity level of a standard violation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        self.standard_type
    }

    fn id(&self) -> String {
        self.id.clone()
    }

    fn description(&self) -> String {
        self.description.clone()
    }

    fn severity(&self) -> Severity {
        self.severity
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

            // Parse bullet list items (lines starting with "- ")
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
                    standard_type,
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
            StandardType::BuildAndCI => "BLD",
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
            ("build-and-ci.md", StandardType::BuildAndCI),
        ];

        for (filename, standard_type) in file_mappings {
            let file_path = docs_path.join(filename);
            if file_path.exists() {
                match load_from_file(&file_path, standard_type) {
                    Ok(standards) => all_standards.extend(standards),
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

/// Project discovery module for finding projects in the monorepo
pub mod discovery {
    use std::path::{Path, PathBuf};

    /// Type of project detected
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum ProjectType {
        /// Rust project with Cargo.toml
        Rust,
        /// TypeScript/Node.js project with package.json
        TypeScript,
    }

    /// Represents a discovered project in the monorepo
    #[derive(Debug, Clone)]
    pub struct Project {
        /// Path to the project directory
        pub path: PathBuf,
        /// Type of project
        pub project_type: ProjectType,
        /// Project name (from Cargo.toml or package.json)
        pub name: String,
    }

    impl Project {
        /// Create a new Project
        pub fn new(path: PathBuf, project_type: ProjectType, name: String) -> Self {
            Self {
                path,
                project_type,
                name,
            }
        }
    }

    /// Discover all projects in a directory tree
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory to search from
    ///
    /// # Returns
    ///
    /// Vector of discovered projects
    pub fn discover_projects(root: &Path) -> Result<Vec<Project>, std::io::Error> {
        const MAX_SEARCH_DEPTH: usize = 10;
        let mut projects = Vec::new();
        discover_projects_recursive(root, &mut projects, 0, MAX_SEARCH_DEPTH)?;
        Ok(projects)
    }

    fn discover_projects_recursive(
        dir: &Path,
        projects: &mut Vec<Project>,
        depth: usize,
        max_depth: usize,
    ) -> Result<(), std::io::Error> {
        if depth > max_depth {
            return Ok(());
        }

        // Skip directories that should not be searched
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            if name == "node_modules" || name == "target" || name == ".git" {
                return Ok(());
            }
        }

        // Check for Cargo.toml (Rust project)
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Some(name) = extract_cargo_name(&cargo_toml) {
                projects.push(Project::new(dir.to_path_buf(), ProjectType::Rust, name));
            }
        }

        // Check for package.json (TypeScript/Node.js project)
        let package_json = dir.join("package.json");
        if package_json.exists() {
            if let Some(name) = extract_package_name(&package_json) {
                projects.push(Project::new(
                    dir.to_path_buf(),
                    ProjectType::TypeScript,
                    name,
                ));
            }
        }

        // Recursively check subdirectories
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    discover_projects_recursive(&path, projects, depth + 1, max_depth)?;
                }
            }
        }

        Ok(())
    }

    fn extract_cargo_name(cargo_toml: &Path) -> Option<String> {
        let content = std::fs::read_to_string(cargo_toml).ok()?;

        // Simple parser to extract package name from Cargo.toml
        // Look for: name = "package_name" in the [package] section
        let mut in_package_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check if we're entering the [package] section
            if trimmed == "[package]" {
                in_package_section = true;
                continue;
            }

            // Check if we're entering a different section
            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[package]" {
                in_package_section = false;
                continue;
            }

            // Only parse 'name' field if we're in the [package] section
            if in_package_section && trimmed.starts_with("name") {
                if let Some(equals_pos) = trimmed.find('=') {
                    // Ensure it's the 'name' field, not something like 'rename'
                    if trimmed[..equals_pos].trim() == "name" {
                        let value = &trimmed[equals_pos + 1..].trim();
                        // Remove quotes
                        let name = value.trim_matches('"').trim_matches('\'');
                        return Some(name.to_string());
                    }
                }
            }
        }

        None
    }

    fn extract_package_name(package_json: &Path) -> Option<String> {
        let content = std::fs::read_to_string(package_json).ok()?;

        // Simple parser to extract name from package.json
        // Look for: "name": "package-name"
        // This is a simplified parser that works for most common JSON formats

        // First try to find "name" field
        if let Some(name_start) = content.find("\"name\"") {
            // Look for the colon after "name"
            let after_name = &content[name_start + "\"name\"".len()..];
            if let Some(colon_pos) = after_name.find(':') {
                let after_colon = &after_name[colon_pos + 1..];

                // Find the opening quote for the value
                let trimmed = after_colon.trim_start();
                if let Some(after_quote) = trimmed.strip_prefix('"') {
                    // Find the closing quote (not escaped)
                    if let Some(end_quote) = after_quote.find('"') {
                        let name = &after_quote[..end_quote];
                        return Some(name.to_string());
                    }
                }
            }
        }

        None
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::fs;

        #[test]
        fn test_discover_rust_project() {
            let temp_dir = std::env::temp_dir().join("test_rust_project");
            fs::create_dir_all(&temp_dir).unwrap();

            let cargo_toml = temp_dir.join("Cargo.toml");
            fs::write(
                &cargo_toml,
                "[package]\nname = \"test_project\"\nversion = \"0.1.0\"",
            )
            .unwrap();

            let projects = discover_projects(&temp_dir).unwrap();

            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0].name, "test_project");
            assert_eq!(projects[0].project_type, ProjectType::Rust);

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_discover_typescript_project() {
            let temp_dir = std::env::temp_dir().join("test_ts_project");
            fs::create_dir_all(&temp_dir).unwrap();

            let package_json = temp_dir.join("package.json");
            fs::write(
                &package_json,
                "{\n  \"name\": \"test-package\",\n  \"version\": \"1.0.0\"\n}",
            )
            .unwrap();

            let projects = discover_projects(&temp_dir).unwrap();

            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0].name, "test-package");
            assert_eq!(projects[0].project_type, ProjectType::TypeScript);

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_discover_multiple_projects() {
            let temp_dir = std::env::temp_dir().join("test_multiple_projects");
            // Clean up any existing test directory
            fs::remove_dir_all(&temp_dir).ok();
            fs::create_dir_all(&temp_dir).unwrap();

            // Create Rust project
            let rust_dir = temp_dir.join("rust_project");
            fs::create_dir_all(&rust_dir).unwrap();
            fs::write(
                rust_dir.join("Cargo.toml"),
                "[package]\nname = \"rust_project\"\n",
            )
            .unwrap();

            // Create TypeScript project
            let ts_dir = temp_dir.join("ts_project");
            fs::create_dir_all(&ts_dir).unwrap();
            fs::write(ts_dir.join("package.json"), "{\"name\": \"ts-project\"}").unwrap();

            let projects = discover_projects(&temp_dir).unwrap();

            // Debug output if assertion fails
            if projects.len() != 2 {
                eprintln!("Expected 2 projects, found {}", projects.len());
                for proj in &projects {
                    eprintln!(
                        "  - {} ({:?}) at {:?}",
                        proj.name, proj.project_type, proj.path
                    );
                }
            }

            assert_eq!(projects.len(), 2);

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_discover_skips_node_modules() {
            let temp_dir = std::env::temp_dir().join("test_skip_node_modules");
            // Clean up any existing test directory
            fs::remove_dir_all(&temp_dir).ok();
            fs::create_dir_all(&temp_dir).unwrap();

            // Create a project with node_modules
            fs::write(
                temp_dir.join("package.json"),
                "{\"name\": \"main-project\"}",
            )
            .unwrap();

            let node_modules = temp_dir.join("node_modules").join("some-lib");
            fs::create_dir_all(&node_modules).unwrap();
            fs::write(
                node_modules.join("package.json"),
                "{\"name\": \"should-skip\"}",
            )
            .unwrap();

            let projects = discover_projects(&temp_dir).unwrap();

            // Debug output if assertion fails
            if projects.len() != 1 {
                eprintln!("Expected 1 project, found {}", projects.len());
                for proj in &projects {
                    eprintln!(
                        "  - {} ({:?}) at {:?}",
                        proj.name, proj.project_type, proj.path
                    );
                }
            }

            // Should only find the main project, not the one in node_modules
            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0].name, "main-project");

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_discover_skips_target_dir() {
            let temp_dir = std::env::temp_dir().join("test_skip_target");
            fs::create_dir_all(&temp_dir).unwrap();

            // Create a Rust project with target directory
            fs::write(
                temp_dir.join("Cargo.toml"),
                "[package]\nname = \"main_project\"\n",
            )
            .unwrap();

            let target = temp_dir.join("target").join("debug");
            fs::create_dir_all(&target).unwrap();
            fs::write(
                target.join("Cargo.toml"),
                "[package]\nname = \"should_skip\"\n",
            )
            .unwrap();

            let projects = discover_projects(&temp_dir).unwrap();

            // Should only find the main project, not the one in target
            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0].name, "main_project");

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_extract_cargo_name() {
            let temp_dir = std::env::temp_dir().join("test_extract_cargo");
            fs::create_dir_all(&temp_dir).unwrap();

            let cargo_toml = temp_dir.join("Cargo.toml");
            fs::write(
                &cargo_toml,
                "[package]\nname = \"my-crate\"\nversion = \"0.1.0\"",
            )
            .unwrap();

            let name = extract_cargo_name(&cargo_toml);
            assert_eq!(name, Some("my-crate".to_string()));

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_extract_cargo_name_ignores_rename() {
            let temp_dir = std::env::temp_dir().join("test_extract_cargo_rename");
            fs::create_dir_all(&temp_dir).unwrap();

            let cargo_toml = temp_dir.join("Cargo.toml");
            fs::write(
                &cargo_toml,
                "[package]\nname = \"correct-name\"\nversion = \"0.1.0\"\n\n[dependencies]\nrename = \"should-not-match\"",
            )
            .unwrap();

            let name = extract_cargo_name(&cargo_toml);
            assert_eq!(name, Some("correct-name".to_string()));

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_extract_cargo_name_only_in_package_section() {
            let temp_dir = std::env::temp_dir().join("test_extract_cargo_section");
            fs::create_dir_all(&temp_dir).unwrap();

            let cargo_toml = temp_dir.join("Cargo.toml");
            fs::write(
                &cargo_toml,
                "[lib]\nname = \"wrong-name\"\n\n[package]\nname = \"correct-name\"\nversion = \"0.1.0\"",
            )
            .unwrap();

            let name = extract_cargo_name(&cargo_toml);
            assert_eq!(name, Some("correct-name".to_string()));

            fs::remove_dir_all(&temp_dir).ok();
        }

        #[test]
        fn test_extract_package_name() {
            let temp_dir = std::env::temp_dir().join("test_extract_package");
            fs::create_dir_all(&temp_dir).unwrap();

            let package_json = temp_dir.join("package.json");
            fs::write(
                &package_json,
                "{\n  \"name\": \"my-package\",\n  \"version\": \"1.0.0\"\n}",
            )
            .unwrap();

            let name = extract_package_name(&package_json);
            assert_eq!(name, Some("my-package".to_string()));

            fs::remove_dir_all(&temp_dir).ok();
        }
    }
}
