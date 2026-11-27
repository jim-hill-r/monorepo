//! The parser against the real files: every `royalty.toml` in the repo and
//! every worked example in `docs/royalty-toml.md`, plus one test per check.

use std::fs;
use std::path::{Path, PathBuf};

use royalty_model::*;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn royalty_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            // Test fixtures hold example producers with terms on purpose.
            if !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "fixtures"
            {
                royalty_files(&path, out);
            }
        } else if name == "royalty.toml" {
            out.push(path);
        }
    }
}

/// The ```toml blocks of the reference page.
fn worked_examples() -> Vec<String> {
    let page =
        fs::read_to_string(repo_root().join("royalty/docs/royalty-toml.md")).unwrap_or_default();
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in page.lines() {
        match (&mut current, line.trim()) {
            (None, "```toml") => current = Some(String::new()),
            (Some(block), "```") => {
                blocks.push(std::mem::take(block));
                current = None;
            }
            (Some(block), _) => {
                block.push_str(line);
                block.push('\n');
            }
            _ => {}
        }
    }
    blocks
}

fn codes(parsed: &Parsed) -> Vec<(Code, Level)> {
    parsed.findings.iter().map(|f| (f.code, f.level)).collect()
}

fn with_project(rest: &str) -> String {
    format!("[project]\nname = \"demo\"\ndescription = \"A demo.\"\n{rest}")
}

#[test]
fn royl_007_every_royalty_toml_in_the_repo_means_no_payment_terms() {
    let mut files = Vec::new();
    royalty_files(&repo_root(), &mut files);
    assert!(
        files.len() >= 20,
        "found only {} royalty.toml files",
        files.len()
    );
    for path in files {
        let text = fs::read_to_string(&path).unwrap_or_default();
        let parsed = parse(&text);
        assert!(
            parsed.findings.is_empty(),
            "{}: {:?}",
            path.display(),
            parsed.findings
        );
        let file = parsed.valid().cloned();
        assert!(
            file.is_some_and(|f| !f.has_terms && f.consumer.is_none()),
            "{} should mean no payment terms",
            path.display()
        );
    }
}

#[test]
fn royl_007_a_file_with_only_project_has_no_findings_and_all_defaults() {
    let parsed = parse(&with_project(""));
    assert!(parsed.findings.is_empty(), "{:?}", parsed.findings);
    let Some(file) = parsed.valid() else {
        panic!("not valid");
    };
    assert_eq!(file.schema, 1);
    assert_eq!(file.project.packages, vec!["demo".to_string()]);
    assert_eq!(file.project.family, "demo");
    assert_eq!(file.license.binding, Binding::Voluntary);
    assert_eq!(file.terms, Terms::default());
    assert_eq!(file.terms.minimum_payment_cents, 500);
    let consumer = file.consumer_or_default();
    assert_eq!(consumer.stage, Stage::Development);
    assert_eq!(consumer.entity, Entity::Person);
    assert!(!consumer.budget.is_set());
}

#[test]
fn every_worked_example_parses_without_errors() {
    let examples = worked_examples();
    assert_eq!(examples.len(), 6, "the reference has six toml blocks");
    for example in &examples {
        let parsed = parse(example);
        assert!(!parsed.has_errors(), "{example}\n{:?}", parsed.findings);
        let unexpected: Vec<_> = parsed
            .findings
            .iter()
            .filter(|f| f.code != Code::Rtoml013)
            .collect();
        assert!(unexpected.is_empty(), "{example}\n{unexpected:?}");
    }
}

#[test]
fn the_framework_example_reads_as_written() {
    let examples = worked_examples();
    let Some(file) = examples
        .iter()
        .find(|e| e.contains("name = \"webframe\""))
        .and_then(|e| parse(e).valid().cloned())
    else {
        panic!("webframe example missing or invalid");
    };
    assert_eq!(file.project.family, "webframe");
    assert!(file.covers("webframe-router"));
    assert!(!file.covers("other"));
    assert_eq!(file.license.binding, Binding::Agreement);
    assert_eq!(file.terms.mode, Mode::Report);
    assert_eq!(file.terms.trial_days, 90);
    assert_eq!(file.payees[0].methods.len(), 2);
    assert_eq!(file.tiers.len(), 3);
    assert!(file.tiers[0].free && file.tiers[1].free && !file.tiers[2].free);
    assert_eq!(file.tiers[1].when.revenue_below_usd, Some(1_000_000));
    let price = &file.tiers[2].prices[0];
    assert_eq!(
        (price.unit.clone(), price.cents, price.included, price.per),
        (Unit::Deployment, 2000, 1, 1)
    );
    assert_eq!(file.caps.project.and_then(|c| c.period_cents), Some(10_000));
    assert_eq!(
        file.caps.organization,
        Some(Cap {
            period_cents: Some(50_000),
            year_cents: Some(500_000)
        })
    );
    assert_eq!(file.income.and_then(|i| i.ceiling_cents), Some(25_000_000));
}

