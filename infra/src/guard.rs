//! Plan guards (decision 0007, item 6): `infra guard <plan.json>` reads the
//! JSON of a saved plan (`tofu show -json plan.tfplan`) and fails it when:
//!
//! - IAC-001: it deletes or replaces anything. IaC never destroys; retire a
//!   resource with a `removed` block instead (decision 0007, item 8).
//! - IAC-002: any value the provider marks sensitive would be in state,
//!   whoever set it. Secrets never go in git or state, even encrypted.
//! - IAC-003: it would leave more resources than a free-tier budget allows
//!   (the budgets in `standards/radar/limits.toml`, the same ones
//!   `free-tiers.sh` reads), or any resource of a type that can bill.
//! - IAC-004: it uses a provider or resource type that isn't allowlisted.
//! - IAC-005: it touches the `infra-credentials` environment or a ruleset's
//!   bypass list, which IaC never manages (decision 0010, item 7).
//!
//! [`credential_changes`] also reports, without failing, whether a plan
//! changes a credential, so the workflows send it to the `infra-credentials`
//! environment, where it waits for Jim's second approval (decision 0010).

use anyhow::Context;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

/// Providers the stacks may use, by their full registry address.
pub const ALLOWED_PROVIDERS: &[&str] = &[
    "registry.opentofu.org/auth0/auth0",
    "registry.opentofu.org/cloudflare/cloudflare",
];

/// Resource types the stacks may manage. Add a type here in the same PR that
/// first uses it, after checking it can't bill.
pub const ALLOWED_RESOURCE_TYPES: &[&str] = &["auth0_client", "cloudflare_web_analytics_site"];

/// Resource types that are free only up to a budget, and the
/// `standards/radar/limits.toml` key that holds it. A Cron Trigger resource
/// counts each of its `schedules`.
pub const BUDGETED_TYPES: &[(&str, &str)] = &[
    ("cloudflare_pages_project", "pages_projects"),
    ("cloudflare_worker", "workers"),
    ("cloudflare_workers_script", "workers"),
    ("cloudflare_workers_cron_trigger", "cron_triggers"),
];

/// Resource type prefixes that can bill, so IaC never creates them, even if
/// someone allowlists one by mistake.
pub const BILLABLE_TYPE_PREFIXES: &[&str] = &[
    "cloudflare_r2_",
    "cloudflare_account_subscription",
    "cloudflare_zone_subscription",
    "cloudflare_argo_",
    "cloudflare_load_balancer",
    "cloudflare_spectrum_",
    "cloudflare_logpush_",
    "cloudflare_workers_for_platforms_",
];

/// One reason a plan fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub id: &'static str,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error[{}] {}", self.id, self.message)
    }
}

fn finding(id: &'static str, message: String) -> Finding {
    Finding { id, message }
}

/// Free-tier budgets, by `limits.toml` key.
pub type Budgets = BTreeMap<String, u64>;

/// Read the budgets from `limits.toml`: each `[[limit]]`'s `budget`, or its
/// `free` limit when it has no budget.
pub fn read_budgets(path: &Path) -> anyhow::Result<Budgets> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    parse_budgets(&text).with_context(|| format!("parsing {}", path.display()))
}

fn parse_budgets(text: &str) -> anyhow::Result<Budgets> {
    let doc: toml::Table = text.parse()?;
    let mut budgets = Budgets::new();
    for limit in doc
        .get("limit")
        .and_then(|l| l.as_array())
        .into_iter()
        .flatten()
    {
        let Some(key) = limit.get("key").and_then(|k| k.as_str()) else {
            continue;
        };
        let number = limit
            .get("budget")
            .or_else(|| limit.get("free"))
            .and_then(|n| n.as_integer());
        if let Some(n) = number.and_then(|n| u64::try_from(n).ok()) {
            budgets.insert(key.to_string(), n);
        }
    }
    Ok(budgets)
}

