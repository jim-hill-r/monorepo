use cookbook_core::RecipeReader;
use cookbook_data_md::MarkdownRecipeStore;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Migrates plan markdown files to use UUID-based recipe references
///
/// This tool reads all plan markdown files (week-*.md) in the content directory,
/// extracts the day-based recipe references, looks up the corresponding recipe UUIDs,
/// and updates the plan files to include UUID-based references.
pub struct PlanUuidMigrator {
    content_dir: PathBuf,
}

#[derive(Debug)]
pub struct MigrationStats {
    pub total_files: usize,
    pub already_had_uuids: usize,
    pub added_uuids: usize,
    pub failed: usize,
}

impl PlanUuidMigrator {
    /// Creates a new PlanUuidMigrator for the given content directory
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

    /// Runs the migration on all plan files in the content directory
    pub fn migrate(&self) -> Result<MigrationStats, String> {
        let mut stats = MigrationStats {
            total_files: 0,
            already_had_uuids: 0,
            added_uuids: 0,
            failed: 0,
        };

        // Load recipe store to look up UUIDs by day
        let recipe_store = MarkdownRecipeStore::new(&self.content_dir)
            .map_err(|e| format!("Failed to load recipe store: {}", e))?;

        // Get all week-*.md files
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
                stats.total_files += 1;
                match self.migrate_file(&path, &recipe_store) {
                    Ok(true) => stats.added_uuids += 1,
                    Ok(false) => stats.already_had_uuids += 1,
                    Err(e) => {
                        eprintln!("Error migrating {}: {}", path.display(), e);
                        stats.failed += 1;
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Migrates a single plan file to include UUID-based recipe references
    /// Returns Ok(true) if UUIDs were added, Ok(false) if UUIDs already existed
    fn migrate_file(
        &self,
        path: &Path,
        recipe_store: &MarkdownRecipeStore,
    ) -> Result<bool, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Check if UUIDs already exist in the content
        if self.has_recipe_uuids(&content) {
            return Ok(false);
        }

        // Extract day numbers from the plan
        let days = self.extract_days(&content)?;

        // Look up recipe UUIDs for each day
        let mut recipe_uuids = Vec::new();
        for day in &days {
            let recipe = recipe_store
                .get_by_day(*day)
                .map_err(|e| format!("Failed to find recipe for day {}: {}", day, e))?;
            recipe_uuids.push(recipe.uuid);
        }

        // Add recipe UUIDs to the content
        let new_content = self.add_recipe_uuids_to_content(&content, &recipe_uuids);

        // Write back to file
        fs::write(path, new_content).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(true)
    }

    /// Checks if the content already has a Recipe UUIDs field
    fn has_recipe_uuids(&self, content: &str) -> bool {
        for line in content.lines() {
            if line.starts_with("Recipe UUIDs:") {
                return true;
            }
        }
        false
    }

    /// Extracts day numbers from the plan content
    fn extract_days(&self, content: &str) -> Result<Vec<u32>, String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(days_str) = trimmed.strip_prefix("Days:") {
                let days: Result<Vec<u32>, _> = days_str
                    .split(',')
                    .map(|s| s.trim().parse::<u32>())
                    .collect();

                return days.map_err(|e| format!("Failed to parse day numbers: {}", e));
            }
        }

        Err("Could not find 'Days:' line in plan markdown".to_string())
    }

