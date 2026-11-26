//! Stacks: one OpenTofu root module each, synthesized to `out/<name>/`.
//!
//! Each stack has its own state and credentials, so a PR plan for one stack
//! needs only that vendor's read-only token.

pub mod auth0;

use crate::constructs::{Auth0App, WebApp};
use crate::synth::Synth;

/// Every web app, and how it signs in. This is the source of truth for the
/// stacks below.
pub fn web_apps() -> Vec<WebApp> {
    vec![
        WebApp::new("cookbook").sign_in(Auth0App::import(
            "savzmZnyHcvewGkQX8aaInwPFonC9k2x",
            "cookbook",
        )),
        // cahokia's Pages project predates the `jimhillr-` naming rule.
        WebApp::new("cahokia")
            .pages_project("cahokia-web")
            .sign_in(Auth0App::import(
                "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E",
                "cahokia",
            )),
    ]
}

/// Every stack, synthesized.
pub fn all() -> anyhow::Result<Vec<Synth>> {
    Ok(vec![auth0::synth(&web_apps())?])
}
