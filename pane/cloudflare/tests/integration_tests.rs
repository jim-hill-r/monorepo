//! Checks pane's Worker configuration: it serves the Dioxus build as static
//! assets, runs only for /api/*, and stays on the free plan.
use std::fs;
use std::path::Path;

fn wrangler() -> toml::Table {
    let content = fs::read_to_string("wrangler.toml").expect("read wrangler.toml");
    content.parse().expect("wrangler.toml is valid TOML")
}

#[test]
fn test_required_files_exist() {
    for file in [
        "README.md",
        "wrangler.toml",
        "mise.toml",
        ".gitignore",
        "Cargo.lock",
    ] {
        assert!(Path::new(file).exists(), "required file '{file}' not found");
    }
}

#[test]
fn test_worker_is_named_for_the_repo() {
    let config = wrangler();
    assert_eq!(config["name"].as_str(), Some("jimhillr-pane"));
    assert_eq!(config["main"].as_str(), Some("build/worker/shim.mjs"));
    assert!(config.contains_key("compatibility_date"));
    assert_eq!(config["workers_dev"].as_bool(), Some(true));
}

#[test]
fn test_is_a_worker_not_a_pages_project() {
    let config = wrangler();
    assert!(
        !config.contains_key("pages_build_output_dir"),
        "pane is a Worker with static assets now, not a Pages project"
    );
}

#[test]
fn test_assets_serve_the_web_build_and_only_api_runs_the_worker() {
    let config = wrangler();
    let assets = config["assets"].as_table().expect("[assets] table");
    assert_eq!(assets["directory"].as_str(), Some("../web/dist/public"));
    assert_eq!(assets["binding"].as_str(), Some("ASSETS"));
    assert_eq!(
        assets["not_found_handling"].as_str(),
        Some("single-page-application"),
        "Dioxus routes are client side, so unknown paths get index.html"
    );
    let first: Vec<&str> = assets["run_worker_first"]
        .as_array()
        .expect("run_worker_first is a list")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(first, ["/api/*"], "only API calls count as Worker requests");
}

#[test]
fn test_stays_on_the_free_plan() {
    let config = wrangler();
    for key in ["usage_model", "limits", "logpush"] {
        assert!(!config.contains_key(key), "{key} needs Workers Paid");
    }
}

#[test]
fn test_build_runs_worker_build() {
    let config = wrangler();
    let command = config["build"]["command"]
        .as_str()
        .expect("[build] command");
    assert!(command.contains("worker-build --release"));
}

#[test]
fn test_mise_toml_uses_worker_assets_tasks_and_builds_the_web_app() {
    let content = fs::read_to_string("mise.toml").expect("read mise.toml");
    let config: toml::Table = content.parse().expect("mise.toml is valid TOML");
    assert!(content.contains("standards/tasks/cloudflare-worker-assets.toml"));
    assert_eq!(
        config["env"]["DEPLOY_URL"].as_str(),
        Some("https://jimhillr-pane.jim-hill-r.workers.dev")
    );
    let release = config["tasks"]["release:assets"]
        .as_table()
        .expect("release:assets task");
    assert_eq!(release["run"].as_str(), Some("mise run release"));
    assert!(
        content
            .lines()
            .any(|l| l.starts_with("# deploy-on-every-push")),
        "pane's snapshot reads the whole repo, so it deploys on every push"
    );
}

#[test]
fn test_gitignore_has_cloudflare_entries() {
    let content = fs::read_to_string(".gitignore").expect("read .gitignore");
    for entry in [".wrangler/", ".dev.vars", "build/"] {
        assert!(content.contains(entry), ".gitignore missing {entry}");
    }
}
