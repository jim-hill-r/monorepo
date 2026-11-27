//! The release export uses the wasm-bindgen CLI that mise installs, and
//! wasm-bindgen refuses to run when the CLI and the crate versions differ,
//! so a Cargo.lock bump must bump `mise.toml` too.

use std::fs;

fn lock_version(lock: &str, name: &str) -> Option<String> {
    let mut lines = lock.lines();
    while let Some(line) = lines.next() {
        if line == format!("name = \"{name}\"") {
            let version = lines.next()?;
            return Some(
                version
                    .strip_prefix("version = ")?
                    .trim_matches('"')
                    .to_string(),
            );
        }
    }
    None
}

#[test]
fn wasm_bindgen_cli_matches_cargo_lock() {
    let lock = fs::read_to_string("Cargo.lock").unwrap_or_default();
    let mise = fs::read_to_string("mise.toml").unwrap_or_default();
    let version = lock_version(&lock, "wasm-bindgen");
    assert!(version.is_some(), "no wasm-bindgen in Cargo.lock");
    let pin = format!(
        "\"github:wasm-bindgen/wasm-bindgen\" = \"{}\"",
        version.unwrap_or_default()
    );
    assert!(
        mise.contains(&pin),
        "mise.toml must pin {pin} to match Cargo.lock"
    );
}

#[test]
fn cargo_lock_stays_in_format_3() {
    // perseus-cli 0.4.2 can't read lockfile format 4.
    let lock = fs::read_to_string("Cargo.lock").unwrap_or_default();
    assert!(lock.contains("\nversion = 3\n"));
}
