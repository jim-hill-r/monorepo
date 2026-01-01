use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Migrates recipe markdown files from day-based naming (day-X.md) to UUID-based naming ({uuid}.md)
///
/// This tool reads all recipe markdown files in the content directory, extracts the UUID
/// from each file's frontmatter, and renames the file to use the UUID as the filename.
/// This migration is a key step in decoupling recipes from day numbers.
pub struct FilenameRenamer {
    content_dir: PathBuf,
}

#[derive(Debug)]
pub struct RenameStats {
    pub total_files: usize,
    pub renamed: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Debug)]
pub struct RenamePreview {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub uuid: Uuid,
}

impl FilenameRenamer {
    /// Creates a new FilenameRenamer for the given content directory
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

    /// Runs a dry-run preview of the migration without actually renaming files
    /// Returns a list of renames that would be performed
    pub fn preview(&self) -> Result<Vec<RenamePreview>, String> {
        let mut previews = Vec::new();

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
                match self.extract_uuid(&path) {
                    Ok(uuid) => {
                        let new_path = self.content_dir.join(format!("{}.md", uuid));
                        previews.push(RenamePreview {
                            old_path: path,
                            new_path,
                            uuid,
                        });
                    }
                    Err(_e) => {
                        // Skip files without valid UUIDs
                        continue;
                    }
                }
            }
        }

        Ok(previews)
    }

    /// Runs the migration on all recipe files in the content directory
    pub fn migrate(&self) -> Result<RenameStats, String> {
        let mut stats = RenameStats {
            total_files: 0,
            renamed: 0,
            skipped: 0,
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
                match self.rename_file(&path) {
                    Ok(true) => stats.renamed += 1,
                    Ok(false) => stats.skipped += 1,
                    Err(e) => {
                        eprintln!("Error renaming {}: {}", path.display(), e);
                        stats.failed += 1;
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Renames a single recipe file to use its UUID as the filename
    /// Returns Ok(true) if file was renamed, Ok(false) if already has UUID name or skipped
    fn rename_file(&self, path: &Path) -> Result<bool, String> {
        // Extract UUID from the file content
        let uuid = self.extract_uuid(path)?;

        // Build the new path with UUID as filename
        let new_path = self.content_dir.join(format!("{}.md", uuid));

        // Check if target file already exists
        if new_path.exists() {
            return Ok(false);
        }

        // Rename the file
        fs::rename(path, &new_path).map_err(|e| format!("Failed to rename file: {}", e))?;

        Ok(true)
    }

    /// Extracts the UUID from a recipe file's frontmatter
    fn extract_uuid(&self, path: &Path) -> Result<Uuid, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Look for UUID field in the frontmatter
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(uuid_str) = trimmed.strip_prefix("UUID:") {
                let uuid_str = uuid_str.trim();
                return Uuid::parse_str(uuid_str)
                    .map_err(|e| format!("Invalid UUID format: {}", e));
            }
        }

        Err(format!(
            "No UUID found in file: {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Recipe Filename Migration Tool");
    println!("==============================\n");

    let content_dir = "../content";

    // Check if content directory exists
    if !Path::new(content_dir).exists() {
        eprintln!("Error: Content directory '{}' does not exist", content_dir);
        eprintln!("Please run this tool from the cookbook/recipe_gen directory");
        std::process::exit(1);
    }

    println!("This tool will rename recipe files from 'day-X.md' to '{{uuid}}.md'");
    println!("All files must have UUID frontmatter before running this migration.\n");

    let renamer = FilenameRenamer::new(content_dir)?;

    // First, show a preview
    println!("Running preview to check which files will be renamed...\n");
    let previews = renamer.preview()?;

    if previews.is_empty() {
        println!("No day-*.md files found with valid UUIDs.");
        println!("Please ensure:");
        println!("  1. Recipe files exist in the content directory");
        println!("  2. All recipe files have UUID frontmatter");
        println!("  3. You've run the uuid-migration tool first");
        return Ok(());
    }

    println!("Found {} files to rename:", previews.len());
    println!("\nShowing first 5 examples:");
    for preview in previews.iter().take(5) {
        let old_name = preview
            .old_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let new_name = preview
            .new_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        println!("  {} -> {}", old_name, new_name);
    }

    if previews.len() > 5 {
        println!("  ... and {} more files", previews.len() - 5);
    }

    println!("\n⚠ WARNING: This operation will rename files and cannot be easily undone.");
    println!("⚠ Make sure you have a backup of your content directory!");
    println!("\nType 'yes' to proceed or anything else to cancel: ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "yes" {
        println!("\nMigration cancelled.");
        return Ok(());
    }

    println!("\nProceeding with migration...\n");

    let stats = renamer.migrate()?;

    println!("\nMigration Complete!");
    println!("===================");
    println!("Total recipe files found: {}", stats.total_files);
    println!("Files renamed:            {}", stats.renamed);
    println!("Files skipped:            {}", stats.skipped);
    println!("Failed:                   {}", stats.failed);

    if stats.failed > 0 {
        eprintln!("\n⚠ Warning: {} files failed to migrate", stats.failed);
        std::process::exit(1);
    }

    if stats.renamed > 0 {
        println!(
            "\n✓ Successfully renamed {} recipe files to UUID-based names",
            stats.renamed
        );
        println!("\nNext steps:");
        println!("  1. Update build.rs to generate UUID-based file inclusions");
        println!("  2. Update data parsers to work with UUID filenames");
        println!("  3. Update tests to use UUIDs instead of day-based IDs");
    } else {
        println!("\n✓ All recipe files already have UUID-based names");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_uuid_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("day-1.md");

        let uuid = Uuid::new_v4();
        let content = format!("# Test Recipe\n\nUUID: {}\n\nDescription", uuid);
        fs::write(&file_path, content).unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let extracted = renamer.extract_uuid(&file_path).unwrap();

        assert_eq!(extracted, uuid);
    }

    #[test]
    fn test_extract_uuid_missing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("day-1.md");

        let content = "# Test Recipe\n\nNo UUID here";
        fs::write(&file_path, content).unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let result = renamer.extract_uuid(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_uuid_invalid_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("day-1.md");

        let content = "# Test Recipe\n\nUUID: not-a-valid-uuid\n\nDescription";
        fs::write(&file_path, content).unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let result = renamer.extract_uuid(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_rename_file() {
        let temp_dir = TempDir::new().unwrap();
        let uuid = Uuid::new_v4();
        let old_path = temp_dir.path().join("day-1.md");

        let content = format!("# Test Recipe\n\nUUID: {}\n\nDescription", uuid);
        fs::write(&old_path, &content).unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let result = renamer.rename_file(&old_path).unwrap();

        assert!(result, "Should return true when file is renamed");
        assert!(!old_path.exists(), "Old file should not exist");

        let new_path = temp_dir.path().join(format!("{}.md", uuid));
        assert!(new_path.exists(), "New file should exist");

        let new_content = fs::read_to_string(&new_path).unwrap();
        assert_eq!(new_content, content);
    }

    #[test]
    fn test_rename_file_target_exists() {
        let temp_dir = TempDir::new().unwrap();
        let uuid = Uuid::new_v4();

        // Create both old and new files
        let old_path = temp_dir.path().join("day-1.md");
        let new_path = temp_dir.path().join(format!("{}.md", uuid));

        let content = format!("# Test Recipe\n\nUUID: {}\n\nDescription", uuid);
        fs::write(&old_path, &content).unwrap();
        fs::write(&new_path, "existing content").unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let result = renamer.rename_file(&old_path).unwrap();

        assert!(!result, "Should return false when target exists");
        assert!(old_path.exists(), "Old file should still exist");
    }

    #[test]
    fn test_migrate_all_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test recipe files with UUIDs
        let mut uuids = Vec::new();
        for i in 1..=5 {
            let uuid = Uuid::new_v4();
            uuids.push(uuid);

            let file_path = temp_dir.path().join(format!("day-{}.md", i));
            let content = format!("# Recipe {}\n\nUUID: {}\n\nDescription", i, uuid);
            fs::write(&file_path, content).unwrap();
        }

        // Create a file without UUID (should be skipped/failed)
        let no_uuid_path = temp_dir.path().join("day-99.md");
        fs::write(&no_uuid_path, "# No UUID Recipe\n\nDescription").unwrap();

        // Create a non-day file that should be ignored
        let other_file = temp_dir.path().join("intro.md");
        fs::write(&other_file, "# Introduction\n").unwrap();

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let stats = renamer.migrate().unwrap();

        assert_eq!(stats.total_files, 6); // 5 with UUIDs + 1 without
        assert_eq!(stats.renamed, 5);
        assert!(stats.failed > 0); // File without UUID should fail

        // Verify old files don't exist and new files do
        for (i, uuid) in uuids.iter().enumerate() {
            let old_path = temp_dir.path().join(format!("day-{}.md", i + 1));
            let new_path = temp_dir.path().join(format!("{}.md", uuid));

            assert!(
                !old_path.exists(),
                "Old file day-{}.md should not exist",
                i + 1
            );
            assert!(new_path.exists(), "New file {}.md should exist", uuid);
        }

        // Verify intro.md was not touched
        assert!(other_file.exists());
    }

    #[test]
    fn test_preview() {
        let temp_dir = TempDir::new().unwrap();

        let mut expected_uuids = Vec::new();
        for i in 1..=3 {
            let uuid = Uuid::new_v4();
            expected_uuids.push(uuid);

            let file_path = temp_dir.path().join(format!("day-{}.md", i));
            let content = format!("# Recipe {}\n\nUUID: {}\n\nDescription", i, uuid);
            fs::write(&file_path, content).unwrap();
        }

        let renamer = FilenameRenamer::new(temp_dir.path()).unwrap();
        let previews = renamer.preview().unwrap();

        assert_eq!(previews.len(), 3);

        for preview in &previews {
            assert!(preview.old_path.to_string_lossy().contains("day-"));
            assert!(expected_uuids.contains(&preview.uuid));
            assert_eq!(
                preview.new_path.file_name().unwrap().to_string_lossy(),
                format!("{}.md", preview.uuid)
            );
        }

        // Verify no files were actually renamed
        for i in 1..=3 {
            let old_path = temp_dir.path().join(format!("day-{}.md", i));
            assert!(old_path.exists(), "Preview should not modify files");
        }
    }
}
