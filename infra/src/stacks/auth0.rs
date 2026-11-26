//! The `auth0` stack: the Auth0 applications our web apps sign in with.
//!
//! Every application already exists, so each one is brought in with an
//! `import` block and only the settings below are managed. Provider
//! credentials come from the environment (`AUTH0_DOMAIN`, `AUTH0_CLIENT_ID`,
//! `AUTH0_CLIENT_SECRET`), never from this code.

use crate::constructs::WebApp;
use crate::synth::Synth;
use crate::tfschema::auth0::{BuildClient, BuildProviderAuth0};
use terrars::{BuildStack, PrimField, Referable};

/// The stack's name, which is also its directory under `out/`.
pub const NAME: &str = "auth0";

fn strings(values: Vec<String>) -> Vec<PrimField<String>> {
    values.into_iter().map(PrimField::from).collect()
}

/// Synthesize the stack for these web apps. Apps without sign-in are skipped.
pub fn synth(apps: &[WebApp]) -> anyhow::Result<Synth> {
    let mut stack = BuildStack {}.build();
    stack.set_command("tofu");
    BuildProviderAuth0 {}.build(&mut stack);
    let mut synth = Synth::new(NAME);
    for app in apps {
        let Some(auth0) = &app.sign_in else {
            continue;
        };
        let client = BuildClient {
            tf_id: app.project.clone(),
            name: auth0.name.as_str().into(),
        }
        .build(&mut stack)
        .set_app_type("spa")
        .set_callbacks(strings(app.callback_urls()))
        .set_web_origins(strings(app.origins()))
        .set_allowed_logout_urls(strings(app.origins()))
        // IaC never destroys (decision 0007, item 8).
        .set_prevent_destroy(true);
        synth.import(client.extract_ref(), &auth0.client_id);
    }
    synth.finish(&stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constructs::Auth0App;
    use serde_json::json;

    fn cookbook() -> WebApp {
        WebApp::new("cookbook").sign_in(Auth0App::import("abc", "cookbook"))
    }

    #[test]
    fn client_has_every_origin_and_is_a_spa() -> anyhow::Result<()> {
        let synth = synth(&[cookbook()])?;
        let client = &synth.json["resource"]["auth0_client"]["cookbook"];
        assert_eq!(client["name"], "cookbook");
        assert_eq!(client["app_type"], "spa");
        assert_eq!(
            client["callbacks"],
            json!([
                "https://jimhillr-cookbook.pages.dev/",
                "https://*.jimhillr-cookbook.pages.dev/",
                "http://localhost:8080/",
            ])
        );
        let origins = json!([
            "https://jimhillr-cookbook.pages.dev",
            "https://*.jimhillr-cookbook.pages.dev",
            "http://localhost:8080",
        ]);
        assert_eq!(client["web_origins"], origins);
        assert_eq!(client["allowed_logout_urls"], origins);
        assert_eq!(client["lifecycle"]["prevent_destroy"], true);
        Ok(())
    }

    #[test]
    fn existing_client_is_imported_by_client_id() -> anyhow::Result<()> {
        let synth = synth(&[cookbook()])?;
        assert_eq!(
            synth.json["import"],
            json!([{ "id": "abc", "to": "auth0_client.cookbook" }])
        );
        Ok(())
    }

    #[test]
    fn apps_without_sign_in_are_skipped() -> anyhow::Result<()> {
        let synth = synth(&[WebApp::new("bear_down")])?;
        assert!(synth.json.get("resource").is_none());
        assert!(synth.json.get("import").is_none());
        Ok(())
    }

    #[test]
    fn provider_is_pinned_and_has_no_credentials() -> anyhow::Result<()> {
        let synth = synth(&[cookbook()])?;
        let provider = &synth.json["terraform"]["required_providers"]["auth0"];
        assert_eq!(provider["source"], "auth0/auth0");
        assert_eq!(provider["version"], crate::tfschema::AUTH0_PROVIDER_VERSION);
        assert_eq!(synth.json["provider"]["auth0"], json!([{}]));
        Ok(())
    }
}