#[test]
fn the_consumer_example_reads_as_written() {
    let examples = worked_examples();
    let Some(file) = examples
        .iter()
        .find(|e| e.contains("[consumer]"))
        .and_then(|e| parse(e).valid().cloned())
    else {
        panic!("consumer example missing or invalid");
    };
    let Some(consumer) = file.consumer else {
        panic!("no consumer");
    };
    assert_eq!(consumer.entity, Entity::Organization);
    assert_eq!(consumer.stage, Stage::Production);
    assert_eq!(consumer.deployments.len(), 2);
    assert_eq!(consumer.budget.period_cents, Some(4500));
    assert_eq!(consumer.budget.on_exceed, OnExceed::Scale);
    assert_eq!(
        consumer.policy.bindings,
        vec![Binding::Voluntary, Binding::Agreement]
    );
    assert_eq!(consumer.policy.transitive, TransitivePolicy::Report);
    assert_eq!(consumer.measure.run, Run::Ci);
}

#[test]
fn royl_007_bad_toml_is_an_error_with_its_line() {
    let parsed = parse("[project]\nname = \"x\"\ndescription = \n");
    assert!(parsed.file.is_none());
    assert_eq!(parsed.findings[0].code, Code::Rtoml001);
    assert_eq!(parsed.findings[0].at, "line 3");
}

#[test]
fn royl_007_a_wrong_type_is_an_error() {
    let parsed = parse(&with_project("[terms]\ntrial_days = \"ninety\"\n"));
    assert!(parsed.file.is_none());
    assert_eq!(codes(&parsed), vec![(Code::Rtoml001, Level::Error)]);
}

#[test]
fn royl_007_an_unknown_schema_is_refused() {
    let parsed = parse(&format!("schema = 2\n{}", with_project("")));
    assert!(parsed.file.is_none());
    assert_eq!(codes(&parsed), vec![(Code::Rtoml002, Level::Error)]);
}

#[test]
fn royl_007_unknown_fields_are_warnings_and_the_rest_is_read() {
    let parsed = parse(&with_project(
        "homepage = \"https://x.example\"\n[terms]\nperiod = \"year\"\nperoid = \"month\"\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://x.example/pay\"\n",
    ));
    assert!(!parsed.has_errors(), "{:?}", parsed.findings);
    let at: Vec<&str> = parsed
        .findings
        .iter()
        .filter(|f| f.code == Code::Rtoml003)
        .map(|f| f.at.as_str())
        .collect();
    assert_eq!(at, vec!["project.homepage", "terms.peroid"]);
    assert_eq!(parsed.valid().map(|f| f.terms.period), Some(Period::Year));
}

#[test]
fn royl_007_project_name_and_description_are_required() {
    let parsed = parse("schema = 1\n");
    assert_eq!(codes(&parsed), vec![(Code::Rtoml004, Level::Error)]);
    let parsed = parse("[project]\nname = \"x\"\n");
    assert_eq!(codes(&parsed), vec![(Code::Rtoml004, Level::Error)]);
}

#[test]
fn agreement_and_license_need_terms_text() {
    for binding in ["agreement", "license"] {
        let parsed = parse(&with_project(&format!(
            "[license]\nbinding = \"{binding}\"\n"
        )));
        assert!(
            codes(&parsed).contains(&(Code::Rtoml005, Level::Error)),
            "{binding}: {:?}",
            parsed.findings
        );
    }
    assert!(!parse(&with_project("[license]\nbinding = \"voluntary\"\n")).has_errors());
}

#[test]
fn a_license_binding_warns_it_is_not_open_source() {
    let parsed = parse(&with_project(
        "[license]\nbinding = \"license\"\nterms = \"LICENSE.md\"\n",
    ));
    assert_eq!(codes(&parsed), vec![(Code::Rtoml013, Level::Warning)]);
}

#[test]
fn a_payee_needs_a_usable_method() {
    let parsed = parse(&with_project("[[payee]]\nname = \"A\"\n"));
    assert_eq!(codes(&parsed), vec![(Code::Rtoml006, Level::Error)]);

    let parsed = parse(&with_project(
        "[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"github-sponsors\"\n",
    ));
    assert!(codes(&parsed).iter().all(|c| c.0 == Code::Rtoml006));
    assert!(parsed.has_errors());

    // An unknown kind is skipped with a warning; the next method is used.
    let parsed = parse(&with_project(
        "[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"carrier-pigeon\"\n[[payee.method]]\nkind = \"invoice\"\ncontact = \"a@x.example\"\n",
    ));
    assert_eq!(codes(&parsed), vec![(Code::Rtoml006, Level::Warning)]);
    assert_eq!(
        parsed.valid().map(|f| f.payees[0].methods.clone()),
        Some(vec![Method::Invoice {
            contact: "a@x.example".into()
        }])
    );
}