/// Every check, over one plan.
pub fn check(plan: &Value, budgets: &Budgets) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(no_destroy(plan));
    findings.extend(no_sensitive_values(plan));
    findings.extend(within_budgets(plan, budgets));
    findings.extend(allowlisted(plan));
    findings.extend(gate_untouched(plan));
    findings
}

fn resource_changes(plan: &Value) -> impl Iterator<Item = &Value> {
    plan.get("resource_changes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

fn str_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn actions(change: &Value) -> Vec<&str> {
    change
        .pointer("/change/actions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect()
}

/// IAC-001: no `delete` action, alone or as half of a replace.
fn no_destroy(plan: &Value) -> Vec<Finding> {
    resource_changes(plan)
        .filter(|change| actions(change).contains(&"delete"))
        .map(|change| {
            let what = if actions(change).contains(&"create") {
                "replaces"
            } else {
                "deletes"
            };
            finding(
                "IAC-001",
                format!(
                    "the plan {what} {}; IaC never destroys: forget it with a `removed` block and retire it through the cleanup plan",
                    str_field(change, "address")
                ),
            )
        })
        .collect()
}

/// The top-level attributes of `sensitive` (an `after_sensitive` value) that
/// hold a sensitive value anywhere inside them.
fn sensitive_attributes(sensitive: &Value) -> Vec<&str> {
    fn any_true(v: &Value) -> bool {
        match v {
            Value::Bool(b) => *b,
            Value::Array(items) => items.iter().any(any_true),
            Value::Object(map) => map.values().any(any_true),
            _ => false,
        }
    }
    sensitive
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(_, v)| any_true(v))
        .map(|(k, _)| k.as_str())
        .collect()
}

/// Attributes a provider marks sensitive that hold no secret, so they may
/// be in state. Each needs a reason.
pub const NOT_SECRET: &[(&str, &str, &str)] = &[(
    "auth0_client",
    "signing_keys",
    "the tenant's public signing certificates (`cert`, `pkcs7`); the Management API never returns the private key",
)];

fn is_set(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => !map.is_empty(),
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

fn is_unknown(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::Array(items)) => items.iter().any(|v| is_unknown(Some(v))),
        Some(Value::Object(map)) => map.values().any(|v| is_unknown(Some(v))),
        _ => false,
    }
}

/// IAC-002: no secret value in state, ever, even encrypted (Jim,
/// 2026-11-26). Fails on any attribute the provider marks sensitive that
/// holds a value, or will once applied (a token or client secret the vendor
/// generates), whether our configuration set it or not. Write-only
/// attributes never reach state, so they pass. The only exceptions are the
/// [`NOT_SECRET`] attributes.
fn no_sensitive_values(plan: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    for change in resource_changes(plan) {
        let Some(after) = change.pointer("/change/after").filter(|a| !a.is_null()) else {
            continue;
        };
        let Some(sensitive) = change.pointer("/change/after_sensitive") else {
            continue;
        };
        let address = str_field(change, "address");
        let kind = str_field(change, "type");
        let unknown = change.pointer("/change/after_unknown");
        for attribute in sensitive_attributes(sensitive) {
            if NOT_SECRET
                .iter()
                .any(|(t, a, _)| *t == kind && *a == attribute)
            {
                continue;
            }
            let held =
                is_set(after.get(attribute)) || is_unknown(unknown.and_then(|u| u.get(attribute)));
            if held {
                findings.push(finding(
                    "IAC-002",
                    format!(
                        "{address} would keep the sensitive attribute `{attribute}` in state; secrets never go in git or OpenTofu state, even encrypted"
                    ),
                ));
            }
        }
    }
    findings
}

/// Every resource in `planned_values`, the state the plan leads to.
fn planned_resources(plan: &Value) -> Vec<&Value> {
    fn walk<'a>(module: &'a Value, out: &mut Vec<&'a Value>) {
        out.extend(
            module
                .get("resources")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|r| str_field(r, "mode") != "data"),
        );
        for child in module
            .get("child_modules")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    if let Some(root) = plan.pointer("/planned_values/root_module") {
        walk(root, &mut out);
    }
    out
}

