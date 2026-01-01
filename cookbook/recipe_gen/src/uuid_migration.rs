use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Migrates recipe markdown files to include UUID frontmatter
///
/// This tool reads all recipe markdown files (day-*.md) in the content directory,
/// generates a UUID for each recipe (or preserves existing UUIDs), and adds the
/// UUID field to the frontmatter of each file.
pub struct UuidMigrator {
    content_dir: PathBuf,
}

#[derive(Debug)]
pub struct MigrationStats {
    pub total_files: usize,
    pub already_had_uuid: usize,
    pub added_uuid: usize,
    pub failed: usize,
}

impl UuidMigrator {
    /// Creates a new UuidMigrator for the given content directory
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

    /// Runs the migration on all recipe files in the content directory
    pub fn migrate(&self) -> Result<MigrationStats, String> {
        let mut stats = MigrationStats {
            total_files: 0,
            already_had_uuid: 0,
            added_uuid: 0,
            failed: 0,
        };

        // Get all day-*.md files
        let entries = fs::read_dir(&self.content_dir)
            .map_err(|e| format!("Failed to read content directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            // Only process day-*.md files
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
                && file_name.starts_with("day-")
                && path.extension().and_then(|s| s.to_str()) == Some("md")
            {
                stats.total_files += 1;
                match self.migrate_file(&path) {
                    Ok(true) => stats.added_uuid += 1,
                    Ok(false) => stats.already_had_uuid += 1,
                    Err(e) => {
                        eprintln!("Error migrating {}: {}", path.display(), e);
                        stats.failed += 1;
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Migrates a single recipe file to include a UUID
    /// Returns Ok(true) if UUID was added, Ok(false) if UUID already existed
    fn migrate_file(&self, path: &Path) -> Result<bool, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Check if UUID already exists in the content
        if self.has_uuid(&content) {
            return Ok(false);
        }

        // Generate a new UUID
        let uuid = Uuid::new_v4();

        // Add UUID to the content
        let new_content = self.add_uuid_to_content(&content, uuid);

        // Write back to file
        fs::write(path, new_content).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(true)
    }

    /// Checks if the content already has a UUID field
    fn has_uuid(&self, content: &str) -> bool {
        for line in content.lines() {
            if line.starts_with("UUID:") {
                return true;
            }
        }
        false
    }

    /// Adds a UUID field to the recipe content
    /// The UUID is inserted after the title and before the description
    fn add_uuid_to_content(&self, content: &str, uuid: Uuid) -> String {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Find the title line (first line starting with "# ")
        let title_index = lines.iter().position(|line| line.trim().starts_with("# "));

        if let Some(title_idx) = title_index {
            // Insert UUID after the title line
            // We add an empty line, the UUID line, and another empty line for formatting
            lines.insert(title_idx + 1, String::new());
            lines.insert(title_idx + 2, format!("UUID: {}", uuid));
            lines.insert(title_idx + 3, String::new());
        } else {
            // If no title found, insert at the beginning
            lines.insert(0, format!("UUID: {}", uuid));
            lines.insert(1, String::new());
        }

        // Join lines back together
        lines.join("\n")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("UUID Migration Tool");
    println!("===================\n");

    let content_dir = "../content";

    // Check if content directory exists
    if !Path::new(content_dir).exists() {
        eprintln!("Error: Content directory '{}' does not exist", content_dir);
        eprintln!("Please run this tool from the cookbook/recipe_gen directory");
        std::process::exit(1);
    }

    println!("Migrating recipe files in '{}'...\n", content_dir);

    let migrator = UuidMigrator::new(content_dir)?;
    let stats = migrator.migrate()?;

    println!("\nMigration Complete!");
    println!("===================");
    println!("Total recipe files found: {}", stats.total_files);
    println!("Already had UUIDs:        {}", stats.already_had_uuid);
    println!("UUIDs added:              {}", stats.added_uuid);
    println!("Failed:                   {}", stats.failed);

    if stats.failed > 0 {
        eprintln!("\n⚠ Warning: {} files failed to migrate", stats.failed);
        std::process::exit(1);
    }

    if stats.added_uuid > 0 {
        println!(
            "\n✓ Successfully added UUIDs to {} recipe files",
            stats.added_uuid
        );
    } else {
        println!("\n✓ All recipe files already have UUIDs");
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
    fn test_has_uuid_true() {
        let migrator = UuidMigrator::new(".").unwrap();
        let content = "# Recipe Title\n\nUUID: 123e4567-e89b-12d3-a456-426614174000\n\nDescription";
        assert!(migrator.has_uuid(content));
    }

    #[test]
    fn test_has_uuid_false() {
        let migrator = UuidMigrator::new(".").unwrap();
        let content = "# Recipe Title\n\nDescription";
        assert!(!migrator.has_uuid(content));
    }

    #[test]
    fn test_add_uuid_to_content() {
        let migrator = UuidMigrator::new(".").unwrap();
        let content = "# Recipe Title\n\nDescription here\n\n## Ingredients";
        let uuid = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let new_content = migrator.add_uuid_to_content(content, uuid);

        assert!(new_content.contains("UUID: 123e4567-e89b-12d3-a456-426614174000"));
        assert!(new_content.contains("# Recipe Title"));
        assert!(new_content.contains("Description here"));
    }

    #[test]
    fn test_migrate_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("day-1.md");

        // Create a test recipe file
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "# Test Recipe").unwrap();
        writeln!(file).unwrap();
        writeln!(file, "A delicious test recipe").unwrap();

        let migrator = UuidMigrator::new(temp_dir.path()).unwrap();

        // First migration should add UUID
        let result = migrator.migrate_file(&file_path).unwrap();
        assert!(result, "Should return true when UUID is added");

        // Verify UUID was added
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("UUID: "));

        // Second migration should skip (UUID already exists)
        let result = migrator.migrate_file(&file_path).unwrap();
        assert!(!result, "Should return false when UUID already exists");
    }

    #[test]
    fn test_migrate_all_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test recipe files
        for i in 1..=5 {
            let file_path = temp_dir.path().join(format!("day-{}.md", i));
            let mut file = fs::File::create(&file_path).unwrap();
            writeln!(file, "# Recipe {}", i).unwrap();
            writeln!(file).unwrap();
            writeln!(file, "Description for recipe {}", i).unwrap();
        }

        // Create a non-recipe file that should be ignored
        let other_file = temp_dir.path().join("intro.md");
        fs::write(&other_file, "# Introduction\n").unwrap();

        let migrator = UuidMigrator::new(temp_dir.path()).unwrap();
        let stats = migrator.migrate().unwrap();

        assert_eq!(stats.total_files, 5);
        assert_eq!(stats.added_uuid, 5);
        assert_eq!(stats.already_had_uuid, 0);
        assert_eq!(stats.failed, 0);

        // Verify all files have UUIDs
        for i in 1..=5 {
            let file_path = temp_dir.path().join(format!("day-{}.md", i));
            let content = fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("UUID: "));
        }

        // Verify intro.md was not modified
        let intro_content = fs::read_to_string(&other_file).unwrap();
        assert_eq!(intro_content, "# Introduction\n");
    }
}
