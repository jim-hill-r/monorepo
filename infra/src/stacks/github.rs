//! The `github` stack: this repository's own settings (decision 0007, phase
//! 2).
//!
//! The repository already exists, so it is brought in with an `import` block
//! and every setting the provider would otherwise default is written out with
//! today's value. Code security (secret scanning with push protection,
//! Dependabot alerts and security updates) and a `CI` ruleset that requires
//! the `CI result` and `Jim review` checks on `main` are what it adds.
//!
//! Every change here is a credential change (decision 0010), so it applies
//! only in `infra-credentials`, after Jim approves the run. Plans read with a
//! token from the read-only infra GitHub App (`GH_INFRA_READ_TOKEN`) and
//! applies use the infra GitHub App (`GH_INFRA_CREDENTIALS_TOKEN`); both are
//! minted per run by `actions/create-github-app-token` and reach the provider
//! as `GITHUB_TOKEN`, never through this code.
//!
//! What it never manages (IAC-005): the `infra-credentials` environment and
//! any ruleset's bypass list. The `Default` ruleset stays a setting Jim makes
//! by hand: provider 6.13.0 can't express its
//! `require_extra_approval_for_unattributed_changes` rule, so an update would
//! silently turn it off. Rulesets add up, so the `CI` ruleset sits beside it.

use crate::synth::Synth;
use crate::tfschema::github::{
    BuildProviderGithub, BuildRepository, BuildRepository_SecurityAndAnalysis,
    BuildRepository_SecurityAndAnalysis_SecretScanning,
    BuildRepository_SecurityAndAnalysis_SecretScanningPushProtection,
    BuildRepositoryDependabotSecurityUpdates, BuildRepositoryRuleset,
    BuildRepositoryRuleset_Conditions, BuildRepositoryRuleset_Conditions_RefName,
    BuildRepositoryRuleset_Rules, BuildRepositoryRuleset_Rules_RequiredStatusChecks,
    BuildRepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck,
    BuildRepositoryVulnerabilityAlerts,
};
use terrars::{BuildStack, PrimField, Referable};

/// The stack's name, which is also its directory under `out/`.
pub const NAME: &str = "github";

/// The repository's owner, set on the provider.
pub const OWNER: &str = "jimhillr";

/// The repository this stack manages. It is the GitHub name, which we
/// don't change; the resource addresses use [`TF_ID`] instead.
pub const REPOSITORY: &str = "monorepo";

/// The name in the repository's resource addresses, such as
/// `github_repository.jimhillr` (AGENTS.md: name things `jimhillr`).
pub const TF_ID: &str = "jimhillr";

/// The name those addresses had before, which `moved` blocks map to
/// [`TF_ID`] so nothing is recreated.
const OLD_TF_ID: &str = "monorepo";

/// The checks every PR into `main` must pass: `CI result` from PR CI
/// (`pull-request-ci.yml`), and `Jim review` (`pull-request-jim-review.yml`,
/// PR-004), which holds a PR with a red attention tag or a cost change
/// until Jim approves it (Jim, 2026-11-27).
pub const REQUIRED_CHECKS: [&str; 2] = ["CI result", "Jim review"];

/// The GitHub Actions app's ID, so only a check from Actions counts.
pub const GITHUB_ACTIONS_APP_ID: f64 = 15368.0;

fn enabled<T>(build: impl FnOnce(PrimField<String>) -> T) -> Vec<T> {
    vec![build("enabled".into())]
}