/// IAC-003: nothing that can bill, and budgeted types within their budget
/// once the plan is applied. The budgets are account-wide, so this counts
/// only what IaC manages; `free-tiers.sh` still counts what wrangler creates.
fn within_budgets(plan: &Value, budgets: &Budgets) -> Vec<Finding> {
    let resources = planned_resources(plan);
    let mut findings = Vec::new();
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for resource in &resources {
        let kind = str_field(resource, "type");
        if BILLABLE_TYPE_PREFIXES.iter().any(|p| kind.starts_with(p)) {
            findings.push(finding(
                "IAC-003",
                format!(
                    "{} is a {kind}, which can bill; IaC uses free resources only",
                    str_field(resource, "address")
                ),
            ));
        }
        if let Some((_, key)) = BUDGETED_TYPES.iter().find(|(t, _)| *t == kind) {
            let n = if kind == "cloudflare_workers_cron_trigger" {
                resource
                    .pointer("/values/schedules")
                    .and_then(Value::as_array)
                    .map_or(1, |s| s.len() as u64)
            } else {
                1
            };
            *counts.entry(key).or_default() += n;
        }
    }
    for (key, count) in counts {
        match budgets.get(key) {
            Some(budget) if count <= *budget => {}
            Some(budget) => findings.push(finding(
                "IAC-003",
                format!("the plan leaves {count} {key}; the budget is {budget} (standards/radar/limits.toml)"),
            )),
            None => findings.push(finding(
                "IAC-003",
                format!("no budget for {key} in standards/radar/limits.toml"),
            )),
        }
    }
    findings
}

/// IAC-004: only allowlisted providers and resource types.
fn allowlisted(plan: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    for change in resource_changes(plan) {
        let address = str_field(change, "address");
        let provider = str_field(change, "provider_name");
        if !ALLOWED_PROVIDERS.contains(&provider) {
            findings.push(finding(
                "IAC-004",
                format!("{address} uses the provider {provider}, which is not allowlisted in infra/src/guard.rs"),
            ));
        }
        let kind = str_field(change, "type");
        if str_field(change, "mode") != "data" && !ALLOWED_RESOURCE_TYPES.contains(&kind) {
            findings.push(finding(
                "IAC-004",
                format!("{address} is a {kind}, which is not allowlisted in infra/src/guard.rs"),
            ));
        }
    }
    findings
}

/// The GitHub environment that holds the credential-capable tokens and
/// Jim's second approval (decision 0010). IaC never manages it.
pub const CREDENTIALS_ENVIRONMENT: &str = "infra-credentials";

/// Resource type prefixes whose every create or update is a credential
/// change (decision 0010's table). Every GitHub type is here: GitHub can't
/// separate repo settings from rulesets and secrets, so the `github` stack
/// always needs the second approval.
pub const CREDENTIAL_TYPE_PREFIXES: &[&str] = &[
    "auth0_client_credentials",
    "auth0_client_grant",
    "auth0_resource_server",
    "auth0_connection",
    "auth0_role",
    "auth0_user_permission",
    "auth0_user_role",
    "cloudflare_api_token",
    "cloudflare_account_token",
    "cloudflare_account_member",
    "cloudflare_zero_trust_access_",
    "cloudflare_access_",
    "github_",
];

/// `auth0_client` attributes that hold no credential, so an update that
/// changes only these is everyday IaC. Creating a client, or changing any
/// other attribute (grant types, authentication method, app type, token
/// settings), is a credential change. A list of what's safe, not of what
/// isn't, so a new provider attribute counts as a credential until it is
/// added here.
pub const AUTH0_CLIENT_NOT_CREDENTIAL: &[&str] = &[
    "allowed_clients",
    "allowed_logout_urls",
    "allowed_origins",
    "callbacks",
    "client_id",
    "description",
    "id",
    "initiate_login_uri",
    "logo_uri",
    "name",
    "web_origins",
];

