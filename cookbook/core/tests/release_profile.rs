//! Guards the workspace release profile that keeps the deployed wasm small
//! (README.md "Release size"). `dx bundle --release` builds with it.

use std::path::Path;

/// The `key = value` lines of one `[section]` of the workspace Cargo.toml.
fn section(manifest: &str, name: &str) -> Vec<(String, String)> {
    let header = format!("[{name}]");
    manifest
        .lines()
        .map(str::trim)
        .skip_while(|line| *line != header)
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}

fn workspace_manifest() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Cargo.toml");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn test_release_profile_optimizes_for_size() {
    let release = section(&workspace_manifest(), "profile.release");
    for (key, value) in [
        ("opt-level", "\"z\""),
        ("lto", "true"),
        ("codegen-units", "1"),
        ("strip", "\"debuginfo\""),
    ] {
        assert!(
            release.iter().any(|(k, v)| k == key && v == value),
            "[profile.release] should set {key} = {value}, found {release:?}"
        );
    }
}

/// `strip = true` drops the `target_features` section wasm-bindgen needs, so
/// `dx bundle --release` fails. Only the main-branch deploy builds the bundle,
/// so this test is what catches it on a pull request.
#[test]
fn test_release_profile_keeps_wasm_bindgen_working() {
    let release = section(&workspace_manifest(), "profile.release");
    assert!(
        !release
            .iter()
            .any(|(k, v)| k == "strip" && (v == "true" || v == "\"symbols\"")),
        "strip = true breaks wasm-bindgen; use strip = \"debuginfo\", found {release:?}"
    );
}

#[test]
fn test_section_stops_at_next_header() {
    let manifest = "[a]\nx = 1\n# note\n\n[b]\ny = 2\n";
    assert_eq!(section(manifest, "a"), vec![("x".into(), "1".into())]);
    assert!(section(manifest, "missing").is_empty());
}
