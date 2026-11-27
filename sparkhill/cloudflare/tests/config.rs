//! Checks the deploy config: the Pages project name, that it uploads the
//! website's static export, and that the release task builds that export.

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {path}: {e}"))
}

#[test]
fn pages_project_follows_the_naming_rule() {
    // standards/docs/naming.md: every Pages project is jimhillr-<project>.
    let wrangler = read("wrangler.toml");
    assert!(wrangler.contains("name = \"jimhillr-sparkhill\""));
    assert!(read("mise.toml").contains("DEPLOY_URL = \"https://jimhillr-sparkhill.pages.dev\""));
}

#[test]
fn deploys_the_website_static_export() {
    let wrangler = read("wrangler.toml");
    assert!(wrangler.contains("pages_build_output_dir = \"../website/dist/exported\""));
    // The release task runs the website's own release, a `perseus export`.
    let mise = read("mise.toml");
    assert!(mise.contains("dir = \"{{config_root}}/../website\""));
    assert!(read("../website/mise.toml").contains("perseus-web.toml"));
}

#[test]
fn website_needs_no_server() {
    // The site is static files on Pages, so it must stay export-only: no
    // server integration, and the export entrypoint.
    let cargo = read("../website/Cargo.toml");
    assert!(
        !cargo.contains("perseus-axum"),
        "the website must not need a server"
    );
    assert!(read("../website/src/main.rs").contains("#[perseus::main_export]"));
}

#[test]
fn free_plan_settings_only() {
    // Pages Functions would count against the Workers free quota; this
    // site is static. Paid-only settings are also refused by FT-005.
    let wrangler = read("wrangler.toml");
    for setting in ["usage_model", "[limits]", "logpush"] {
        assert!(!wrangler.contains(setting), "wrangler.toml has {setting}");
    }
}

#[test]
fn ignores_wrangler_state() {
    assert!(read(".gitignore").contains(".wrangler/"));
}