/// One credential change in a plan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CredentialChange {
    pub address: String,
    pub kind: String,
    pub actions: Vec<String>,
    /// The attributes that change (all of them for a create).
    pub attributes: Vec<String>,
}

/// The top-level attributes whose value differs between `before` and
/// `after`, or that are unknown until apply.
fn changed_attributes(change: &Value) -> Vec<String> {
    let empty = serde_json::Map::new();
    let object = |pointer: &str| {
        change
            .pointer(pointer)
            .and_then(Value::as_object)
            .unwrap_or(&empty)
    };
    let (before, after, unknown) = (
        object("/change/before"),
        object("/change/after"),
        object("/change/after_unknown"),
    );
    let mut keys: Vec<&String> = before.keys().chain(after.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|k| before.get(*k) != after.get(*k) || is_unknown(unknown.get(*k)))
        .cloned()
        .collect()
}

/// Every create or update in the plan that changes a credential (decision
/// 0010). Reads, no-ops and imports that change nothing are not changes.
/// An empty list means the plan may apply in the everyday `infra`
/// environment; otherwise it applies only in `infra-credentials`, after
/// Jim approves the run.
pub fn credential_changes(plan: &Value) -> Vec<CredentialChange> {
    let mut out = Vec::new();
    for change in resource_changes(plan) {
        if str_field(change, "mode") == "data" {
            continue;
        }
        let acts = actions(change);
        if !acts.iter().any(|a| *a == "create" || *a == "update") {
            continue;
        }
        let kind = str_field(change, "type");
        let attributes = changed_attributes(change);
        let credential = if kind == "auth0_client" {
            acts.contains(&"create")
                || attributes
                    .iter()
                    .any(|a| !AUTH0_CLIENT_NOT_CREDENTIAL.contains(&a.as_str()))
        } else {
            CREDENTIAL_TYPE_PREFIXES.iter().any(|p| kind.starts_with(p))
        };
        if credential {
            out.push(CredentialChange {
                address: str_field(change, "address").to_string(),
                kind: kind.to_string(),
                actions: acts.iter().map(|a| a.to_string()).collect(),
                attributes,
            });
        }
    }
    out
}

