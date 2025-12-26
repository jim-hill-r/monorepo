use std::fs;
use std::path::{Path, PathBuf};

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

/// Find all directories containing package.json files (TypeScript/Node.js projects)
fn find_typescript_projects() -> Vec<PathBuf> {
    let mut projects = Vec::new();
    let repo_root = PathBuf::from("..")
        .canonicalize()
        .expect("Failed to get repo root");

    // Walk through the repository to find package.json files
    if let Ok(entries) = fs::read_dir(&repo_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                check_dir_for_package_json(&path, &mut projects);
            }
        }
    }

    projects
}

/// Recursively check a directory for package.json files
fn check_dir_for_package_json(dir: &Path, projects: &mut Vec<PathBuf>) {
    // Skip common directories that don't need checking
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        if name == "node_modules" || name == "target" || name == ".git" {
            return;
        }
    }

    let package_json = dir.join("package.json");
    if package_json.exists() {
        projects.push(dir.to_path_buf());
    }

    // Recursively check subdirectories (limit depth to avoid deep traversal)
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                check_dir_for_package_json(&path, projects);
            }
        }
    }
}

/// Check if a tsconfig.json has strict mode enabled
fn validate_tsconfig_strict(tsconfig_path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(tsconfig_path)
        .map_err(|e| format!("Failed to read tsconfig.json: {}", e))?;

    // Simple check: look for "strict": true in the file
    // This is more robust than trying to parse JSONC (JSON with comments)
    // which TypeScript allows in tsconfig.json files
    let strict_pattern_true = r#""strict": true"#;
    let strict_pattern_false = r#""strict": false"#;

    if content.contains(strict_pattern_false) {
        return Err(
            "tsconfig.json must have 'compilerOptions.strict' set to true, not false".to_string(),
        );
    }

    if !content.contains(strict_pattern_true) {
        return Err("tsconfig.json must have '\"strict\": true' in compilerOptions".to_string());
    }

    Ok(())
}

#[test]
fn test_typescript_projects_have_tsconfig() {
    let projects = find_typescript_projects();
    let mut missing_tsconfig = Vec::new();
    let mut invalid_tsconfig = Vec::new();

    for project in &projects {
        let tsconfig = project.join("tsconfig.json");
        if !tsconfig.exists() {
            missing_tsconfig.push(project.clone());
        } else {
            // Validate the tsconfig has strict mode enabled
            if let Err(e) = validate_tsconfig_strict(&tsconfig) {
                invalid_tsconfig.push((project.clone(), e));
            }
        }
    }

    if !missing_tsconfig.is_empty() {
        let paths: Vec<String> = missing_tsconfig
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        panic!(
            "The following TypeScript projects are missing tsconfig.json:\n{}",
            paths.join("\n")
        );
    }

    if !invalid_tsconfig.is_empty() {
        let errors: Vec<String> = invalid_tsconfig
            .iter()
            .map(|(p, e)| format!("{}: {}", p.display(), e))
            .collect();
        panic!(
            "The following TypeScript projects have invalid tsconfig.json:\n{}",
            errors.join("\n")
        );
    }
}

#[test]
fn test_typescript_standard_exists() {
    let typescript_standard = Path::new("docs/typescript.md");
    assert!(
        typescript_standard.exists(),
        "TypeScript standard documentation must exist at docs/typescript.md"
    );

    let content = fs::read_to_string(typescript_standard).expect("Failed to read typescript.md");

    // Verify the standard includes required sections
    assert!(
        content.contains("strict"),
        "Standard must mention strict mode"
    );
    assert!(
        content.contains("tsconfig.json"),
        "Standard must mention tsconfig.json"
    );
}
