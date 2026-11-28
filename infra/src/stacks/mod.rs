//! Stacks: one OpenTofu root module each, synthesized to `out/<name>/`.
//!
//! Each stack has its own state and credentials, so a PR plan for one stack
//! needs only that vendor's read-only token.

pub mod auth0;
pub mod cloudflare;
pub mod github;

use crate::constructs::{Auth0App, WebApp};
use crate::synth::Synth;

/// Every web app on Cloudflare Pages, and what it uses. This is the source
/// of truth for the stacks below; a test checks it against every
/// `wrangler.toml` and `DEPLOY_URL` in the repo.
pub fn web_apps() -> Vec<WebApp> {
    vec![
        WebApp::new("bear_down").web_analytics(),
        WebApp::new("cahokia")
            .sign_in(Auth0App::import(
                "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E",
                "cahokia",
            ))
            .web_analytics(),
        WebApp::new("controlled_chaos").web_analytics(),
        WebApp::new("cookbook")
            .sign_in(Auth0App::import(
                "savzmZnyHcvewGkQX8aaInwPFonC9k2x",
                "cookbook",
            ))
            .web_analytics(),
        WebApp::new("design_system").web_analytics(),
        // pane is not here: it is a Worker with static assets on
        // workers.dev (pane/cloudflare), where Web Analytics can't be
        // auto-installed, and it sits behind Cloudflare Access for one person.
        WebApp::new("sparkhill").web_analytics(),
        // Blue Eel. Web Analytics isn't on yet; that adds a Cloudflare
        // resource, so it waits for its own PR.
        WebApp::new("sparkhill_webapp"),
    ]
}

/// Every stack, synthesized.
pub fn all() -> anyhow::Result<Vec<Synth>> {
    let apps = web_apps();
    Ok(vec![
        auth0::synth(&apps)?,
        cloudflare::synth(&apps)?,
        github::synth()?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// `(pages project, DEPLOY_URL host)` for every `wrangler.toml` in the
    /// repo that deploys a Pages project.
    fn pages_projects_in_repo() -> anyhow::Result<Vec<(String, String)>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let mut found = Vec::new();
        let mut dirs = vec![root.clone()];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let path = entry?.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
                if path.is_dir() {
                    if !name.starts_with('.') && name != "target" && name != "node_modules" {
                        dirs.push(path);
                    }
                    continue;
                }
                if name != "wrangler.toml" {
                    continue;
                }
                let wrangler: toml::Table = std::fs::read_to_string(&path)?.parse()?;
                if wrangler.get("pages_build_output_dir").is_none() {
                    continue;
                }
                let project = wrangler
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or_default()
                    .to_string();
                let mise = path.with_file_name("mise.toml");
                let mise = std::fs::read_to_string(&mise).unwrap_or_default();
                let host = mise
                    .lines()
                    .find_map(|l| l.trim().strip_prefix("DEPLOY_URL = \"https://"))
                    .and_then(|l| l.strip_suffix(".pages.dev\""))
                    .unwrap_or_default()
                    .to_string();
                found.push((project, host));
            }
        }
        found.sort();
        Ok(found)
    }

    #[test]
    fn web_apps_match_the_pages_projects_in_the_repo() -> anyhow::Result<()> {
        let mut ours: Vec<(String, String)> = web_apps()
            .into_iter()
            .map(|a| (a.pages_project.clone(), a.pages_project))
            .collect();
        ours.sort();
        assert_eq!(ours, pages_projects_in_repo()?);
        Ok(())
    }
}
