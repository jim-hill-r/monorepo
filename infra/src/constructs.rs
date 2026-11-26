//! Constructs: building blocks that say what we mean ("cookbook is a web app
//! with sign-in") and expand to the provider resources that make it so.
//!
//! They hold no `terrars` types, so a stack can be tested without
//! serializing it, and the typed bindings under them can be swapped (decision
//! 0007, A1 or A2) without touching them.

/// The port `dx serve` uses for every web app (`web/Dioxus.toml`).
pub const DEV_PORT: u16 = 8080;

/// A web app served from a Cloudflare Pages project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebApp {
    /// The monorepo project, such as `cookbook`. Used for resource names.
    pub project: String,
    /// The Cloudflare Pages project name. `jimhillr-{project}` unless the
    /// project hasn't been renamed yet.
    pub pages_project: String,
    /// The Auth0 application it signs in with, if any.
    pub sign_in: Option<Auth0App>,
}

impl WebApp {
    /// A web app whose Pages project follows the `jimhillr-{project}` rule.
    pub fn new(project: &str) -> Self {
        Self {
            project: project.to_string(),
            pages_project: format!("jimhillr-{}", project.replace('_', "-")),
            sign_in: None,
        }
    }

    /// Sign in with an Auth0 application.
    pub fn sign_in(mut self, app: Auth0App) -> Self {
        self.sign_in = Some(app);
        self
    }

    /// The production origin, `https://<pages project>.pages.dev`.
    pub fn production_origin(&self) -> String {
        format!("https://{}.pages.dev", self.pages_project)
    }

    /// Every preview of this project. Pages previews are
    /// `<branch or hash>.<pages project>.pages.dev`, one level under the
    /// project, which is the one left-most wildcard level Auth0 allows.
    pub fn preview_origin(&self) -> String {
        format!("https://*.{}.pages.dev", self.pages_project)
    }

    /// The origin `dx serve` uses.
    pub fn dev_origin(&self) -> String {
        format!("http://localhost:{DEV_PORT}")
    }

    /// Production, previews and localhost, in that order.
    pub fn origins(&self) -> Vec<String> {
        vec![
            self.production_origin(),
            self.preview_origin(),
            self.dev_origin(),
        ]
    }

    /// The OAuth callback URLs: each origin plus `/`, which is what
    /// `auth_sdk::redirect::redirect_url_for_origin` sends.
    pub fn callback_urls(&self) -> Vec<String> {
        self.origins()
            .into_iter()
            .map(|o| format!("{o}/"))
            .collect()
    }
}

/// An Auth0 application that already exists and is imported by client ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auth0App {
    /// The client ID, which is also the import ID.
    pub client_id: String,
    /// The application's name in the Auth0 dashboard.
    pub name: String,
}

impl Auth0App {
    /// An existing application, brought under IaC with an `import` block.
    pub fn import(client_id: &str, name: &str) -> Self {
        Self {
            client_id: client_id.to_string(),
            name: name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_project_follows_the_naming_rule() {
        assert_eq!(WebApp::new("cookbook").pages_project, "jimhillr-cookbook");
        assert_eq!(WebApp::new("bear_down").pages_project, "jimhillr-bear-down");
    }

    #[test]
    fn origins_cover_production_previews_and_localhost() {
        assert_eq!(
            WebApp::new("cookbook").origins(),
            vec![
                "https://jimhillr-cookbook.pages.dev",
                "https://*.jimhillr-cookbook.pages.dev",
                "http://localhost:8080",
            ]
        );
    }

    #[test]
    fn callback_urls_are_origins_with_a_trailing_slash() {
        assert_eq!(
            WebApp::new("cookbook").callback_urls(),
            vec![
                "https://jimhillr-cookbook.pages.dev/",
                "https://*.jimhillr-cookbook.pages.dev/",
                "http://localhost:8080/",
            ]
        );
    }

    #[test]
    fn preview_wildcard_is_one_left_most_level() {
        let preview = WebApp::new("cookbook").preview_origin();
        let host = preview.trim_start_matches("https://");
        assert_eq!(host.matches('*').count(), 1);
        assert!(host.starts_with("*."));
    }
}
