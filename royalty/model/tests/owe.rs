//! Caps, the consumer's budget and small amounts (steps 6, 8 and 9).

use royalty_model::*;

fn file(text: &str) -> RoyaltyFile {
    let parsed = parse(text);
    match parsed.valid() {
        Some(file) => file.clone(),
        None => panic!("{text}\n{:?}", parsed.findings),
    }
}

/// A producer with one flat price of `cents`, a binding and extra tables.
fn producer(name: &str, binding: &str, cents: u64, extra: &str) -> RoyaltyFile {
    file(&format!(
        "[project]\nname = \"{name}\"\ndescription = \"A library.\"\n[license]\nbinding = \"{binding}\"\nterms = \"TERMS.md\"\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[terms]\nminimum_payment_cents = 0\n[[tier]]\nid = \"paid\"\n[[tier.price]]\nunit = \"flat\"\ncents = {cents}\n{extra}"
    ))
}

fn consumer(body: &str) -> Consumer {
    let text = format!(
        "[project]\nname = \"app\"\ndescription = \"An app.\"\n[consumer]\nstage = \"production\"\n{body}"
    );
    match file(&text).consumer {
        Some(c) => c,
        None => panic!("no consumer"),
    }
}

fn direct_use<'a>(family: &str, producer: &'a RoyaltyFile) -> FamilyUse<'a> {
    FamilyUse {
        family: family.to_string(),
        producer,
        usage: Usage::production(Dependency::Direct).with(Unit::Flat, 1),
    }
}

fn one(consumer: &Consumer, producer: &RoyaltyFile) -> FamilyOwed {
    let owed = owe(consumer, &[direct_use("lib", producer)]);
    owed.families[0].clone()
}

#[test]
fn royl_004_the_project_cap_then_the_payer_cap() {
    let lib = producer(
        "lib",
        "voluntary",
        9000,
        "[caps.project]\nperiod_cents = 5000\n[caps.organization]\nperiod_cents = 3000\n[caps.consumer]\nperiod_cents = 1000\n",
    );
    let org = one(&consumer("entity = \"organization\""), &lib);
    assert_eq!(org.cents, 3000);
    assert_eq!(org.set_by(), Some(Limit::OrganizationCap));
    assert_eq!(
        org.steps,
        vec![
            Step {
                limit: Limit::ProjectCap,
                cents: 5000
            },
            Step {
                limit: Limit::OrganizationCap,
                cents: 3000
            },
        ]
    );
    // No person cap: a person falls back to the consumer cap.
    let person = one(&consumer("entity = \"person\""), &lib);
    assert_eq!(
        (person.cents, person.set_by()),
        (1000, Some(Limit::ConsumerCap))
    );
}

#[test]
fn royl_004_a_year_cap_limits_the_single_period_until_the_ledger_exists() {
    // The tinycolor example: $1 a month, at most $12 a year.
    let lib = producer(
        "lib",
        "voluntary",
        100,
        "[caps.consumer]\nyear_cents = 1200\n",
    );
    assert_eq!(one(&consumer(""), &lib).cents, 100);
    let lib = producer(
        "lib",
        "voluntary",
        5000,
        "[caps.consumer]\nyear_cents = 1200\n",
    );
    assert_eq!(one(&consumer(""), &lib).cents, 1200);
    // The formkit example: a person never pays.
    let lib = producer("lib", "license", 5000, "[caps.person]\nyear_cents = 0\n");
    let owed = one(&consumer("entity = \"person\""), &lib);
    assert_eq!((owed.cents, owed.status), (0, Status::Free));
}

#[test]
fn royl_004_caps_apply_to_every_binding() {
    let lib = producer(
        "lib",
        "agreement",
        9000,
        "[caps.project]\nperiod_cents = 2000\n",
    );
    let owed = one(&consumer(""), &lib);
    assert_eq!((owed.cents, owed.set_by()), (2000, Some(Limit::ProjectCap)));
}

#[test]
fn royl_004_the_per_producer_budget_lowers_voluntary_and_flags_the_rest() {
    let budget = consumer("[consumer.budget]\nper_producer_period_cents = 1500\n");
    let vol = producer("lib", "voluntary", 4000, "");
    let owed = owe(&budget, &[direct_use("lib", &vol)]);
    assert_eq!(owed.families[0].cents, 1500);
    assert_eq!(owed.families[0].set_by(), Some(Limit::PerProducerBudget));
    assert!(!owed.over_budget);

    let agreement = producer("lib", "agreement", 4000, "");
    let owed = owe(&budget, &[direct_use("lib", &agreement)]);
    assert_eq!(owed.families[0].cents, 4000);
    assert!(owed.families[0].over_budget && owed.over_budget);
}

