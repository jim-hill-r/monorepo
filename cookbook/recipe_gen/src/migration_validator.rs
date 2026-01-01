use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Validates that recipe and plan migration completed successfully
///
/// This tool verifies:
/// - All recipe files have valid UUIDs in frontmatter
/// - All recipe files are named with their UUID (no day-*.md files)
/// - All plan files have Recipe UUIDs field
/// - All recipe UUIDs in plan files reference existing recipe files
/// - No orphaned day-*.md files exist
pub struct MigrationValidator {
    content_dir: PathBuf,
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub total_recipe_files: usize,
    pub recipes_with_uuid: usize,
    pub recipes_with_uuid_name: usize,
    pub day_named_files: Vec<String>,
    pub recipes_missing_uuid: Vec<String>,
    pub recipes_with_invalid_uuid: Vec<String>,
    pub total_plan_files: usize,
    pub plans_with_uuids: usize,
    pub plans_missing_uuids: Vec<String>,
    pub orphaned_recipe_refs: Vec<String>,
    pub errors: Vec<String>,
}

impl ValidationReport {
    /// Returns true if the migration validation passed
    pub fn is_valid(&self) -> bool {
        self.day_named_files.is_empty()
            && self.recipes_missing_uuid.is_empty()
            && self.recipes_with_invalid_uuid.is_empty()
            && self.plans_missing_uuids.is_empty()
            && self.orphaned_recipe_refs.is_empty()
            && self.errors.is_empty()
    }

    /// Returns a human-readable summary of the validation
    pub fn summary(&self) -> String {
        let mut output = String::new();
        output.push_str("Migration Validation Report\n");
        output.push_str("===========================\n\n");

        // Recipe validation
        output.push_str("Recipe Files:\n");
        output.push_str(&format!(
            "  Total files found: {}\n",
            self.total_recipe_files
        ));
        output.push_str(&format!("  Files with UUID: {}\n", self.recipes_with_uuid));
        output.push_str(&format!(
            "  Files with UUID-based names: {}\n",
            self.recipes_with_uuid_name
        ));

        if !self.day_named_files.is_empty() {
            output.push_str(&format!(
                "\n  ⚠ {} files still have day-based names:\n",
                self.day_named_files.len()
            ));
            for file in &self.day_named_files {
                output.push_str(&format!("    - {}\n", file));
            }
        }

        if !self.recipes_missing_uuid.is_empty() {
            output.push_str(&format!(
                "\n  ⚠ {} files missing UUID:\n",
                self.recipes_missing_uuid.len()
            ));
            for file in &self.recipes_missing_uuid {
                output.push_str(&format!("    - {}\n", file));
            }
        }

        if !self.recipes_with_invalid_uuid.is_empty() {
            output.push_str(&format!(
                "\n  ⚠ {} files with invalid UUID:\n",
                self.recipes_with_invalid_uuid.len()
            ));
            for file in &self.recipes_with_invalid_uuid {
                output.push_str(&format!("    - {}\n", file));
            }
        }

        // Plan validation
        output.push_str("\nPlan Files:\n");
        output.push_str(&format!("  Total files found: {}\n", self.total_plan_files));
        output.push_str(&format!(
            "  Files with Recipe UUIDs: {}\n",
            self.plans_with_uuids
        ));

        if !self.plans_missing_uuids.is_empty() {
            output.push_str(&format!(
                "\n  ⚠ {} files missing Recipe UUIDs:\n",
                self.plans_missing_uuids.len()
            ));
            for file in &self.plans_missing_uuids {
                output.push_str(&format!("    - {}\n", file));
            }
        }

        if !self.orphaned_recipe_refs.is_empty() {
            output.push_str(&format!(
                "\n  ⚠ {} recipe UUIDs reference non-existent files:\n",
                self.orphaned_recipe_refs.len()
            ));
            for uuid in &self.orphaned_recipe_refs {
                output.push_str(&format!("    - {}\n", uuid));
            }
        }

        // Errors
        if !self.errors.is_empty() {
            output.push_str(&format!("\nErrors ({}):\n", self.errors.len()));
            for error in &self.errors {
                output.push_str(&format!("  ⚠ {}\n", error));
            }
        }

        // Overall result
        output.push('\n');
        if self.is_valid() {
            output.push_str("✓ Migration validation PASSED\n");
            output.push_str("  All recipe and plan files are properly migrated.\n");
        } else {
            output.push_str("✗ Migration validation FAILED\n");
            output.push_str("  Some files require attention.\n");
        }

        output
    }
}