/// IAC-005: IaC never manages its own gate (decision 0010, item 7): no
/// change to the `infra-credentials` environment, its secrets, variables
/// or deployment policies, and no change to a ruleset's bypass list. Those
/// stay settings Jim makes by hand.
fn gate_untouched(plan: &Value) -> Vec<Finding> {
    let mut findings = Vec::new();
    for change in resource_changes(plan) {
        let acts = actions(change);
        if str_field(change, "mode") == "data" || acts.iter().all(|a| *a == "no-op" || *a == "read")
        {
            continue;
        }
        let kind = str_field(change, "type");
        if !kind.starts_with("github_") {
            continue;
        }
        let address = str_field(change, "address");
        let names_gate = ["/change/before/environment", "/change/after/environment"]
            .iter()
            .any(|p| change.pointer(p).and_then(Value::as_str) == Some(CREDENTIALS_ENVIRONMENT));
        if names_gate {
            findings.push(finding(
                "IAC-005",
                format!(
                    "{address} changes the `{CREDENTIALS_ENVIRONMENT}` environment, which holds Jim's second approval; IaC never manages it (decision 0010)"
                ),
            ));
        }
        if kind == "github_repository_ruleset"
            && change.pointer("/change/before/bypass_actors")
                != change.pointer("/change/after/bypass_actors")
        {
            findings.push(finding(
                "IAC-005",
                format!(
                    "{address} changes a ruleset's bypass list; only Jim changes who can bypass rules (decision 0010)"
                ),
            ));
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(name: &str) -> anyhow::Result<Value> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/plans")
            .join(name);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        Ok(serde_json::from_str(&text)?)
    }

    fn budgets() -> anyhow::Result<Budgets> {
        read_budgets(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../standards/radar/limits.toml"))
    }

    fn ids(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.id).collect()
    }

    #[test]
    fn importing_and_updating_auth0_clients_passes() -> anyhow::Result<()> {
        let findings = check(&plan("auth0-import.json")?, &budgets()?);
        assert!(findings.is_empty(), "{findings:?}");
        Ok(())
    }

    #[test]
    fn deleting_fails() -> anyhow::Result<()> {
        let findings = check(&plan("delete.json")?, &budgets()?);
        assert_eq!(ids(&findings), ["IAC-001"]);
        assert!(
            findings[0]
                .message
                .contains("deletes auth0_client.cookbook")
        );
        Ok(())
    }

    #[test]
    fn replacing_fails() -> anyhow::Result<()> {
        let findings = check(&plan("replace.json")?, &budgets()?);
        assert_eq!(ids(&findings), ["IAC-001"]);
        assert!(
            findings[0]
                .message
                .contains("replaces auth0_client.cookbook")
        );
        Ok(())
    }

    #[test]
    fn forgetting_with_a_removed_block_passes() -> anyhow::Result<()> {
        let findings = check(&plan("forget.json")?, &budgets()?);
        assert!(findings.is_empty(), "{findings:?}");
        Ok(())
    }

    #[test]
    fn any_sensitive_value_in_state_fails() -> anyhow::Result<()> {
        // One we set (`encryption_key`) and one the vendor generated
        // (`client_secret`): both would be in state.
        let findings = check(&plan("sensitive.json")?, &budgets()?);
        assert_eq!(ids(&findings), ["IAC-002", "IAC-002"]);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("`encryption_key`"))
        );
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("`client_secret`"))
        );
        Ok(())
    }

    #[test]
    fn a_generated_token_fails_before_it_exists() -> anyhow::Result<()> {
        // A new token's value is unknown at plan time and lands in state on
        // apply.
        let findings = no_sensitive_values(&plan("minted-token.json")?);
        assert_eq!(ids(&findings), ["IAC-002"]);
        assert!(findings[0].message.contains("cloudflare_api_token.deploy"));
        assert!(findings[0].message.contains("`value`"));
        Ok(())
    }

    #[test]
    fn write_only_and_public_sensitive_attributes_pass() -> anyhow::Result<()> {
        let findings = no_sensitive_values(&plan("not-secret.json")?);
        assert!(findings.is_empty(), "{findings:?}");
        Ok(())
    }

    #[test]
    fn pages_projects_past_the_budget_fail() -> anyhow::Result<()> {
        let mut budgets = budgets()?;
        budgets.insert("pages_projects".into(), 1);
        let findings = within_budgets(&plan("cloudflare.json")?, &budgets);
        assert_eq!(ids(&findings), ["IAC-003"]);
        assert!(
            findings[0]
                .message
                .contains("leaves 2 pages_projects; the budget is 1")
        );
        Ok(())
    }

    #[test]
    fn cron_triggers_count_each_schedule() -> anyhow::Result<()> {
        let mut budgets = budgets()?;
        budgets.insert("cron_triggers".into(), 2);
        let findings = within_budgets(&plan("cloudflare.json")?, &budgets);
        assert_eq!(ids(&findings), ["IAC-003"]);
        assert!(findings[0].message.contains("leaves 3 cron_triggers"));
        Ok(())
    }

    #[test]
    fn billable_types_fail_and_are_not_allowlisted() -> anyhow::Result<()> {
        let findings = check(&plan("r2.json")?, &budgets()?);
        assert_eq!(ids(&findings), ["IAC-003", "IAC-004"]);
        Ok(())
    }

    #[test]
    fn other_providers_and_types_fail() -> anyhow::Result<()> {
        let findings = allowlisted(&plan("cloudflare.json")?);
        assert!(!findings.is_empty());
        assert!(findings.iter().all(|f| f.id == "IAC-004"));
        Ok(())
    }

    /// A real `tofu show -json` plan (OpenTofu 1.13.10, built-in
    /// `terraform_data`): one delete, one update of a value set from a
    /// sensitive variable, one create.
    #[test]
    fn real_opentofu_plan_is_read() -> anyhow::Result<()> {
        let findings = check(&plan("tofu-real.json")?, &budgets()?);
        let ids = ids(&findings);
        assert_eq!(ids.iter().filter(|id| **id == "IAC-001").count(), 1);
        assert_eq!(ids.iter().filter(|id| **id == "IAC-002").count(), 1);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("deletes terraform_data.a"))
        );
        assert!(findings.iter().any(|f| {
            f.message
                .contains("terraform_data.b would keep the sensitive attribute `input`")
        }));
        Ok(())
    }

    #[test]
    fn budgets_come_from_limits_toml() -> anyhow::Result<()> {
        let budgets = budgets()?;
        for (_, key) in BUDGETED_TYPES {
            assert!(budgets.contains_key(*key), "no budget for {key}");
        }
        Ok(())
    }

    #[test]
    fn budget_falls_back_to_the_free_limit() -> anyhow::Result<()> {
        let budgets = parse_budgets(
            "[[limit]]\nkey = \"a\"\nfree = 5\nbudget = 3\n\n[[limit]]\nkey = \"b\"\nfree = 7\n",
        )?;
        assert_eq!(budgets.get("a"), Some(&3));
        assert_eq!(budgets.get("b"), Some(&7));
        Ok(())
    }

    #[test]
    fn every_type_the_stacks_use_is_allowlisted() -> anyhow::Result<()> {
        for stack in crate::stacks::all()? {
            for kind in stack
                .json
                .get("resource")
                .and_then(Value::as_object)
                .into_iter()
                .flatten()
                .map(|(k, _)| k.as_str())
            {
                assert!(
                    ALLOWED_RESOURCE_TYPES.contains(&kind),
                    "{kind} is not allowlisted"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn auth0_url_updates_are_not_credential_changes() -> anyhow::Result<()> {
        assert_eq!(credential_changes(&plan("auth0-import.json")?), []);
        Ok(())
    }

    #[test]
    fn creating_an_auth0_client_is_a_credential_change() -> anyhow::Result<()> {
        let changes = credential_changes(&plan("credential-auth0.json")?);
        let addresses: Vec<&str> = changes.iter().map(|c| c.address.as_str()).collect();
        assert_eq!(
            addresses,
            [
                "auth0_client.new_app",
                "auth0_client.cookbook",
                "auth0_client_grant.infra"
            ]
        );
        // The URL-only update in the same plan is not listed.
        assert_eq!(
            changes[1].attributes,
            ["callbacks", "token_endpoint_auth_method"]
        );
        Ok(())
    }

    #[test]
    fn tokens_and_access_are_credential_changes() -> anyhow::Result<()> {
        let changes = credential_changes(&plan("credential-cloudflare.json")?);
        let kinds: Vec<&str> = changes.iter().map(|c| c.kind.as_str()).collect();
        assert_eq!(
            kinds,
            [
                "cloudflare_api_token",
                "cloudflare_zero_trust_access_policy"
            ]
        );
        // The Pages project in the same plan is not a credential.
        assert!(credential_changes(&plan("cloudflare.json")?).is_empty());
        Ok(())
    }

    #[test]
    fn every_github_change_is_a_credential_change() -> anyhow::Result<()> {
        let changes = credential_changes(&plan("gate.json")?);
        assert_eq!(changes.len(), 3);
        assert!(changes.iter().all(|c| c.kind.starts_with("github_")));
        Ok(())
    }

    #[test]
    fn touching_the_gate_fails() -> anyhow::Result<()> {
        let findings = gate_untouched(&plan("gate.json")?);
        assert_eq!(ids(&findings), ["IAC-005", "IAC-005"]);
        assert!(
            findings[0]
                .message
                .contains("github_repository_environment.credentials")
        );
        assert!(findings[1].message.contains("bypass list"));
        Ok(())
    }
}
