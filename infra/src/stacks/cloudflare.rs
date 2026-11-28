//! The `cloudflare` stack: account settings for our Pages projects.
//!
//! Wrangler still creates the Pages projects and deploys their content
//! (`free-tiers.sh pages-project` guards how many). This stack starts with
//! Web Analytics on each production site. Its beacon isn't auto-installed
//! (Cloudflare allows that only on proxied zone hostnames), so each app adds
//! the beacon script itself. The account ID comes from the
//! `cloudflare_account_id` variable (`TF_VAR_cloudflare_account_id`, from the
//! `CLOUDFLARE_ACCOUNT_ID` secret) and the token from `CLOUDFLARE_API_TOKEN`,
//! never from this code.

use crate::constructs::WebApp;
use crate::synth::Synth;
use crate::tfschema::cloudflare::{BuildProviderCloudflare, BuildWebAnalyticsSite};
use terrars::{BuildStack, BuildVariable, Variable};

/// The stack's name, which is also its directory under `out/`.
pub const NAME: &str = "cloudflare";

/// Synthesize the stack for these web apps.
pub fn synth(apps: &[WebApp]) -> anyhow::Result<Synth> {
    let mut stack = BuildStack {}.build();
    stack.set_command("tofu");
    BuildProviderCloudflare {}.build(&mut stack);
    let account_id: Variable<String> = BuildVariable {
        tf_id: "cloudflare_account_id".into(),
    }
    .build(&mut stack);
    for app in apps.iter().filter(|a| a.web_analytics) {
        BuildWebAnalyticsSite {
            tf_id: app.project.clone(),
            account_id: (&account_id).into(),
        }
        .build(&mut stack)
        .set_host(app.production_host())
        // Off: Cloudflare injects the beacon itself only on hostnames
        // proxied through a zone in the account, and refuses
        // `auto_install` (`autoInstallInvalid`) for a bare `*.pages.dev`
        // host. Each app loads the beacon with its site token instead.
        .set_auto_install(false)
        // IaC never destroys (decision 0007, item 8).
        .set_prevent_destroy(true);
    }
    Synth::new(NAME).finish(&stack)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_analytics_is_on_for_each_production_host() -> anyhow::Result<()> {
        let apps = [
            WebApp::new("cookbook").web_analytics(),
            WebApp::new("bear_down").web_analytics(),
            WebApp::new("quiet"),
        ];
        let synth = synth(&apps)?;
        let sites = &synth.json["resource"]["cloudflare_web_analytics_site"];
        assert_eq!(sites["cookbook"]["host"], "jimhillr-cookbook.pages.dev");
        assert_eq!(sites["bear_down"]["host"], "jimhillr-bear-down.pages.dev");
        // Cloudflare refuses auto_install for a `*.pages.dev` host.
        assert_eq!(sites["cookbook"]["auto_install"], false);
        assert_eq!(sites["bear_down"]["auto_install"], false);
        assert_eq!(
            sites["cookbook"]["account_id"],
            "${var.cloudflare_account_id}"
        );
        assert_eq!(sites["cookbook"]["lifecycle"]["prevent_destroy"], true);
        assert!(sites.get("quiet").is_none());
        Ok(())
    }

    #[test]
    fn account_id_is_a_variable_not_a_value() -> anyhow::Result<()> {
        let synth = synth(&[WebApp::new("cookbook").web_analytics()])?;
        assert!(
            synth.json["variable"]
                .get("cloudflare_account_id")
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn provider_is_pinned() -> anyhow::Result<()> {
        let synth = synth(&[])?;
        let provider = &synth.json["terraform"]["required_providers"]["cloudflare"];
        assert_eq!(provider["source"], "cloudflare/cloudflare");
        assert_eq!(
            provider["version"],
            crate::tfschema::CLOUDFLARE_PROVIDER_VERSION
        );
        Ok(())
    }
}