#[test]
fn royl_004_scale_shares_the_space_left_among_voluntary_amounts() {
    let budget = consumer("[consumer.budget]\nperiod_cents = 5000\non_exceed = \"scale\"\n");
    let a = producer("a", "voluntary", 3000, "");
    let b = producer("b", "voluntary", 6000, "");
    let fixed = producer("fixed", "agreement", 2000, "");
    let owed = owe(
        &budget,
        &[
            direct_use("b", &b),
            direct_use("fixed", &fixed),
            direct_use("a", &a),
        ],
    );
    let names: Vec<&str> = owed.families.iter().map(|f| f.family.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "fixed"], "sorted by family");
    let cents: Vec<u64> = owed.families.iter().map(|f| f.cents).collect();
    // 3000 of space, shared 1:2 between a and b; the agreement is whole.
    assert_eq!(cents, vec![1000, 2000, 2000]);
    assert_eq!(owed.total_cents, 5000);
    assert!(!owed.over_budget);
    assert_eq!(owed.families[0].set_by(), Some(Limit::TotalBudget));
}

#[test]
fn royl_004_scaling_rounds_down_and_never_exceeds_the_budget() {
    let budget = consumer("[consumer.budget]\nperiod_cents = 1000\non_exceed = \"scale\"\n");
    let libs: Vec<RoyaltyFile> = ["a", "b", "c"]
        .iter()
        .map(|n| producer(n, "voluntary", 1000, ""))
        .collect();
    let uses: Vec<FamilyUse<'_>> = ["a", "b", "c"]
        .iter()
        .zip(&libs)
        .map(|(n, lib)| direct_use(n, lib))
        .collect();
    let owed = owe(&budget, &uses);
    assert_eq!(
        owed.families.iter().map(|f| f.cents).collect::<Vec<_>>(),
        vec![333, 333, 333]
    );
    assert!(owed.total_cents <= 1000);
}

#[test]
fn royl_004_an_agreement_over_the_total_budget_is_never_scaled() {
    let budget = consumer("[consumer.budget]\nperiod_cents = 4500\non_exceed = \"scale\"\n");
    let big = producer("big", "agreement", 6000, "");
    let vol = producer("vol", "voluntary", 1000, "");
    let owed = owe(&budget, &[direct_use("big", &big), direct_use("vol", &vol)]);
    assert!(owed.over_budget);
    assert_eq!(owed.families[0].cents, 6000);
    assert!(owed.families[0].over_budget);
    assert_eq!(
        owed.families[1].cents, 0,
        "no space left for voluntary amounts"
    );
}

#[test]
fn royl_004_fail_flags_and_changes_nothing() {
    let budget = consumer("[consumer.budget]\nperiod_cents = 1000\n");
    let a = producer("a", "voluntary", 800, "");
    let b = producer("b", "voluntary", 800, "");
    let owed = owe(&budget, &[direct_use("a", &a), direct_use("b", &b)]);
    assert!(owed.over_budget);
    assert_eq!(owed.total_cents, 1600);
    // Under budget: fine.
    let owed = owe(&budget, &[direct_use("a", &a)]);
    assert!(!owed.over_budget);
}

#[test]
fn exceptions_lower_only_voluntary_amounts() {
    let decline = consumer(
        "[[consumer.package]]\nname = \"lib\"\naction = \"decline\"\nreason = \"We sponsor them directly\"\n",
    );
    let vol = producer("lib", "voluntary", 3000, "");
    let owed = one(&decline, &vol);
    assert_eq!((owed.cents, owed.set_by()), (0, Some(Limit::Declined)));

    let agreement = producer("lib", "agreement", 3000, "");
    let owed = one(&decline, &agreement);
    assert_eq!(owed.cents, 3000);
    assert_eq!(owed.notes.len(), 1);

    let cap =
        consumer("[[consumer.package]]\nname = \"lib\"\naction = \"cap\"\nperiod_cents = 700\n");
    let owed = one(&cap, &vol);
    assert_eq!(
        (owed.cents, owed.set_by()),
        (700, Some(Limit::ConsumerPackageCap))
    );
}

#[test]
fn reach_and_the_transitive_policy() {
    let direct_reach = producer("lib", "voluntary", 3000, "");
    let transitive_use = |p| FamilyUse {
        family: "lib".into(),
        producer: p,
        usage: Usage::production(Dependency::Transitive).with(Unit::Flat, 1),
    };
    let owed = owe(&consumer(""), &[transitive_use(&direct_reach)]);
    assert_eq!(owed.families[0].status, Status::NotBillable);
    assert_eq!(owed.total_cents, 0);

    let reach_all = file(
        "[project]\nname = \"lib\"\ndescription = \"A library.\"\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[terms]\nreach = \"transitive\"\n[[tier]]\nid = \"paid\"\n[[tier.price]]\nunit = \"flat\"\ncents = 3000\n",
    );
    assert_eq!(reach_all.terms.reach, Reach::Transitive);
    let pay = owe(&consumer(""), &[transitive_use(&reach_all)]);
    assert_eq!(pay.total_cents, 3000);
    let report = owe(
        &consumer("[consumer.policy]\ntransitive = \"report\"\n"),
        &[transitive_use(&reach_all)],
    );
    assert_eq!(report.families[0].set_by(), Some(Limit::TransitivePolicy));
    assert_eq!(report.total_cents, 0);
}

