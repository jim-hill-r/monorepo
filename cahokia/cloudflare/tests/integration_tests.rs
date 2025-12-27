use std::fs;
use std::path::Path;

#[test]
fn test_required_files_exist() {
    let required_files = [
        "README.md",
        "wrangler.toml",
        "Cast.toml",
        ".gitignore",
        "ISSUES.md",
    ];

    for file in &required_files {
        assert!(
            Path::new(file).exists(),
            "Required file '{}' not found",
            file
        );
    }
}

#[test]
fn test_wrangler_toml_has_required_fields() {
    let content = fs::read_to_string("wrangler.toml").expect("Failed to read wrangler.toml");

    assert!(
        content.contains("name = \"cahokia-web\""),
        "wrangler.toml missing project name"
    );

    assert!(
        content.contains("compatibility_date"),
        "wrangler.toml missing compatibility_date"
    );
}

#[test]
fn test_readme_has_required_sections() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let required_sections = [
        "Prerequisites",
        "Building Cahokia Web",
        "Deploying to Cloudflare Pages",
    ];

    for section in &required_sections {
        assert!(
            content.contains(section),
            "README missing '{}' section",
            section
        );
    }
}

#[test]
fn test_gitignore_has_cloudflare_entries() {
    let content = fs::read_to_string(".gitignore").expect("Failed to read .gitignore");

    assert!(
        content.contains(".wrangler/"),
        ".gitignore missing .wrangler/ entry"
    );
}

#[test]
fn test_cahokia_web_project_exists() {
    let cahokia_web_dir = Path::new("../web");

    assert!(
        cahokia_web_dir.exists() && cahokia_web_dir.is_dir(),
        "cahokia/web project not found at ../web"
    );

    let cargo_toml = cahokia_web_dir.join("Cargo.toml");
    assert!(
        cargo_toml.exists(),
        "cahokia/web project missing Cargo.toml"
    );
}

#[test]
fn test_readme_does_not_recommend_cargo_install_wrangler() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    assert!(
        !content.contains("cargo install wrangler"),
        "README mentions 'cargo install wrangler' which is no longer supported"
    );
}

#[test]
fn test_cast_toml_has_framework_field() {
    let content = fs::read_to_string("Cast.toml").expect("Failed to read Cast.toml");

    assert!(
        content.contains("framework = \"cloudflare-pages\""),
        "Cast.toml missing framework = \"cloudflare-pages\""
    );
}
