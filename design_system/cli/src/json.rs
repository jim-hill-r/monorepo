//! `dist/tokens.json`, in the W3C Design Tokens Community Group format.

use design_system_model::Resolved;
use serde_json::{Map, Value, json};

fn token_type(name: &str) -> Option<&'static str> {
    if name.starts_with("font-") {
        Some("fontFamily")
    } else if name.starts_with("motion-") {
        Some("duration")
    } else if name == "ease" {
        Some("cubicBezier")
    } else if name.starts_with("weight-") {
        Some("fontWeight")
    } else if name.starts_with("leading-") {
        Some("number")
    } else if name.starts_with("shadow-") {
        None
    } else {
        Some("dimension")
    }
}

fn combo(r: &Resolved) -> Value {
    let mut color = Map::new();
    for (k, v) in &r.colors {
        color.insert(
            k.clone(),
            json!({ "$type": "color", "$value": v.to_string() }),
        );
    }
    let mut out = Map::new();
    out.insert("color".into(), Value::Object(color));
    for (k, v) in &r.tokens {
        let mut t = Map::new();
        if let Some(ty) = token_type(k) {
            t.insert("$type".into(), json!(ty));
        }
        t.insert("$value".into(), json!(v));
        out.insert(k.clone(), Value::Object(t));
    }
    Value::Object(out)
}

/// One theme per top-level group, one group per combination inside it.
pub fn tokens_json(themes: &[(String, String, Vec<Resolved>)]) -> String {
    let mut root = Map::new();
    root.insert(
        "$description".into(),
        json!(format!("{} Values are CSS strings.", crate::css::HEADER)),
    );
    let combos: Vec<String> = themes
        .first()
        .map(|(_, _, all)| all.iter().map(|r| r.combo.slug()).collect())
        .unwrap_or_default();
    root.insert(
        "$extensions".into(),
        json!({ "design_system": {
            "themes": themes.iter().map(|(n, _, _)| n).collect::<Vec<_>>(),
            "combinations": combos,
            "attributes": { "theme": "data-theme", "scheme": "data-scheme", "accessible": "data-a11y" },
        }}),
    );
    for (name, label, all) in themes {
        let mut group = Map::new();
        group.insert("$description".into(), json!(label));
        for r in all {
            group.insert(r.combo.slug(), combo(r));
        }
        root.insert(name.clone(), Value::Object(group));
    }
    let mut s = serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_default();
    s.push('\n');
    s
}
