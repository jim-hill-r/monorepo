use std::path::Path;

#[test]
fn test_project_structure_exists() {
    // Verify the standards project has the expected structure
    assert!(Path::new("Cargo.toml").exists());
    assert!(Path::new("README.md").exists());
    assert!(Path::new("ISSUES.md").exists());
    assert!(Path::new("src/lib.rs").exists());
}

#[test]
fn test_cargo_toml_has_correct_name() {
    let cargo_content = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    assert!(cargo_content.contains("name = \"standards\""));
}

#[test]
fn test_readme_describes_purpose() {
    let readme_content = std::fs::read_to_string("README.md").expect("Failed to read README.md");
    assert!(readme_content.contains("standards"));
    assert!(readme_content.contains("monorepo"));
}

#[test]
fn test_issues_contains_todos() {
    let issues_content = std::fs::read_to_string("ISSUES.md").expect("Failed to read ISSUES.md");
    assert!(issues_content.contains("TODO"));
    assert!(issues_content.to_lowercase().contains("standards"));
}