impl MigrationValidator {
    /// Creates a new MigrationValidator for the given content directory
    pub fn new<P: AsRef<Path>>(content_dir: P) -> Result<Self, String> {
        let content_dir = content_dir.as_ref().to_path_buf();

        if !content_dir.exists() {
            return Err(format!(
                "Content directory does not exist: {}",
                content_dir.display()
            ));
        }

        Ok(Self { content_dir })
    }

    /// Validates the migration and returns a report
    pub fn validate(&self) -> Result<ValidationReport, String> {
        let mut report = ValidationReport::default();

        // Collect all recipe UUIDs for cross-reference validation
        let recipe_uuids = self.collect_recipe_uuids(&mut report)?;

        // Validate recipes
        self.validate_recipes(&mut report)?;

        // Validate plans
        self.validate_plans(&mut report, &recipe_uuids)?;

        Ok(report)
    }

    /// Collects all valid recipe UUIDs from recipe files
    fn collect_recipe_uuids(&self, report: &mut ValidationReport) -> Result<HashSet<Uuid>, String> {
        let mut uuids = HashSet::new();

        let entries = fs::read_dir(&self.content_dir)
            .map_err(|e| format!("Failed to read content directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Check for UUID-named files
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
                && path.extension().and_then(|s| s.to_str()) == Some("md")
            {
                // Try to parse filename as UUID
                if let Ok(uuid) = Uuid::parse_str(file_name) {
                    // Verify the file content has the same UUID
                    match self.extract_uuid_from_content(&path) {
                        Ok(content_uuid) => {
                            if content_uuid == uuid {
                                uuids.insert(uuid);
                            } else {
                                report.errors.push(format!(
                                    "File {} has mismatched UUID: filename={}, content={}",
                                    file_name, uuid, content_uuid
                                ));
                            }
                        }
                        Err(_) => {
                            // Will be caught in validate_recipes
                        }
                    }
                }
            }
        }

        Ok(uuids)
    }

    /// Validates all recipe files
    fn validate_recipes(&self, report: &mut ValidationReport) -> Result<(), String> {
        let entries = fs::read_dir(&self.content_dir)
            .map_err(|e| format!("Failed to read content directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>");

            // Check if this is a recipe file (either day-*.md or UUID.md)
            let is_day_file = file_name.starts_with("day-");
            let is_uuid_file = Uuid::parse_str(file_name).is_ok();
            let is_week_file = file_name.starts_with("week-");

            // Only process recipe files (not plan files)
            if (!is_day_file && !is_uuid_file) || is_week_file {
                continue;
            }

            report.total_recipe_files += 1;

            // Check if file has day-based name
            if is_day_file {
                report.day_named_files.push(file_name.to_string());
            }

            // Check if file has UUID-based name
            if is_uuid_file {
                report.recipes_with_uuid_name += 1;
            }

            // Check if content has UUID
            match self.extract_uuid_from_content(&path) {
                Ok(_uuid) => {
                    report.recipes_with_uuid += 1;
                }
                Err(e) => {
                    if e.contains("No UUID found") {
                        report.recipes_missing_uuid.push(file_name.to_string());
                    } else {
                        report
                            .recipes_with_invalid_uuid
                            .push(format!("{}: {}", file_name, e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates all plan files
    fn validate_plans(
        &self,
        report: &mut ValidationReport,
        recipe_uuids: &HashSet<Uuid>,
    ) -> Result<(), String> {
        let entries = fs::read_dir(&self.content_dir)
            .map_err(|e| format!("Failed to read content directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Only process week-*.md files
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
                && file_name.starts_with("week-")
                && path.extension().and_then(|s| s.to_str()) == Some("md")
            {
                report.total_plan_files += 1;

                match self.extract_plan_uuids(&path) {
                    Ok(plan_uuids) => {
                        report.plans_with_uuids += 1;

                        // Check if all plan UUIDs reference existing recipes
                        for uuid in plan_uuids {
                            if !recipe_uuids.contains(&uuid) {
                                report.orphaned_recipe_refs.push(format!(
                                    "{} references non-existent recipe {}",
                                    file_name, uuid
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        if e.contains("No Recipe UUIDs found") {
                            report.plans_missing_uuids.push(file_name.to_string());
                        } else {
                            report
                                .errors
                                .push(format!("Error processing {}: {}", file_name, e));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extracts the UUID from a recipe file's content
    fn extract_uuid_from_content(&self, path: &Path) -> Result<Uuid, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(uuid_str) = trimmed.strip_prefix("UUID:") {
                let uuid_str = uuid_str.trim();
                return Uuid::parse_str(uuid_str)
                    .map_err(|e| format!("Invalid UUID format: {}", e));
            }
        }

        Err("No UUID found in file".to_string())
    }

    /// Extracts recipe UUIDs from a plan file's content
    fn extract_plan_uuids(&self, path: &Path) -> Result<Vec<Uuid>, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(uuids_str) = trimmed.strip_prefix("Recipe UUIDs:") {
                let uuids: Result<Vec<Uuid>, _> = uuids_str
                    .split(',')
                    .map(|s| Uuid::parse_str(s.trim()))
                    .collect();

                return uuids.map_err(|e| format!("Failed to parse recipe UUIDs: {}", e));
            }
        }

        Err("No Recipe UUIDs found in plan file".to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Migration Validation Tool");
    println!("=========================\n");

    let content_dir = "../content";

    // Check if content directory exists
    if !Path::new(content_dir).exists() {
        eprintln!("Error: Content directory '{}' does not exist", content_dir);
        eprintln!("Please run this tool from the cookbook/recipe_gen directory");
        std::process::exit(1);
    }

    println!("Validating migration in '{}'...\n", content_dir);

    let validator = MigrationValidator::new(content_dir)?;
    let report = validator.validate()?;

    println!("{}", report.summary());

    if !report.is_valid() {
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_validation_passes_with_fully_migrated_content() {
        let temp_dir = TempDir::new().unwrap();

        // Create properly migrated recipe files
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        for uuid in [uuid1, uuid2] {
            let recipe_path = temp_dir.path().join(format!("{}.md", uuid));
            let mut file = fs::File::create(&recipe_path).unwrap();
            writeln!(file, "# Recipe Title").unwrap();
            writeln!(file).unwrap();
            writeln!(file, "UUID: {}", uuid).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "Description").unwrap();
        }

        // Create properly migrated plan file
        let plan_path = temp_dir.path().join("week-1.md");
        let mut file = fs::File::create(&plan_path).unwrap();
        writeln!(file, "# Week 1 Plan").unwrap();
        writeln!(file, "Week: 1").unwrap();
        writeln!(file, "Days: 1, 2").unwrap();
        writeln!(file, "Recipe UUIDs: {}, {}", uuid1, uuid2).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(report.is_valid());
        assert_eq!(report.total_recipe_files, 2);
        assert_eq!(report.recipes_with_uuid, 2);
        assert_eq!(report.recipes_with_uuid_name, 2);
        assert_eq!(report.total_plan_files, 1);
        assert_eq!(report.plans_with_uuids, 1);
        assert!(report.day_named_files.is_empty());
        assert!(report.recipes_missing_uuid.is_empty());
        assert!(report.plans_missing_uuids.is_empty());
        assert!(report.orphaned_recipe_refs.is_empty());
    }

    #[test]
    fn test_validation_detects_day_named_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a day-named file
        let recipe_path = temp_dir.path().join("day-1.md");
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "UUID: {}", Uuid::new_v4()).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.day_named_files.len(), 1);
        assert_eq!(report.day_named_files[0], "day-1");
    }

    #[test]
    fn test_validation_detects_missing_uuid() {
        let temp_dir = TempDir::new().unwrap();

        let uuid = Uuid::new_v4();
        let recipe_path = temp_dir.path().join(format!("{}.md", uuid));
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "Description without UUID").unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.recipes_missing_uuid.len(), 1);
    }

    #[test]
    fn test_validation_detects_plans_missing_uuids() {
        let temp_dir = TempDir::new().unwrap();

        // Create a plan without Recipe UUIDs
        let plan_path = temp_dir.path().join("week-1.md");
        let mut file = fs::File::create(&plan_path).unwrap();
        writeln!(file, "# Week 1 Plan").unwrap();
        writeln!(file, "Week: 1").unwrap();
        writeln!(file, "Days: 1, 2, 3").unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.plans_missing_uuids.len(), 1);
        assert_eq!(report.plans_missing_uuids[0], "week-1");
    }

    #[test]
    fn test_validation_detects_orphaned_recipe_refs() {
        let temp_dir = TempDir::new().unwrap();

        // Create a recipe
        let uuid1 = Uuid::new_v4();
        let recipe_path = temp_dir.path().join(format!("{}.md", uuid1));
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "UUID: {}", uuid1).unwrap();

        // Create a plan referencing a non-existent recipe
        let uuid2 = Uuid::new_v4();
        let plan_path = temp_dir.path().join("week-1.md");
        let mut file = fs::File::create(&plan_path).unwrap();
        writeln!(file, "# Week 1 Plan").unwrap();
        writeln!(file, "Week: 1").unwrap();
        writeln!(file, "Days: 1, 2").unwrap();
        writeln!(file, "Recipe UUIDs: {}, {}", uuid1, uuid2).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert_eq!(report.orphaned_recipe_refs.len(), 1);
        assert!(report.orphaned_recipe_refs[0].contains(&uuid2.to_string()));
    }

    #[test]
    fn test_validation_detects_mismatched_uuids() {
        let temp_dir = TempDir::new().unwrap();

        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        // Create a file where filename UUID doesn't match content UUID
        let recipe_path = temp_dir.path().join(format!("{}.md", uuid1));
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "UUID: {}", uuid2).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(!report.is_valid());
        assert!(!report.errors.is_empty());
        assert!(report.errors[0].contains("mismatched UUID"));
    }

    #[test]
    fn test_validation_ignores_non_recipe_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create some non-recipe files
        let readme_path = temp_dir.path().join("README.md");
        fs::write(&readme_path, "# README").unwrap();

        let intro_path = temp_dir.path().join("intro.md");
        fs::write(&intro_path, "# Introduction").unwrap();

        // Create a valid recipe
        let uuid = Uuid::new_v4();
        let recipe_path = temp_dir.path().join(format!("{}.md", uuid));
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "UUID: {}", uuid).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        assert!(report.is_valid());
        assert_eq!(report.total_recipe_files, 1);
        assert_eq!(report.total_plan_files, 0);
    }

    #[test]
    fn test_validation_summary_formatting() {
        let temp_dir = TempDir::new().unwrap();

        // Create a day-named file to trigger validation failure
        let recipe_path = temp_dir.path().join("day-1.md");
        let mut file = fs::File::create(&recipe_path).unwrap();
        writeln!(file, "# Recipe Title").unwrap();
        writeln!(file, "UUID: {}", Uuid::new_v4()).unwrap();

        let validator = MigrationValidator::new(temp_dir.path()).unwrap();
        let report = validator.validate().unwrap();

        let summary = report.summary();

        assert!(summary.contains("Migration Validation Report"));
        assert!(summary.contains("Recipe Files:"));
        assert!(summary.contains("Plan Files:"));
        assert!(summary.contains("✗ Migration validation FAILED"));
        assert!(summary.contains("day-1"));
    }
}
