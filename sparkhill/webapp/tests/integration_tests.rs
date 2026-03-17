use std::path::Path;

#[test]
fn test_project_structure_exists() {
    assert!(Path::new("Cargo.toml").exists(), "Cargo.toml should exist");
    assert!(Path::new("README.md").exists(), "README.md should exist");
    assert!(
        Path::new("src/main.rs").exists(),
        "src/main.rs should exist"
    );
    assert!(Path::new(".gitignore").exists(), ".gitignore should exist");
}

#[test]
fn test_multi_module_source_structure() {
    assert!(
        Path::new("src/state.rs").exists(),
        "src/state.rs should exist"
    );
    assert!(
        Path::new("src/canvas_js.rs").exists(),
        "src/canvas_js.rs should exist"
    );
    assert!(
        Path::new("src/pages/mod.rs").exists(),
        "src/pages/mod.rs should exist"
    );
    assert!(
        Path::new("src/components/mod.rs").exists(),
        "src/components/mod.rs should exist"
    );
    assert!(
        Path::new("src/pages/home.rs").exists(),
        "src/pages/home.rs should exist"
    );
    assert!(
        Path::new("src/pages/letter.rs").exists(),
        "src/pages/letter.rs should exist"
    );
    assert!(
        Path::new("src/pages/word.rs").exists(),
        "src/pages/word.rs should exist"
    );
    assert!(
        Path::new("src/pages/congratulations.rs").exists(),
        "src/pages/congratulations.rs should exist"
    );
    assert!(
        Path::new("src/components/practice.rs").exists(),
        "src/components/practice.rs should exist"
    );
}

#[test]
fn test_cargo_toml_has_correct_name() {
    let cargo_content = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    assert!(
        cargo_content.contains("name = \"sparkhill_webapp\""),
        "Cargo.toml should contain correct package name"
    );
}

#[test]
fn test_cargo_toml_uses_dioxus_with_router() {
    let cargo_content = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    assert!(
        cargo_content.contains("dioxus"),
        "Cargo.toml should include dioxus dependency"
    );
    assert!(
        cargo_content.contains("router"),
        "Cargo.toml should enable the dioxus router feature"
    );
}

#[test]
fn test_readme_describes_purpose() {
    let readme_content = std::fs::read_to_string("README.md").expect("Failed to read README.md");
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
fn test_main_rs_uses_router() {
    let main_content = std::fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");
    assert!(
        main_content.contains("Routable"),
        "main.rs should use Dioxus router with Routable"
    );
    assert!(
        main_content.contains("Router::<Route>"),
        "main.rs should render a Router component"
    );
}

#[test]
fn test_main_rs_has_four_routes() {
    let main_content = std::fs::read_to_string("src/main.rs").expect("Failed to read src/main.rs");
    assert!(
        main_content.contains("Home {}"),
        "main.rs should define a Home route"
    );
    assert!(
        main_content.contains("Letter {}"),
        "main.rs should define a Letter route"
    );
    assert!(
        main_content.contains("Word {}"),
        "main.rs should define a Word route"
    );
    assert!(
        main_content.contains("Congratulations {}"),
        "main.rs should define a Congratulations route"
    );
}

#[test]
fn test_home_page_matches_blue_eel() {
    let home_content =
        std::fs::read_to_string("src/pages/home.rs").expect("Failed to read home.rs");
    assert!(
        home_content.contains("Blue Eel"),
        "Home page should display 'Blue Eel' title"
    );
    assert!(
        home_content.contains("Writing made simple!"),
        "Home page should display 'Writing made simple!' subtitle"
    );
    assert!(
        home_content.contains("Begin"),
        "Home page should have 'Begin' button"
    );
}

#[test]
fn test_state_has_letter_sequences() {
    let state_content = std::fs::read_to_string("src/state.rs").expect("Failed to read state.rs");
    assert!(
        state_content.contains("\"b\"") && state_content.contains("\"c\""),
        "state.rs should include letter sequences"
    );
    assert!(
        state_content.contains("RETRY_LIMIT"),
        "state.rs should define retry limit constant"
    );
    assert!(
        state_content.contains("STABILIZE_COUNT"),
        "state.rs should define stabilize count constant"
    );
}

#[test]
fn test_canvas_js_has_guidelines() {
    let canvas_content =
        std::fs::read_to_string("src/canvas_js.rs").expect("Failed to read canvas_js.rs");
    assert!(
        canvas_content.contains("#178CA4"),
        "canvas_js.rs should use #178CA4 as user stroke color"
    );
    assert!(
        canvas_content.contains("#F9F7F0"),
        "canvas_js.rs should use #F9F7F0 as canvas background"
    );
    assert!(
        canvas_content.contains("capLine"),
        "canvas_js.rs should define cap guide line"
    );
    assert!(
        canvas_content.contains("baseLine"),
        "canvas_js.rs should define base guide line"
    );
}

#[test]
fn test_congratulations_page_content() {
    let congrats_content = std::fs::read_to_string("src/pages/congratulations.rs")
        .expect("Failed to read congratulations.rs");
    assert!(
        congrats_content.contains("Congratulations"),
        "Congratulations page should display 'Congratulations'"
    );
    assert!(
        congrats_content.contains("graduated"),
        "Congratulations page should mention graduation"
    );
    assert!(
        congrats_content.contains("Start Over"),
        "Congratulations page should have 'Start Over' button"
    );
}

#[test]
fn test_gitignore_excludes_build_artifacts() {
    let gitignore_content =
        std::fs::read_to_string(".gitignore").expect("Failed to read .gitignore");
    assert!(
        gitignore_content.contains("/target"),
        ".gitignore should exclude target directory"
    );
}
