use cast::config::CastConfig;
use std::fs;
use tempdir::TempDir;

fn main() {
    println!("Cast Configuration Loading Examples\n");

    // Example 1: Load from Cast.toml
    println!("Example 1: Loading from Cast.toml");
    let tmp_dir = TempDir::new("cast_example").unwrap();
    let cast_toml_path = tmp_dir.path().join("Cast.toml");
    fs::write(
        &cast_toml_path,
        r#"exemplar = true
framework = "dioxus"
deploys = ["cloudflare-deploy"]"#,
    )
    .unwrap();

    let config = CastConfig::load(&cast_toml_path).unwrap();
    println!("  Exemplar: {:?}", config.exemplar);
    println!("  Framework: {:?}", config.framework);
    println!("  Deploys: {:?}\n", config.deploys);

    // Example 2: Load from Cargo.toml with [package.metadata.cast]
    println!("Example 2: Loading from Cargo.toml [package.metadata.cast]");
    let cargo_toml_path = tmp_dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml_path,
        r#"[package]
name = "example-project"
version = "0.1.0"
edition = "2021"

[package.metadata.cast]
exemplar = false
proof_of_concept = true
framework = "rust-library""#,
    )
    .unwrap();

    let config = CastConfig::load_from_cargo_toml(&cargo_toml_path).unwrap();
    println!("  Exemplar: {:?}", config.exemplar);
    println!("  Proof of Concept: {:?}", config.proof_of_concept);
    println!("  Framework: {:?}\n", config.framework);

    // Example 3: Load from directory (checks Cargo.toml first, then Cast.toml)
    println!("Example 3: Loading from directory (automatic detection)");
    let project_dir = TempDir::new("cast_project").unwrap();

    // Create Cargo.toml with cast metadata
    fs::write(
        project_dir.path().join("Cargo.toml"),
        r#"[package]
name = "auto-detect"
version = "0.1.0"

[package.metadata.cast]
framework = "cloudflare-pages""#,
    )
    .unwrap();

    let config = CastConfig::load_from_dir(project_dir.path()).unwrap();
    println!("  Framework: {:?}\n", config.framework);

    // Example 4: Fallback to Cast.toml when Cargo.toml has no cast metadata
    println!("Example 4: Fallback to Cast.toml");
    let fallback_dir = TempDir::new("cast_fallback").unwrap();

    // Create Cargo.toml without cast metadata
    fs::write(
        fallback_dir.path().join("Cargo.toml"),
        r#"[package]
name = "fallback-test"
version = "0.1.0""#,
    )
    .unwrap();

    // Create Cast.toml with config
    fs::write(fallback_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

    let config = CastConfig::load_from_dir(fallback_dir.path()).unwrap();
    println!(
        "  Exemplar: {:?} (loaded from Cast.toml)\n",
        config.exemplar
    );

    println!("All examples completed successfully!");
}
