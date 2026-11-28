//! Steps 3 to 5: environment, tier and price.

use royalty_model::*;

fn producer(body: &str) -> RoyaltyFile {
    let text = format!(
        "[project]\nname = \"lib\"\ndescription = \"A library.\"\npackages = [\"lib\", \"lib-macros\"]\n[[payee]]\nname = \"A\"\n[[payee.method]]\nkind = \"link\"\nurl = \"https://pay.example\"\n{body}"
    );
    let parsed = parse(&text);
    match parsed.valid() {
        Some(file) => file.clone(),
        None => panic!("{text}\n{:?}", parsed.findings),
    }
}

fn consumer(body: &str) -> Consumer {
    let text = format!(
        "[project]\nname = \"app\"\ndescription = \"An app.\"\n[consumer]\nstage = \"production\"\n{body}"
    );
    let parsed = parse(&text);
    match parsed.valid().and_then(|f| f.consumer.clone()) {
        Some(consumer) => consumer,
        None => panic!("{text}\n{:?}", parsed.findings),
    }
}

fn direct() -> Usage {
    Usage::production(Dependency::Direct)
}

fn tier_id(priced: &Priced) -> Option<&str> {
    match &priced.tier {
        TierChoice::Tier { id, .. } => Some(id),
        _ => None,
    }
}

const FLAT_100: &str = "[[tier]]\nid = \"paid\"\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n";

#[test]
fn royl_002_non_production_is_never_charged() {
    let lib = producer(FLAT_100);
    // A development-stage consumer.
    let dev = consumer("").clone();
    let dev = Consumer {
        stage: Stage::Development,
        ..dev
    };
    let priced = price(&lib, &dev, &direct().with(Unit::Flat, 1));
    assert_eq!((priced.tier, priced.cents), (TierChoice::NonProduction, 0));

    // A production consumer, but the package is only a dev or build dependency.
    let usage = Usage {
        production: false,
        ..direct().with(Unit::Flat, 1)
    };
    let priced = price(&lib, &consumer(""), &usage);
    assert_eq!((priced.tier, priced.cents), (TierChoice::NonProduction, 0));

    // A consumer with no [consumer] table at all is in development.
    let priced = price(&lib, &Consumer::default(), &direct().with(Unit::Flat, 1));
    assert_eq!(priced.cents, 0);
}

#[test]
fn the_first_matching_tier_applies_and_no_match_is_free() {
    let lib = producer(
        "[[tier]]\nid = \"orgs\"\nwhen = { entity = [\"organization\"] }\n[[tier.price]]\nunit = \"flat\"\ncents = 500\n[[tier]]\nid = \"everyone\"\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n",
    );
    let usage = direct().with(Unit::Flat, 1);
    let org = consumer("entity = \"organization\"");
    let person = consumer("entity = \"person\"");
    assert_eq!(price(&lib, &org, &usage).cents, 500);
    assert_eq!(price(&lib, &person, &usage).cents, 100);

    let only_orgs = producer(
        "[[tier]]\nid = \"orgs\"\nwhen = { entity = [\"organization\"] }\n[[tier.price]]\nunit = \"flat\"\ncents = 500\n",
    );
    let priced = price(&only_orgs, &person, &usage);
    assert_eq!((priced.tier, priced.cents), (TierChoice::NoMatch, 0));
}

#[test]
fn a_free_tier_and_a_tier_without_prices_cost_nothing() {
    let lib = producer(
        "[[tier]]\nid = \"oss\"\nfree = true\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n",
    );
    let priced = price(&lib, &consumer(""), &direct().with(Unit::Flat, 1));
    assert_eq!(tier_id(&priced), Some("oss"));
    assert_eq!(priced.cents, 0);
    let lib = producer("[[tier]]\nid = \"empty\"\n");
    assert_eq!(price(&lib, &consumer(""), &direct()).cents, 0);
}

#[test]
fn each_condition_matches_as_documented() {
    let cases: &[(&str, &str, bool)] = &[
        ("environment = [\"production\"]", "", true),
        (
            "class = [\"open-source\", \"education\"]",
            "class = \"education\"",
            true,
        ),
        ("class = [\"open-source\"]", "", false), // default class is commercial
        (
            "revenue_below_usd = 1000000",
            "annual_revenue_below_usd = 500000",
            true,
        ),
        (
            "revenue_below_usd = 1000000",
            "annual_revenue_below_usd = 1000000",
            true,
        ),
        (
            "revenue_below_usd = 1000000",
            "annual_revenue_below_usd = 5000000",
            false,
        ),
        ("revenue_below_usd = 1000000", "", false), // no band declared
        ("seats_at_most = 5", "seats = 5", true),
        ("seats_at_most = 5", "seats = 6", false),
        ("seats_at_most = 5", "", false), // seats not known
        ("dependency = [\"direct\"]", "", true),
        ("dependency = [\"transitive\"]", "", false),
    ];
    for (when, consumer_body, expected) in cases {
        let lib = producer(&format!(
            "[[tier]]\nid = \"t\"\nwhen = {{ {when} }}\n[[tier.price]]\nunit = \"flat\"\ncents = 100\n"
        ));
        let priced = price(
            &lib,
            &consumer(consumer_body),
            &direct().with(Unit::Flat, 1),
        );
        assert_eq!(
            priced.cents == 100,
            *expected,
            "when {{ {when} }} with {consumer_body:?}"
        );
    }
}