    /// Adds recipe UUID references to the plan content
    /// The UUIDs are inserted after the Days line for backward compatibility
    fn add_recipe_uuids_to_content(&self, content: &str, recipe_uuids: &[Uuid]) -> String {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Find the Days line
        let days_index = lines
            .iter()
            .position(|line| line.trim().starts_with("Days:"));

        if let Some(days_idx) = days_index {
            // Format the UUIDs as a comma-separated list
            let uuids_str = recipe_uuids
                .iter()
                .map(|u| u.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            // Insert Recipe UUIDs line after the Days line
            lines.insert(days_idx + 1, format!("Recipe UUIDs: {}", uuids_str));
        } else {
            // If no Days line found, insert at the beginning of metadata section
            // Find the "Week:" line and insert after it
            let week_index = lines
                .iter()
                .position(|line| line.trim().starts_with("Week:"));

            if let Some(week_idx) = week_index {
                let uuids_str = recipe_uuids
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                // Insert after Year line (which should be after Week line)
                let insert_pos = week_idx + 2;
                if insert_pos < lines.len() {
                    lines.insert(insert_pos, format!("Recipe UUIDs: {}", uuids_str));
                } else {
                    lines.push(format!("Recipe UUIDs: {}", uuids_str));
                }
            }
        }

        // Join lines back together
        lines.join("\n")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Plan UUID Migration Tool");
    println!("========================\n");

    let content_dir = "../content";

    // Check if content directory exists
    if !Path::new(content_dir).exists() {
        eprintln!("Error: Content directory '{}' does not exist", content_dir);
        eprintln!("Please run this tool from the cookbook/recipe_gen directory");
        std::process::exit(1);
    }

    println!("Migrating plan files in '{}'...\n", content_dir);
    println!("This tool will add 'Recipe UUIDs:' references to plan files");
    println!("based on the day numbers in each plan.\n");

    let migrator = PlanUuidMigrator::new(content_dir)?;
    let stats = migrator.migrate()?;

    println!("\nMigration Complete!");
    println!("===================");
    println!("Total plan files found: {}", stats.total_files);
    println!("Already had UUIDs:      {}", stats.already_had_uuids);
    println!("UUIDs added:            {}", stats.added_uuids);
    println!("Failed:                 {}", stats.failed);

    if stats.failed > 0 {
        eprintln!("\n⚠ Warning: {} files failed to migrate", stats.failed);
        std::process::exit(1);
    }

    if stats.added_uuids > 0 {
        println!(
            "\n✓ Successfully added recipe UUIDs to {} plan files",
            stats.added_uuids
        );
        println!("\nNext steps:");
        println!("  1. Update plan parser to read 'Recipe UUIDs:' line");
        println!("  2. Update tests to validate UUID-based plan loading");
        println!("  3. Consider deprecating day-based references in future");
    } else {
        println!("\n✓ All plan files already have recipe UUID references");
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
    fn test_has_recipe_uuids_true() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "Week: 1\nDays: 1, 2, 3\nRecipe UUIDs: 123e4567-e89b-12d3-a456-426614174000";
        assert!(migrator.has_recipe_uuids(content));
    }

    #[test]
    fn test_has_recipe_uuids_false() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "Week: 1\nDays: 1, 2, 3";
        assert!(!migrator.has_recipe_uuids(content));
    }

    #[test]
    fn test_extract_days_success() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "Week: 1\nYear: 2024\nDays: 1, 2, 3, 4, 5, 6, 7\n";
        let days = migrator.extract_days(content).unwrap();
        assert_eq!(days, vec![1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_extract_days_missing() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "Week: 1\nYear: 2024\n";
        let result = migrator.extract_days(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_days_invalid_format() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "Week: 1\nYear: 2024\nDays: one, two, three\n";
        let result = migrator.extract_days(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_recipe_uuids_to_content() {
        let migrator = PlanUuidMigrator::new(".").unwrap();
        let content = "# Week 1\n\nWeek: 1\nYear: 2024\nDays: 1, 2, 3\n\n## Details";
        let uuid1 = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let uuid2 = Uuid::parse_str("223e4567-e89b-12d3-a456-426614174000").unwrap();
        let uuid3 = Uuid::parse_str("323e4567-e89b-12d3-a456-426614174000").unwrap();
        let uuids = vec![uuid1, uuid2, uuid3];

        let new_content = migrator.add_recipe_uuids_to_content(content, &uuids);

        assert!(new_content.contains("Recipe UUIDs:"));
        assert!(new_content.contains("123e4567-e89b-12d3-a456-426614174000"));
        assert!(new_content.contains("223e4567-e89b-12d3-a456-426614174000"));
        assert!(new_content.contains("323e4567-e89b-12d3-a456-426614174000"));
        assert!(new_content.contains("Days: 1, 2, 3"));
    }

    #[test]
    fn test_migrate_file_with_recipes() {
        let temp_dir = TempDir::new().unwrap();

        // Create test recipe files with UUIDs
        let uuid1 = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
        let uuid2 = Uuid::parse_str("223e4567-e89b-12d3-a456-426614174000").unwrap();
        let uuid3 = Uuid::parse_str("323e4567-e89b-12d3-a456-426614174000").unwrap();

        for (day, uuid) in [(1, uuid1), (2, uuid2), (3, uuid3)] {
            let recipe_path = temp_dir.path().join(format!("day-{}.md", day));
            let mut file = fs::File::create(&recipe_path).unwrap();
            writeln!(file, "# Recipe {}", day).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "UUID: {}", uuid).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "Description for recipe {}", day).unwrap();
        }

        // Create a test plan file
        let plan_path = temp_dir.path().join("week-1.md");
        let mut file = fs::File::create(&plan_path).unwrap();
        writeln!(file, "# Week 1 Plan").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "Week: 1").unwrap();
        writeln!(file, "Year: 2024").unwrap();
        writeln!(file, "Days: 1, 2, 3").unwrap();

        // Load the recipe store
        let recipe_store = MarkdownRecipeStore::new(temp_dir.path()).unwrap();

        let migrator = PlanUuidMigrator::new(temp_dir.path()).unwrap();

        // First migration should add UUIDs
        let result = migrator.migrate_file(&plan_path, &recipe_store).unwrap();
        assert!(result, "Should return true when UUIDs are added");

        // Verify UUIDs were added
        let content = fs::read_to_string(&plan_path).unwrap();
        assert!(content.contains("Recipe UUIDs:"));
        assert!(content.contains(&uuid1.to_string()));
        assert!(content.contains(&uuid2.to_string()));
        assert!(content.contains(&uuid3.to_string()));

        // Second migration should skip (UUIDs already exist)
        let result = migrator.migrate_file(&plan_path, &recipe_store).unwrap();
        assert!(!result, "Should return false when UUIDs already exist");
    }

    #[test]
    fn test_migrate_all_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test recipe files
        for day in 1..=7 {
            let uuid = Uuid::new_v4();
            let recipe_path = temp_dir.path().join(format!("day-{}.md", day));
            let mut file = fs::File::create(&recipe_path).unwrap();
            writeln!(file, "# Recipe {}", day).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "UUID: {}", uuid).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "Description for recipe {}", day).unwrap();
        }

