//! Checks pane's Worker configuration: it serves the Dioxus build as static
//! assets, runs only for /api/*, and stays on the free plan.
use std::fs;
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn wrangler() -> Result<toml::Table> {
    Ok(fs::read_to_string("wrangler.toml")?.parse()?)
}

#[test]
fn test_required_files_exist() -> Result {
    for file in [
        "README.md",
        "wrangler.toml",
        "mise.toml",
        ".gitignore",
        "Cargo.lock",
    ] {
        assert!(Path::new(file).exists(), "required file '{file}' not found");
    }
    Ok(())
}

#[test]
fn test_worker_is_named_for_the_repo() -> Result {
    let config = wrangler()?;
    assert_eq!(config["name"].as_str(), Some("jimhillr-pane"));
    assert_eq!(config["main"].as_str(), Some("build/worker/shim.mjs"));
    assert!(config.contains_key("compatibility_date"));
    assert_eq!(config["workers_dev"].as_bool(), Some(true));
    Ok(())
}

#[test]
fn test_is_a_worker_not_a_pages_project() -> Result {
    let config = wrangler()?;
    assert!(
        !config.contains_key("pages_build_output_dir"),
        "pane is a Worker with static assets now, not a Pages project"
    );
    Ok(())
}

#[test]
fn test_assets_serve_the_web_build_and_only_api_runs_the_worker() -> Result {
    let config = wrangler()?;
    let assets = config["assets"].as_table().ok_or("[assets] table")?;
    assert_eq!(assets["directory"].as_str(), Some("../web/dist/public"));
    assert_eq!(assets["binding"].as_str(), Some("ASSETS"));
    assert_eq!(
        assets["not_found_handling"].as_str(),
        Some("single-page-application"),
        "Dioxus routes are client side, so unknown paths get index.html"
    );
    let first: Vec<&str> = assets["run_worker_first"]
        .as_array()
        .ok_or("run_worker_first is a list")?
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(first, ["/api/*"], "only API calls count as Worker requests");
    Ok(())
}

#[test]
fn test_stays_on_the_free_plan() -> Result {
    let config = wrangler()?;
    for key in ["usage_model", "limits", "logpush"] {
        assert!(!config.contains_key(key), "{key} needs Workers Paid");
    }
    Ok(())
}

#[test]
fn test_build_runs_worker_build() -> Result {
    let config = wrangler()?;
    let command = config["build"]["command"]
        .as_str()
        .ok_or("[build] command")?;
    assert!(command.contains("worker-build --release"));
    Ok(())
}

#[test]
fn test_mise_toml_uses_worker_assets_tasks_and_builds_the_web_app() -> Result {
    let content = fs::read_to_string("mise.toml")?;
    let config: toml::Table = content.parse()?;
    assert!(content.contains("standards/tasks/cloudflare-worker-assets.toml"));
    assert_eq!(
        config["env"]["DEPLOY_URL"].as_str(),
        Some("https://jimhillr-pane.jim-hill-r.workers.dev")
    );
    let release = config["tasks"]["release:assets"]
        .as_table()
        .ok_or("release:assets task")?;
    assert_eq!(release["run"].as_str(), Some("mise run release"));
    assert!(
        content
            .lines()
            .any(|l| l.starts_with("# deploy-on-every-push")),
        "pane's snapshot reads the whole repo, so it deploys on every push"
    );
    Ok(())
}

#[test]
fn test_gitignore_has_cloudflare_entries() -> Result {
    let content = fs::read_to_string(".gitignore")?;
    for entry in [".wrangler/", ".dev.vars", "build/"] {
        assert!(content.contains(entry), ".gitignore missing {entry}");
    }
    Ok(())
}