#[test]
fn small_amounts_are_held_back() {
    let lib = file(
        "[project]\nname = \"lib\"\ndescription = \"A library.\"\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[[tier]]\nid = \"paid\"\n[[tier.price]]\nunit = \"flat\"\ncents = 499\n",
    );
    let owed = owe(&consumer(""), &[direct_use("lib", &lib)]);
    assert_eq!(owed.families[0].status, Status::HeldBack);
    assert_eq!((owed.families[0].cents, owed.total_cents), (0, 0));
}

#[test]
fn a_binding_the_consumer_does_not_accept_is_flagged() {
    let only_voluntary = consumer("[consumer.policy]\nbindings = [\"voluntary\"]\n");
    let lib = producer("lib", "license", 3000, "");
    assert!(one(&only_voluntary, &lib).binding_refused);
    assert!(!one(&consumer(""), &lib).binding_refused);
}

#[test]
fn royl_004_what_acme_owes_in_one_month() {
    // The table in docs/royalty-toml.md.
    let acme = consumer(
        "id = \"github:acme-example\"\nentity = \"organization\"\nclass = \"commercial\"\nannual_revenue_below_usd = 5000000\n[consumer.budget]\nperiod_cents = 4500\nyear_cents = 50000\nper_producer_period_cents = 4000\non_exceed = \"scale\"\n[consumer.policy]\nbindings = [\"voluntary\", \"agreement\"]\ntransitive = \"report\"\n",
    );
    let webframe = file(
        "[project]\nname = \"webframe\"\ndescription = \"A web framework.\"\n[license]\nbinding = \"agreement\"\nterms = \"COMMERCIAL-TERMS.md\"\n[[payee]]\nname = \"W\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[[tier]]\nid = \"community\"\nwhen = { class = [\"open-source\", \"education\", \"nonprofit\"] }\nfree = true\n[[tier]]\nid = \"small\"\nwhen = { revenue_below_usd = 1000000 }\nfree = true\n[[tier]]\nid = \"production\"\nwhen = { environment = [\"production\"] }\n[[tier.price]]\nunit = \"deployment\"\ncents = 2000\nincluded = 1\n[caps.project]\nperiod_cents = 10000\n[caps.organization]\nperiod_cents = 50000\nyear_cents = 500000\n",
    );
    let chartlib = file(
        "[project]\nname = \"chartlib\"\ndescription = \"Charts.\"\n[[payee]]\nname = \"C\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[[tier]]\nid = \"team\"\n[[tier.price]]\nunit = \"seat\"\ncents = 500\n[caps.project]\nperiod_cents = 4000\n",
    );
    let tinycolor = file(
        "[project]\nname = \"tinycolor\"\ndescription = \"Colours.\"\n[[payee]]\nname = \"T\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n[[tier]]\nid = \"suggested\"\nwhen = { environment = [\"production\"], entity = [\"organization\"] }\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n[caps.consumer]\nyear_cents = 1200\n",
    );
    let uses = [
        FamilyUse {
            family: "webframe".into(),
            producer: &webframe,
            usage: Usage::production(Dependency::Direct).with(Unit::Deployment, 2),
        },
        FamilyUse {
            family: "chartlib".into(),
            producer: &chartlib,
            usage: Usage::production(Dependency::Direct).with(Unit::Seat, 14),
        },
        FamilyUse {
            family: "tinycolor".into(),
            producer: &tinycolor,
            usage: Usage::production(Dependency::Transitive).with(Unit::Flat, 1),
        },
    ];
    let owed = owe(&acme, &uses);
    let rows: Vec<(&str, u64, u64, u64, Status)> = owed
        .families
        .iter()
        .map(|f| {
            (
                f.family.as_str(),
                f.before_caps(),
                f.after_producer_caps(),
                f.cents,
                f.status,
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("chartlib", 7000, 4000, 2500, Status::Owed),
            ("tinycolor", 0, 0, 0, Status::NotBillable),
            ("webframe", 2000, 2000, 2000, Status::Owed),
        ]
    );
    assert_eq!(owed.families[0].set_by(), Some(Limit::TotalBudget));
    assert_eq!(owed.families[2].set_by(), None);
    assert_eq!(owed.total_cents, 4500);
    assert_eq!(owed.budget_cents, Some(4500));
    assert!(!owed.over_budget);
}