        // Create test plan files
        let plan1_path = temp_dir.path().join("week-1.md");
        let mut file = fs::File::create(&plan1_path).unwrap();
        writeln!(file, "# Week 1 Plan").unwrap();
        writeln!(file, "Week: 1").unwrap();
        writeln!(file, "Days: 1, 2, 3, 4, 5, 6, 7").unwrap();

        let plan2_path = temp_dir.path().join("week-2.md");
        let mut file = fs::File::create(&plan2_path).unwrap();
        writeln!(file, "# Week 2 Plan").unwrap();
        writeln!(file, "Week: 2").unwrap();
        writeln!(file, "Days: 1, 2, 3, 4, 5, 6, 7").unwrap();

        // Create a non-plan file that should be ignored
        let other_file = temp_dir.path().join("intro.md");
        fs::write(&other_file, "# Introduction\n").unwrap();

        let migrator = PlanUuidMigrator::new(temp_dir.path()).unwrap();
        let stats = migrator.migrate().unwrap();

        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.added_uuids, 2);
        assert_eq!(stats.already_had_uuids, 0);
        assert_eq!(stats.failed, 0);

        // Verify both plan files have UUIDs
        for week in [1, 2] {
            let plan_path = temp_dir.path().join(format!("week-{}.md", week));
            let content = fs::read_to_string(&plan_path).unwrap();
            assert!(
                content.contains("Recipe UUIDs:"),
                "week-{}.md should contain Recipe UUIDs",
                week
            );
        }

        // Verify intro.md was not modified
        let intro_content = fs::read_to_string(&other_file).unwrap();
        assert_eq!(intro_content, "# Introduction\n");
    }
}