#[test]
fn seats_at_most_uses_the_measured_count_when_none_is_declared() {
    let lib = producer(
        "[[tier]]\nid = \"small\"\nwhen = { seats_at_most = 5 }\nfree = true\n[[tier]]\nid = \"big\"\n[[tier.price]]\nunit = \"seat\"\ncents = 100\n",
    );
    let few = direct().with(Unit::Seat, 4);
    let many = direct().with(Unit::Seat, 9);
    assert_eq!(tier_id(&price(&lib, &consumer(""), &few)), Some("small"));
    assert_eq!(price(&lib, &consumer(""), &many).cents, 900);
    // The consumer's own number wins.
    assert_eq!(price(&lib, &consumer("seats = 2"), &many).cents, 0);
}

#[test]
fn every_counted_unit_is_priced_from_its_count() {
    for unit in [
        "flat",
        "project",
        "seat",
        "deployment",
        "build",
        "release",
        "call",
    ] {
        let lib = producer(&format!(
            "[[tier]]\nid = \"t\"\n[[tier.price]]\nunit = \"{unit}\"\ncents = 250\n"
        ));
        let Some(u) = Unit::parse(unit) else {
            panic!("{unit}");
        };
        let priced = price(&lib, &consumer(""), &direct().with(u, 3));
        assert_eq!(priced.cents, 750, "{unit}");
    }
}

#[test]
fn included_and_max_units_limit_what_is_charged() {
    // The formkit example: $5 a seat, 3 included, at most 100 seats counted.
    let lib = producer(
        "[[tier]]\nid = \"team\"\n[[tier.price]]\nunit = \"seat\"\ncents = 500\nincluded = 3\nmax_units = 100\n",
    );
    let at = |seats: u64| price(&lib, &consumer(""), &direct().with(Unit::Seat, seats)).cents;
    assert_eq!(at(2), 0);
    assert_eq!(at(3), 0);
    assert_eq!(at(14), 11 * 500);
    assert_eq!(at(100), 97 * 500);
    assert_eq!(at(5000), 97 * 500);
}

#[test]
fn royl_009_per_is_exact_and_rounds_down_once_per_family() {
    // $1 per million calls.
    let lib = producer(
        "[[tier]]\nid = \"t\"\n[[tier.price]]\nunit = \"call\"\ncents = 100\nper = 1000000\n",
    );
    let at = |calls: u64| price(&lib, &consumer(""), &direct().with(Unit::Call, calls));
    assert_eq!(at(1_000_000).cents, 100);
    assert_eq!(at(1_499_999).cents, 149); // 149.9999 cents rounds down
    assert_eq!(at(9_999).cents, 0);
    assert_eq!(at(9_999).lines[0].millionths, 999_900);

    // Two prices of 0.6 cents each make 1 cent, not 0: rounding is once.
    let lib = producer(
        "[[tier]]\nid = \"t\"\n[[tier.price]]\nunit = \"call\"\ncents = 6\nper = 10\n[[tier.price]]\nunit = \"build\"\ncents = 6\nper = 10\n",
    );
    let usage = direct().with(Unit::Call, 1).with(Unit::Build, 1);
    assert_eq!(price(&lib, &consumer(""), &usage).cents, 1);
}

#[test]
fn royl_009_revenue_is_basis_points_of_declared_cents() {
    // The geocalc example: 0.1% of attributed revenue.
    let lib = producer(
        "[[tier]]\nid = \"commercial\"\n[[tier.price]]\nunit = \"revenue\"\nbasis_points = 10\n",
    );
    let declared = consumer(
        "[[consumer.package]]\nname = \"lib\"\naction = \"declare\"\nunit = \"revenue\"\nunits = 12345678\n",
    );
    let priced = price(&lib, &declared, &direct());
    assert_eq!(priced.cents, 12_345); // 12,345.678 rounds down
    assert!(priced.lines[0].declared);
}

#[test]
fn custom_units_and_unmeasured_units() {
    let lib = producer(
        "[[tier]]\nid = \"t\"\n[[tier.price]]\nunit = \"custom:gigabytes\"\ndescribe = \"One gigabyte processed\"\ncents = 2\n[[tier.price]]\nunit = \"deployment\"\ncents = 1000\n",
    );
    // Nothing measured, nothing declared: "not measured", never guessed.
    let priced = price(&lib, &consumer(""), &direct());
    assert_eq!(priced.cents, 0);
    assert!(priced.lines.iter().all(|l| l.units.is_none()));

    // Declared by family or package name, with - and _ the same.
    let declared = consumer(
        "[[consumer.package]]\nname = \"lib_macros\"\naction = \"declare\"\nunit = \"custom:gigabytes\"\nunits = 50\n",
    );
    let priced = price(&lib, &declared, &direct().with(Unit::Deployment, 2));
    assert_eq!(priced.cents, 100 + 2000);
}

#[test]
fn what_acme_owes_before_caps() {
    // The table in docs/royalty-toml.md: webframe $20.00 and chartlib $70.00
    // before caps.
    let acme = consumer(
        "entity = \"organization\"\nclass = \"commercial\"\nannual_revenue_below_usd = 5000000\n",
    );
    let webframe = producer(
        "[[tier]]\nid = \"community\"\nwhen = { class = [\"open-source\", \"education\", \"nonprofit\"] }\nfree = true\n[[tier]]\nid = \"small\"\nwhen = { revenue_below_usd = 1000000 }\nfree = true\n[[tier]]\nid = \"production\"\nwhen = { environment = [\"production\"] }\n[[tier.price]]\nunit = \"deployment\"\ncents = 2000\nincluded = 1\n",
    );
    let priced = price(&webframe, &acme, &direct().with(Unit::Deployment, 2));
    assert_eq!((tier_id(&priced), priced.cents), (Some("production"), 2000));

    let chartlib =
        producer("[[tier]]\nid = \"team\"\n[[tier.price]]\nunit = \"seat\"\ncents = 500\n");
    let priced = price(&chartlib, &acme, &direct().with(Unit::Seat, 14));
    assert_eq!((tier_id(&priced), priced.cents), (Some("team"), 7000));
}