/// Synthesize the stack.
pub fn synth() -> anyhow::Result<Synth> {
    let mut stack = BuildStack {}.build();
    stack.set_command("tofu");
    BuildProviderGithub {}.build(&mut stack).set_owner(OWNER);
    let mut synth = Synth::new(NAME);

    let security = BuildRepository_SecurityAndAnalysis {}
        .build()
        .set_secret_scanning(enabled(|status| {
            BuildRepository_SecurityAndAnalysis_SecretScanning { status }.build()
        }))
        .set_secret_scanning_push_protection(enabled(|status| {
            BuildRepository_SecurityAndAnalysis_SecretScanningPushProtection { status }.build()
        }));
    // Today's settings, as `GET /repos/jimhillr/monorepo` returned them on
    // 2026-11-27, written out so the import changes only code security.
    let repository = BuildRepository {
        tf_id: TF_ID.into(),
        name: REPOSITORY.into(),
    }
    .build(&mut stack)
    .set_description("Monorepo for all projects")
    .set_homepage_url("")
    .set_visibility("public")
    .set_has_issues(false)
    .set_has_projects(false)
    .set_has_wiki(false)
    .set_has_discussions(false)
    .set_is_template(false)
    .set_archived(false)
    .set_allow_merge_commit(false)
    .set_allow_rebase_merge(false)
    .set_allow_squash_merge(true)
    .set_allow_auto_merge(true)
    .set_allow_update_branch(true)
    .set_delete_branch_on_merge(true)
    .set_squash_merge_commit_title("PR_TITLE")
    .set_squash_merge_commit_message("BLANK")
    .set_merge_commit_title("MERGE_MESSAGE")
    .set_merge_commit_message("PR_TITLE")
    .set_web_commit_signoff_required(false)
    .set_security_and_analysis(vec![security])
    // If anything ever tried to destroy it, archive instead of deleting.
    .set_archive_on_destroy(true)
    // IaC never destroys (decision 0007, item 8).
    .set_prevent_destroy(true);
    synth.import(repository.extract_ref(), REPOSITORY);
    synth.moved(
        &format!("github_repository.{OLD_TF_ID}"),
        repository.extract_ref(),
    );

    let alerts = BuildRepositoryVulnerabilityAlerts {
        tf_id: TF_ID.into(),
        repository: REPOSITORY.into(),
    }
    .build(&mut stack)
    .set_enabled(true)
    .set_prevent_destroy(true);
    synth.moved(
        &format!("github_repository_vulnerability_alerts.{OLD_TF_ID}"),
        alerts.extract_ref(),
    );

    let updates = BuildRepositoryDependabotSecurityUpdates {
        tf_id: TF_ID.into(),
        enabled: true.into(),
        repository: REPOSITORY.into(),
    }
    .build(&mut stack)
    .set_prevent_destroy(true);
    synth.moved(
        &format!("github_repository_dependabot_security_updates.{OLD_TF_ID}"),
        updates.extract_ref(),
    );

    let checks: Vec<_> = REQUIRED_CHECKS
        .iter()
        .map(|context| {
            BuildRepositoryRuleset_Rules_RequiredStatusChecks_RequiredCheck {
                context: (*context).into(),
            }
            .build()
            .set_integration_id(GITHUB_ACTIONS_APP_ID)
        })
        .collect();
    let status_checks = BuildRepositoryRuleset_Rules_RequiredStatusChecks {}
        .build()
        // A PR need not be up to date with `main`, only green.
        .set_strict_required_status_checks_policy(false)
        .set_required_check(checks);
    let ref_name = BuildRepositoryRuleset_Conditions_RefName {
        exclude: Vec::<PrimField<String>>::new().into(),
        include: vec![PrimField::from("~DEFAULT_BRANCH")].into(),
    }
    .build();
    BuildRepositoryRuleset {
        tf_id: "ci".into(),
        enforcement: "active".into(),
        name: "CI".into(),
        repository: REPOSITORY.into(),
        target: "branch".into(),
    }
    .build(&mut stack)
    .set_conditions(vec![
        BuildRepositoryRuleset_Conditions {}
            .build()
            .set_ref_name(vec![ref_name]),
    ])
    .set_rules(vec![
        BuildRepositoryRuleset_Rules {}
            .build()
            .set_required_status_checks(vec![status_checks]),
    ])
    // Who may bypass a ruleset is Jim's alone (IAC-005): the ruleset is
    // created with nobody, and a bypass Jim adds by hand is left alone.
    .ignore_changes_to_attr("bypass_actors")
    .set_prevent_destroy(true);

    synth.finish(&stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_repository_is_imported_with_todays_settings() -> anyhow::Result<()> {
        let synth = synth()?;
        assert_eq!(
            synth.json["import"],
            json!([{ "id": "monorepo", "to": "github_repository.jimhillr" }])
        );
        let repo = &synth.json["resource"]["github_repository"]["jimhillr"];
        assert_eq!(repo["name"], "monorepo");
        assert_eq!(repo["visibility"], "public");
        assert_eq!(repo["delete_branch_on_merge"], true);
        assert_eq!(repo["allow_squash_merge"], true);
        assert_eq!(repo["allow_merge_commit"], false);
        assert_eq!(repo["allow_rebase_merge"], false);
        assert_eq!(repo["allow_auto_merge"], true);
        assert_eq!(repo["has_issues"], false);
        assert_eq!(repo["archive_on_destroy"], true);
        assert_eq!(repo["lifecycle"]["prevent_destroy"], true);
        Ok(())
    }

    #[test]
    fn the_old_monorepo_addresses_move_to_jimhillr() -> anyhow::Result<()> {
        let synth = synth()?;
        assert_eq!(
            synth.json["moved"],
            json!([
                {
                    "from": "github_repository.monorepo",
                    "to": "github_repository.jimhillr"
                },
                {
                    "from": "github_repository_vulnerability_alerts.monorepo",
                    "to": "github_repository_vulnerability_alerts.jimhillr"
                },
                {
                    "from": "github_repository_dependabot_security_updates.monorepo",
                    "to": "github_repository_dependabot_security_updates.jimhillr"
                }
            ])
        );
        Ok(())
    }

    #[test]
    fn secret_scanning_and_push_protection_are_on() -> anyhow::Result<()> {
        let synth = synth()?;
        let security =
            &synth.json["resource"]["github_repository"]["jimhillr"]["security_and_analysis"][0];
        assert_eq!(security["secret_scanning"][0]["status"], "enabled");
        assert_eq!(
            security["secret_scanning_push_protection"][0]["status"],
            "enabled"
        );
        // A public repo always has advanced security; the API refuses it.
        assert!(security["advanced_security"].is_null());
        Ok(())
    }

    #[test]
    fn dependabot_alerts_and_security_updates_are_on() -> anyhow::Result<()> {
        let synth = synth()?;
        let resources = &synth.json["resource"];
        let alerts = &resources["github_repository_vulnerability_alerts"]["jimhillr"];
        assert_eq!(alerts["enabled"], true);
        assert_eq!(alerts["repository"], "monorepo");
        let updates = &resources["github_repository_dependabot_security_updates"]["jimhillr"];
        assert_eq!(updates["enabled"], true);
        assert_eq!(updates["lifecycle"]["prevent_destroy"], true);
        Ok(())
    }

    #[test]
    fn main_requires_ci_result_and_jim_review_from_actions() -> anyhow::Result<()> {
        let synth = synth()?;
        let ruleset = &synth.json["resource"]["github_repository_ruleset"]["ci"];
        assert_eq!(ruleset["name"], "CI");
        assert_eq!(ruleset["enforcement"], "active");
        assert_eq!(ruleset["target"], "branch");
        assert_eq!(
            ruleset["conditions"][0]["ref_name"][0]["include"],
            json!(["~DEFAULT_BRANCH"])
        );
        let checks = &ruleset["rules"][0]["required_status_checks"][0];
        assert_eq!(
            checks["required_check"],
            json!([
                { "context": "CI result", "integration_id": 15368.0 },
                { "context": "Jim review", "integration_id": 15368.0 }
            ])
        );
        assert_eq!(checks["strict_required_status_checks_policy"], false);
        Ok(())
    }

    #[test]
    fn bypass_lists_are_never_managed() -> anyhow::Result<()> {
        let synth = synth()?;
        let ruleset = &synth.json["resource"]["github_repository_ruleset"]["ci"];
        assert!(
            ruleset["bypass_actors"].is_null(),
            "no bypass actors in code"
        );
        assert_eq!(
            ruleset["lifecycle"]["ignore_changes"],
            json!(["bypass_actors"])
        );
        // The Default ruleset (and its bypass list) is not in IaC at all.
        assert_eq!(
            synth.json["resource"]["github_repository_ruleset"]
                .as_object()
                .map(|m| m.len()),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn the_infra_credentials_environment_is_not_managed() -> anyhow::Result<()> {
        let synth = synth()?;
        assert!(
            synth.json["resource"]
                .get("github_repository_environment")
                .is_none()
        );
        assert!(!synth.contents()?.contains("infra-credentials"));
        Ok(())
    }

    #[test]
    fn provider_is_pinned_and_has_no_token() -> anyhow::Result<()> {
        let synth = synth()?;
        let provider = &synth.json["terraform"]["required_providers"]["github"];
        assert_eq!(provider["source"], "integrations/github");
        assert_eq!(
            provider["version"],
            crate::tfschema::GITHUB_PROVIDER_VERSION
        );
        assert_eq!(
            synth.json["provider"]["github"],
            json!([{ "owner": "jimhillr" }])
        );
        Ok(())
    }
}
