//! Checks the deploy config: the Pages project name, that it uploads the web
//! app's release bundle, and that the release task builds that bundle.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"))
}

#[test]
fn pages_project_follows_the_naming_rule() {
    // standards/docs/naming.md: jimhillr-<project>, plus the sub-project
    // because jimhillr-sparkhill already hosts the website.
    let wrangler = read("wrangler.toml");
    assert!(wrangler.contains("name = \"jimhillr-sparkhill-webapp\""));
    assert!(
        read("mise.toml").contains("DEPLOY_URL = \"https://jimhillr-sparkhill-webapp.pages.dev\"")
    );
}

#[test]
fn deploys_the_webapp_release_bundle() {
    let wrangler = read("wrangler.toml");
    assert!(wrangler.contains("pages_build_output_dir = \"../webapp/dist/public\""));
    // The release task runs the web app's own release, a `dx bundle`.
    let mise = read("mise.toml");
    assert!(mise.contains("dir = \"{{config_root}}/../webapp\""));
    assert!(read("../webapp/mise.toml").contains("dioxus-web.toml"));
}

#[test]
fn webapp_needs_no_outside_host() {
    // The letter patterns are bundled, so the static site is all it needs.
    let source = read("../webapp/src/patterns.rs");
    assert!(source.contains("patterns.json"));
    assert!(!source.contains("amazonaws"));
}

#[test]
fn free_plan_settings_only() {
    // Pages Functions would count against the Workers free quota; this
    // site is static. Paid-only settings are also refused by FT-005.
    let wrangler = read("wrangler.toml");
    for setting in ["usage_model", "[limits]", "logpush"] {
        assert!(!wrangler.contains(setting), "wrangler.toml has {setting}");
    }
    assert!(!std::path::Path::new("functions").exists());
}

#[test]
fn ignores_wrangler_state() {
    assert!(read(".gitignore").contains(".wrangler/"));
}
