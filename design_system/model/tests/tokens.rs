//! The real token files, and custom themes written inline.

use design_system_model::{Combo, DesignSystem, Scheme, ThemeFile, check, validate};
use std::path::Path;

fn ds() -> DesignSystem {
    DesignSystem::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("..")).unwrap()
}

fn custom(toml_text: &str) -> ThemeFile {
    toml::from_str(toml_text).unwrap()
}

fn failures(ds: &DesignSystem, t: &ThemeFile) -> Vec<String> {
    ds.resolve_all(t)
        .unwrap()
        .iter()
        .flat_map(|r| check(ds, r).unwrap().failures())
        .collect()
}

const BEARS_OK: &str = r##"
name = "bears"
label = "Chicago Bears"
extends = "colorful"

[primitives]
navy = "#0b162a"
orange = "#c83803"
orange-light = "#ff8f5c"

[light.color]
accent = "orange"
accent-hover = "#a32e02"
text = "navy"

[dark.color]
bg = "navy"
surface = "#14223b"
surface-raised = "#172540"
accent = "orange-light"
accent-hover = "#ffab85"

[accessible.light.color]
accent = "#a32e02"
accent-hover = "#8a2702"

[accessible.dark.color]
accent = "#ffa77d"
accent-hover = "#ffc2a6"
"##;

#[test]
fn every_standard_theme_is_complete_and_passes() {
    let ds = ds();
    assert_eq!(ds.themes.len(), 5);
    for t in &ds.themes {
        assert_eq!(validate(t), Vec::<String>::new(), "{}", t.name);
        let all = ds.resolve_all(t).unwrap();
        assert_eq!(all.len(), 4);
        assert_eq!(failures(&ds, t), Vec::<String>::new(), "{}", t.name);
    }
}

#[test]
fn accessible_mode_changes_more_than_colour() {
    let ds = ds();
    let t = ds.theme("gamey").unwrap();
    let all = ds.resolve_all(t).unwrap();
    let get = |c: Combo, k: &str| {
        all.iter()
            .find(|r| r.combo == c)
            .and_then(|r| r.token(k))
            .unwrap()
            .to_string()
    };
    let (std, acc) = (Combo::ALL[0], Combo::ALL[2]);
    assert_eq!(get(std, "target-min"), "24px");
    assert_eq!(get(acc, "target-min"), "44px");
    assert_eq!(get(acc, "motion-fast"), "0ms");
    assert!(get(std, "font-heading").contains("Press Start 2P"));
    assert!(get(acc, "font-heading").contains("Atkinson Hyperlegible"));
    assert_eq!(get(acc, "text-base"), "1.125rem");
}

#[test]
fn a_custom_theme_overrides_only_what_it_sets() {
    let ds = ds();
    let bears = custom(BEARS_OK);
    assert_eq!(validate(&bears), Vec::<String>::new());
    assert_eq!(failures(&ds, &bears), Vec::<String>::new());
    let all = ds.resolve_all(&bears).unwrap();
    let light = &all[0];
    assert_eq!(light.color("accent").unwrap().to_string(), "#c83803");
    // Unset values come from colorful.
    let colorful = ds.resolve_all(ds.theme("colorful").unwrap()).unwrap();
    assert_eq!(light.color("success"), colorful[0].color("success"));
    assert_eq!(light.token("radius-md"), colorful[0].token("radius-md"));
    // on-accent is "bg" in colorful's dark scheme, so it follows the new navy.
    let dark = all.iter().find(|r| r.combo.scheme == Scheme::Dark).unwrap();
    assert_eq!(dark.color("on-accent").unwrap().to_string(), "#0b162a");
}

#[test]
fn a_brand_colour_that_fails_contrast_fails_the_check() {
    let ds = ds();
    // The Bears orange is only 3.5:1 on navy.
    let bad = custom(&BEARS_OK.replace(r#"accent = "orange-light""#, r#"accent = "orange""#));
    let f = failures(&ds, &bad);
    assert!(
        f.iter().any(|m| m.starts_with("bears dark: link on bg")),
        "{f:#?}"
    );
}

#[test]
fn a_custom_theme_cannot_make_an_app_less_usable() {
    let t = custom(
        r##"
name = "tiny"
label = "Tiny"
extends = "airy"
[tokens]
target-min = "12px"
radius-sm = "2px"
"##,
    );
    let problems = validate(&t);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(problems[0].contains("target-min"));
}

#[test]
fn a_custom_theme_must_extend_a_standard_theme() {
    let t = custom("name = \"x\"\nlabel = \"X\"\nextends = \"bears\"\n");
    assert!(validate(&t)[0].contains("not a standard theme"));
}

#[test]
fn a_light_only_custom_theme_has_two_combinations() {
    let ds = ds();
    let t = custom(
        r##"
name = "paper"
label = "Paper"
extends = "professional"
schemes = ["light"]
[light.color]
bg = "#fbfaf7"
"##,
    );
    assert_eq!(validate(&t), Vec::<String>::new());
    let all = ds.resolve_all(&t).unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.iter().all(|r| r.combo.scheme == Scheme::Light));
}

#[test]
fn a_standard_theme_missing_a_token_is_reported() {
    let ds = ds();
    let mut t = ds.theme("earthy").unwrap().clone();
    t.dark.color.remove("info");
    let problems = validate(&t);
    assert_eq!(
        problems,
        vec![r#"[dark.color] doesn't define "info""#.to_string()]
    );
}

#[test]
fn unknown_tokens_and_bad_references_are_reported() {
    let ds = ds();
    let t = custom(
        r##"
name = "oops"
label = "Oops"
extends = "earthy"
[light.color]
acent = "#ff0000"
text = "no-such-primitive"
"##,
    );
    assert!(validate(&t)[0].contains("unknown colour token \"acent\""));
    let err = ds.resolve_all(&t).unwrap_err();
    assert!(err.contains("no-such-primitive"), "{err}");
}