#[test]
fn royl_008_a_value_that_looks_like_a_secret_is_an_error() {
    // Split so secret scanners don't flag this file.
    let token = format!("{}{}", "ghp_", "0123456789abcdefghijABCDEFGHIJ");
    let parsed = parse(&with_project(&format!(
        "[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"github-sponsors\"\naccount = \"{token}\"\n"
    )));
    let secret: Vec<&Finding> = parsed
        .findings
        .iter()
        .filter(|f| f.code == Code::Rtoml007)
        .collect();
    assert_eq!(secret.len(), 1, "{:?}", parsed.findings);
    assert_eq!(secret[0].at, "payee[0].method[0].account");
    assert_eq!(secret[0].requirement, Some("ROYL-008"));
    assert!(parsed.valid().is_none());
}

#[test]
fn royl_002_a_tier_cannot_name_a_non_production_environment() {
    for env in ["development", "test", "ci", "staging"] {
        let parsed = parse(&with_project(&format!(
            "[[tier]]\nid = \"dev\"\nwhen = {{ environment = [\"{env}\"] }}\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n"
        )));
        let finding = parsed.findings.iter().find(|f| f.code == Code::Rtoml008);
        assert!(
            finding.is_some_and(|f| f.requirement == Some("ROYL-002")),
            "{env}: {:?}",
            parsed.findings
        );
        assert!(parsed.valid().is_none());
    }
}

#[test]
fn an_unknown_tier_condition_is_an_error_not_a_warning() {
    let parsed = parse(&with_project(
        "[[tier]]\nid = \"t\"\nwhen = { country = [\"FR\"] }\nfree = true\n",
    ));
    assert!(
        parsed
            .findings
            .iter()
            .any(|f| f.code == Code::Rtoml008 && f.at == "tier[0].when.country"),
        "{:?}",
        parsed.findings
    );
}

#[test]
fn a_condition_can_be_one_word_or_a_list() {
    let parsed = parse(&with_project(
        "[[tier]]\nid = \"t\"\nwhen = { entity = \"organization\", class = [\"commercial\", \"government\"] }\nfree = true\n",
    ));
    let Some(file) = parsed.valid() else {
        panic!("{:?}", parsed.findings);
    };
    assert_eq!(file.tiers[0].when.entity, Some(vec![Entity::Organization]));
    assert_eq!(
        file.tiers[0].when.class,
        Some(vec![Class::Commercial, Class::Government])
    );
}

#[test]
fn unusable_prices_are_errors() {
    for (price, field) in [
        ("unit = \"installs\"\ncents = 1", "unit"),
        ("cents = 1", "unit"),
        ("unit = \"seat\"", "cents"),
        ("unit = \"seat\"\ncents = 1\nper = 0", "per"),
        ("unit = \"revenue\"", "basis_points"),
        ("unit = \"custom:gigabytes\"\ncents = 1", "describe"),
    ] {
        let parsed = parse(&with_project(&format!(
            "[[tier]]\nid = \"t\"\n[[tier.price]]\n{price}\n"
        )));
        assert!(
            parsed
                .findings
                .iter()
                .any(|f| f.code == Code::Rtoml008 && f.at == format!("tier[0].price[0].{field}")),
            "{price}: {:?}",
            parsed.findings
        );
    }
}

#[test]
fn duplicate_tier_ids_are_an_error_and_unreachable_tiers_a_warning() {
    let parsed = parse(&with_project(
        "[[tier]]\nid = \"all\"\nfree = true\n[[tier]]\nid = \"all\"\nfree = true\n",
    ));
    assert_eq!(
        codes(&parsed),
        vec![
            (Code::Rtoml008, Level::Error),
            (Code::Rtoml011, Level::Warning),
            (Code::Rtoml010, Level::Warning),
        ]
    );
}

#[test]
fn royl_009_negative_amounts_and_other_currencies_are_errors() {
    let parsed = parse(&with_project(
        "[terms]\ncurrency = \"EUR\"\n[caps.consumer]\nyear_cents = -5\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://x.example\"\n",
    ));
    let at: Vec<&str> = parsed
        .findings
        .iter()
        .filter(|f| f.code == Code::Rtoml009)
        .map(|f| f.at.as_str())
        .collect();
    assert_eq!(at, vec!["terms.currency", "caps.consumer.year_cents"]);
}

#[test]
fn consumer_fields_are_checked() {
    let parsed = parse(&with_project(
        "[consumer]\nstage = \"live\"\n[[consumer.deployment]]\nname = \"api\"\n[[consumer.package]]\nname = \"x\"\naction = \"cap\"\n",
    ));
    let at: Vec<&str> = parsed
        .findings
        .iter()
        .filter(|f| f.code == Code::Rtoml012)
        .map(|f| f.at.as_str())
        .collect();
    assert_eq!(
        at,
        vec![
            "consumer.stage",
            "consumer.deployment[0].manifest",
            "consumer.package[0].period_cents"
        ]
    );
}

#[test]
fn findings_print_with_code_place_and_requirement() {
    let finding =
        Finding::error(Code::Rtoml008, "tier[0].when.environment[0]", "no").protects("ROYL-002");
    assert_eq!(
        finding.to_string(),
        "error[RTOML-008] tier[0].when.environment[0]: no (ROYL-002)"
    );
}
