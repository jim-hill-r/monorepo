use std::path::Path;

#[test]
fn test_project_structure_exists() {
    // Verify the blueeel project has the expected structure
    assert!(Path::new("Cargo.toml").exists(), "Cargo.toml should exist");
    assert!(Path::new("README.md").exists(), "README.md should exist");
    assert!(Path::new("src/main.rs").exists(), "src/main.rs should exist");
    assert!(Path::new(".gitignore").exists(), ".gitignore should exist");
}

#[test]
fn test_cargo_toml_has_correct_name() {
    let cargo_content = std::fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");
    assert!(
        cargo_content.contains("name = \"blueeel\""),
        "Cargo.toml should contain correct package name"
    );
}

#[test]
fn test_cargo_toml_uses_dioxus() {
    let cargo_content = std::fs::read_to_string("Cargo.toml")
        .expect("Failed to read Cargo.toml");
    assert!(
        cargo_content.contains("dioxus"),
        "Cargo.toml should include dioxus dependency"
    );
}

#[test]
fn test_readme_describes_purpose() {
    let readme_content = std::fs::read_to_string("README.md")
        .expect("Failed to read README.md");
    assert!(
        readme_content.to_lowercase().contains("blue eel"),
        "README should mention Blue Eel"
    );
    assert!(
        readme_content.to_lowercase().contains("reading"),
        "README should mention reading education"
    );
    assert!(
        readme_content.contains("Dioxus"),
        "README should mention Dioxus framework"
    );
}

#[test]
fn test_main_rs_contains_dioxus_app() {
    let main_content = std::fs::read_to_string("src/main.rs")
        .expect("Failed to read src/main.rs");
    assert!(
        main_content.contains("use dioxus::prelude::*"),
        "main.rs should import Dioxus prelude"
    );
    assert!(
        main_content.contains("fn main()"),
        "main.rs should have a main function"
    );
    assert!(
        main_content.contains("dioxus::launch"),
        "main.rs should launch a Dioxus app"
    );
}

#[test]
fn test_app_component_exists() {
    let main_content = std::fs::read_to_string("src/main.rs")
        .expect("Failed to read src/main.rs");
    assert!(
        main_content.contains("#[component]"),
        "main.rs should have a component attribute"
    );
    assert!(
        main_content.contains("fn App()"),
        "main.rs should define an App component"
    );
}

#[test]
fn test_app_contains_educational_content() {
    let main_content = std::fs::read_to_string("src/main.rs")
        .expect("Failed to read src/main.rs");
    // Check for educational theme elements
    assert!(
        main_content.to_lowercase().contains("blue eel") || 
        main_content.to_lowercase().contains("reading"),
        "App should contain educational content related to reading or Blue Eel"
    );
}

#[test]
fn test_gitignore_excludes_build_artifacts() {
    let gitignore_content = std::fs::read_to_string(".gitignore")
        .expect("Failed to read .gitignore");
    assert!(
        gitignore_content.contains("/target"),
        ".gitignore should exclude target directory"
    );
}
