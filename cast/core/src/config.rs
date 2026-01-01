use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, PartialEq, Default)]
pub struct CastConfig {
    /// Whether this project is an exemplar project (example/template project)
    #[serde(default)]
    pub exemplar: Option<bool>,
    /// Whether this project is a proof of concept project
    #[serde(default)]
    pub proof_of_concept: Option<bool>,
    /// The framework used by the project (e.g., "dioxus", "cloudflare-pages", "cargo")
    #[serde(default)]
    pub framework: Option<String>,
    /// List of projects that are used to deploy this project
    #[serde(default)]
    pub deploys: Option<Vec<String>>,
    /// The type of project (e.g., "static_website", "web_app", "iac", "library", "binary")
    #[serde(default)]
    pub project_type: Option<String>,
    /// The language of the project (e.g., "rust", "typescript")
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<CargoPackage>,
    #[serde(default)]
    lib: Option<CargoLib>,
    #[serde(default)]
    bin: Option<Vec<CargoBin>>,
}

#[derive(Debug, Deserialize)]
struct CargoLib {
    // We just need to know if lib exists
}

#[derive(Debug, Deserialize)]
struct CargoBin {
    // We just need to know if bin exists
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    metadata: Option<CargoMetadata>,
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    cast: Option<CastConfig>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlDeserializeError(#[from] toml::de::Error),
    #[error("TOML serialize error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),
}

impl CastConfig {
    /// Check if this config has any cast metadata set
    pub fn has_cast_metadata(&self) -> bool {
        self.exemplar.is_some()
            || self.proof_of_concept.is_some()
            || self.framework.is_some()
            || self.deploys.is_some()
            || self.project_type.is_some()
            || self.language.is_some()
    }

    /// Load Cast configuration from a directory, checking Cargo.toml first, then Cast.toml
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let dir = dir.as_ref();
        
        let cargo_toml_path = dir.join("Cargo.toml");
        let cast_toml_path = dir.join("Cast.toml");

        // Priority 1: Cargo.toml with cast metadata
        if cargo_toml_path.exists() {
            let config = Self::load_from_cargo_toml(&cargo_toml_path)?;
            
            if config.has_cast_metadata() {
                return Ok(config);
            }
        }
        
        // Priority 2: Cast.toml
        if cast_toml_path.exists() {
            return Self::load(&cast_toml_path);
        }
        
        // Priority 3: Cargo.toml without metadata -> apply defaults
        if cargo_toml_path.exists() {
            let mut config = CastConfig::default();
            
            // Detect project type from Cargo.toml structure
            let contents = fs::read_to_string(&cargo_toml_path)?;
            let cargo_toml: CargoToml = toml::from_str(&contents)?;
            let project_type = Self::detect_project_type(dir, &cargo_toml);
            
            config.project_type = Some(project_type);
            config.language = Some("rust".to_string());
            config.framework = Some("cargo".to_string());
            
            return Ok(config);
        }

        // If no files exist, return default config
        Ok(Self::default())
    }
    
    /// Detect project type from Cargo.toml structure and filesystem
    fn detect_project_type(dir: &Path, cargo_toml: &CargoToml) -> String {
        let has_lib_section = cargo_toml.lib.is_some();
        let has_bin_section = cargo_toml.bin.as_ref().map_or(false, |bins| !bins.is_empty());
        let has_lib_rs = dir.join("src/lib.rs").exists();
        let has_main_rs = dir.join("src/main.rs").exists();
        let has_bin_dir = dir.join("src/bin").exists();
        
        // Determine if it's a library or binary
        let is_library = has_lib_section || has_lib_rs;
        let is_binary = has_bin_section || has_main_rs || has_bin_dir;
        
        // Default to library if both or neither (library is more common)
        if is_library && !is_binary {
            "library".to_string()
        } else if is_binary && !is_library {
            "binary".to_string()
        } else if is_library && is_binary {
            // If it has both, prefer library
            "library".to_string()
        } else {
            // Default to library
            "library".to_string()
        }
    }

    /// Load Cast configuration from Cargo.toml [package.metadata.cast] section
    pub fn load_from_cargo_toml(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let cargo_toml: CargoToml = toml::from_str(&contents)?;

        // Extract cast config using chained option methods
        let cast_config = cargo_toml
            .package
            .and_then(|pkg| pkg.metadata)
            .and_then(|meta| meta.cast)
            .unwrap_or_default();

        Ok(cast_config)
    }

    /// Load a Cast.toml configuration file from the given path
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: CastConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save the configuration to a Cast.toml file at the given path
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_load_from_cargo_toml_with_cast_metadata() {
        let tmp_dir = TempDir::new("test_cargo_metadata").unwrap();
        let cargo_path = tmp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.cast]
exemplar = true
framework = "dioxus"
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        let config = CastConfig::load_from_cargo_toml(&cargo_path).unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.framework, Some("dioxus".to_string()));
    }

    #[test]
    fn test_load_from_cargo_toml_without_cast_metadata() {
        let tmp_dir = TempDir::new("test_cargo_no_metadata").unwrap();
        let cargo_path = tmp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        let config = CastConfig::load_from_cargo_toml(&cargo_path).unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_load_from_dir_prefers_cargo_toml() {
        let tmp_dir = TempDir::new("test_prefer_cargo").unwrap();

        // Create both Cargo.toml and Cast.toml
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.cast]
exemplar = true
framework = "dioxus"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "exemplar = false\nframework = \"rust-library\"",
        )
        .unwrap();

        // Should load from Cargo.toml
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.framework, Some("dioxus".to_string()));
    }

    #[test]
    fn test_load_from_dir_fallback_to_cast_toml() {
        let tmp_dir = TempDir::new("test_fallback_cast").unwrap();

        // Create only Cast.toml
        fs::write(tmp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        assert_eq!(config.exemplar, Some(true));
    }

    #[test]
    fn test_load_from_dir_cargo_toml_without_metadata_fallback_to_cast() {
        let tmp_dir = TempDir::new("test_cargo_no_meta_fallback").unwrap();

        // Create Cargo.toml without cast metadata
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        // Create Cast.toml with config
        fs::write(tmp_dir.path().join("Cast.toml"), "exemplar = true").unwrap();

        // Should fall back to Cast.toml since Cargo.toml has no cast metadata
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        assert_eq!(config.exemplar, Some(true));
    }

    #[test]
    fn test_load_from_dir_returns_default_when_no_files_exist() {
        let tmp_dir = TempDir::new("test_no_config").unwrap();

        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_load_from_cargo_toml_with_all_fields() {
        let tmp_dir = TempDir::new("test_cargo_all_fields").unwrap();
        let cargo_path = tmp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.cast]
exemplar = true
proof_of_concept = false
framework = "dioxus"
deploys = ["deploy1", "deploy2"]
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        let config = CastConfig::load_from_cargo_toml(&cargo_path).unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.proof_of_concept, Some(false));
        assert_eq!(config.framework, Some("dioxus".to_string()));
        assert_eq!(
            config.deploys,
            Some(vec!["deploy1".to_string(), "deploy2".to_string()])
        );
    }

    #[test]
    fn test_parse_empty_config() {
        let config: CastConfig = toml::from_str("").unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_parse_config_with_exemplar_true() {
        let config: CastConfig = toml::from_str("exemplar = true").unwrap();
        assert_eq!(config.exemplar, Some(true));
    }

    #[test]
    fn test_parse_config_with_exemplar_false() {
        let config: CastConfig = toml::from_str("exemplar = false").unwrap();
        assert_eq!(config.exemplar, Some(false));
    }

    #[test]
    fn test_parse_config_without_exemplar() {
        let config: CastConfig = toml::from_str("# Just a comment").unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_load_config_from_file() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        fs::write(&config_path, "exemplar = true").unwrap();

        let config = CastConfig::load(&config_path).unwrap();
        assert_eq!(config.exemplar, Some(true));
    }

    #[test]
    fn test_load_empty_config_from_file() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        fs::write(&config_path, "").unwrap();

        let config = CastConfig::load(&config_path).unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_load_config_with_comment_from_file() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        fs::write(&config_path, "# Cast configuration\nexemplar = false").unwrap();

        let config = CastConfig::load(&config_path).unwrap();
        assert_eq!(config.exemplar, Some(false));
    }

    #[test]
    fn test_save_config() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: Some(true),
            proof_of_concept: None,
            framework: None,
            deploys: None,
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, Some(true));
        assert_eq!(loaded_config.proof_of_concept, None);
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.deploys, None);
        assert_eq!(loaded_config.project_type, None);
    }

    #[test]
    fn test_save_config_with_none() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: None,
            proof_of_concept: None,
            framework: None,
            deploys: None,
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.proof_of_concept, None);
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.deploys, None);
        assert_eq!(loaded_config.project_type, None);
        assert_eq!(loaded_config.language, None);
    }

    #[test]
    fn test_default_config() {
        let config = CastConfig::default();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
        assert_eq!(config.project_type, None);
    }

    #[test]
    fn test_load_nonexistent_file_returns_error() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("nonexistent.toml");

        let result = CastConfig::load(&config_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_config_with_proof_of_concept_true() {
        let config: CastConfig = toml::from_str("proof_of_concept = true").unwrap();
        assert_eq!(config.proof_of_concept, Some(true));
        assert_eq!(config.exemplar, None);
    }

    #[test]
    fn test_parse_config_with_proof_of_concept_false() {
        let config: CastConfig = toml::from_str("proof_of_concept = false").unwrap();
        assert_eq!(config.proof_of_concept, Some(false));
        assert_eq!(config.exemplar, None);
    }

    #[test]
    fn test_parse_config_with_both_fields() {
        let config: CastConfig =
            toml::from_str("exemplar = true\nproof_of_concept = false").unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.proof_of_concept, Some(false));
    }

    #[test]
    fn test_load_config_with_proof_of_concept_from_file() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        fs::write(&config_path, "proof_of_concept = true").unwrap();

        let config = CastConfig::load(&config_path).unwrap();
        assert_eq!(config.proof_of_concept, Some(true));
        assert_eq!(config.exemplar, None);
    }

    #[test]
    fn test_save_config_with_proof_of_concept() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: None,
            proof_of_concept: Some(true),
            framework: None,
            deploys: None,
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.proof_of_concept, Some(true));
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.deploys, None);
        assert_eq!(loaded_config.project_type, None);
    }

    #[test]
    fn test_save_and_load_config_with_both_fields() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: Some(false),
            proof_of_concept: Some(true),
            framework: None,
            deploys: None,
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, Some(false));
        assert_eq!(loaded_config.proof_of_concept, Some(true));
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.deploys, None);
        assert_eq!(loaded_config.project_type, None);
    }

    #[test]
    fn test_parse_config_with_framework() {
        let config: CastConfig = toml::from_str("framework = \"dioxus\"").unwrap();
        assert_eq!(config.framework, Some("dioxus".to_string()));
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
    }

    #[test]
    fn test_parse_config_with_all_fields() {
        let config: CastConfig =
            toml::from_str("exemplar = true\nproof_of_concept = false\nframework = \"dioxus\"")
                .unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.proof_of_concept, Some(false));
        assert_eq!(config.framework, Some("dioxus".to_string()));
    }

    #[test]
    fn test_save_config_with_framework() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: None,
            proof_of_concept: None,
            framework: Some("dioxus".to_string()),
            deploys: None,
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.framework, Some("dioxus".to_string()));
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.proof_of_concept, None);
        assert_eq!(loaded_config.deploys, None);
        assert_eq!(loaded_config.project_type, None);
    }

    #[test]
    fn test_save_and_load_config_with_different_frameworks() {
        let tmp_dir = TempDir::new("test_config").unwrap();

        let test_cases = vec!["dioxus", "cloudflare-pages", "rust-library", "rust-binary"];

        for framework in test_cases {
            let config_path = tmp_dir.path().join(format!("{}.toml", framework));

            let config = CastConfig {
                exemplar: None,
                proof_of_concept: None,
                framework: Some(framework.to_string()),
                deploys: None,
                project_type: None,
                language: None,
            };

            config.save(&config_path).unwrap();

            let loaded_config = CastConfig::load(&config_path).unwrap();
            assert_eq!(
                loaded_config.framework,
                Some(framework.to_string()),
                "Failed for framework: {}",
                framework
            );
        }
    }

    #[test]
    fn test_moved_poc_projects_have_proof_of_concept_flag() {
        // Test that all moved proof-of-concept projects have Cast.toml with proof_of_concept = true
        let poc_projects = vec!["dioxus_ssg", "dioxus_static_website", "marp"];

        // CARGO_MANIFEST_DIR is now cast_workspace/core, so go up two levels to reach monorepo root
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        for project in poc_projects {
            let cast_toml = root.join(project).join("Cast.toml");
            let config = CastConfig::load(&cast_toml)
                .unwrap_or_else(|e| panic!("Failed to load {} Cast.toml: {}", project, e));

            assert_eq!(
                config.proof_of_concept,
                Some(true),
                "Expected proof_of_concept to be true for {}",
                project
            );
        }
    }

    #[test]
    fn test_parse_config_with_deploys() {
        let config: CastConfig = toml::from_str("deploys = [\"project-deploy\"]").unwrap();
        assert_eq!(config.deploys, Some(vec!["project-deploy".to_string()]));
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
    }

    #[test]
    fn test_parse_config_with_multiple_deploys() {
        let config: CastConfig =
            toml::from_str("deploys = [\"deploy1\", \"deploy2\", \"deploy3\"]").unwrap();
        assert_eq!(
            config.deploys,
            Some(vec![
                "deploy1".to_string(),
                "deploy2".to_string(),
                "deploy3".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_config_with_empty_deploys() {
        let config: CastConfig = toml::from_str("deploys = []").unwrap();
        assert_eq!(config.deploys, Some(vec![]));
    }

    #[test]
    fn test_save_config_with_deploys() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: None,
            proof_of_concept: None,
            framework: None,
            deploys: Some(vec!["pane-cloudflare".to_string()]),
            project_type: None,
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(
            loaded_config.deploys,
            Some(vec!["pane-cloudflare".to_string()])
        );
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.proof_of_concept, None);
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.project_type, None);
    }

    #[test]
    fn test_parse_config_with_all_fields_including_deploys() {
        let config: CastConfig = toml::from_str(
            "exemplar = true\nproof_of_concept = false\nframework = \"dioxus\"\ndeploys = [\"deploy-project\"]"
        ).unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.proof_of_concept, Some(false));
        assert_eq!(config.framework, Some("dioxus".to_string()));
        assert_eq!(config.deploys, Some(vec!["deploy-project".to_string()]));
    }

    #[test]
    fn test_parse_config_with_project_type() {
        let config: CastConfig = toml::from_str("project_type = \"static_website\"").unwrap();
        assert_eq!(config.project_type, Some("static_website".to_string()));
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
        assert_eq!(config.framework, None);
        assert_eq!(config.deploys, None);
    }

    #[test]
    fn test_parse_config_with_all_project_types() {
        let project_types = vec!["static_website", "web_app", "iac", "library", "binary"];

        for project_type in project_types {
            let config: CastConfig =
                toml::from_str(&format!("project_type = \"{}\"", project_type)).unwrap();
            assert_eq!(
                config.project_type,
                Some(project_type.to_string()),
                "Failed for project_type: {}",
                project_type
            );
        }
    }

    #[test]
    fn test_save_config_with_project_type() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");

        let config = CastConfig {
            exemplar: None,
            proof_of_concept: None,
            framework: None,
            deploys: None,
            project_type: Some("static_website".to_string()),
            language: None,
        };

        config.save(&config_path).unwrap();

        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(
            loaded_config.project_type,
            Some("static_website".to_string())
        );
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.proof_of_concept, None);
        assert_eq!(loaded_config.framework, None);
        assert_eq!(loaded_config.deploys, None);
    }

    #[test]
    fn test_parse_config_with_all_fields_including_project_type() {
        let config: CastConfig = toml::from_str(
            "exemplar = true\nproof_of_concept = false\nframework = \"dioxus\"\ndeploys = [\"deploy-project\"]\nproject_type = \"web_app\""
        ).unwrap();
        assert_eq!(config.exemplar, Some(true));
        assert_eq!(config.proof_of_concept, Some(false));
        assert_eq!(config.framework, Some("dioxus".to_string()));
        assert_eq!(config.deploys, Some(vec!["deploy-project".to_string()]));
        assert_eq!(config.project_type, Some("web_app".to_string()));
    }

    #[test]
    fn test_load_from_cargo_toml_with_project_type() {
        let tmp_dir = TempDir::new("test_cargo_project_type").unwrap();
        let cargo_path = tmp_dir.path().join("Cargo.toml");

        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"

[package.metadata.cast]
project_type = "library"
"#;
        fs::write(&cargo_path, cargo_content).unwrap();

        let config = CastConfig::load_from_cargo_toml(&cargo_path).unwrap();
        assert_eq!(config.project_type, Some("library".to_string()));
    }

    #[test]
    fn test_cargo_toml_without_metadata_gets_defaults() {
        let tmp_dir = TempDir::new("test_cargo_defaults").unwrap();
        
        // Create Cargo.toml without cast metadata
        let cargo_content = r#"
[package]
name = "test_lib"
version = "0.1.0"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        // Create src/lib.rs to indicate it's a library
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "// library code").unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should have defaults applied
        assert_eq!(config.project_type, Some("library".to_string()));
        assert_eq!(config.language, Some("rust".to_string()));
        assert_eq!(config.framework, Some("cargo".to_string()));
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
    }

    #[test]
    fn test_cargo_toml_detects_binary_project() {
        let tmp_dir = TempDir::new("test_cargo_binary").unwrap();
        
        // Create Cargo.toml without cast metadata
        let cargo_content = r#"
[package]
name = "test_bin"
version = "0.1.0"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        // Create src/main.rs to indicate it's a binary
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should detect as binary
        assert_eq!(config.project_type, Some("binary".to_string()));
        assert_eq!(config.language, Some("rust".to_string()));
        assert_eq!(config.framework, Some("cargo".to_string()));
    }

    #[test]
    fn test_cargo_toml_with_both_lib_and_bin_prefers_library() {
        let tmp_dir = TempDir::new("test_cargo_both").unwrap();
        
        // Create Cargo.toml without cast metadata
        let cargo_content = r#"
[package]
name = "test_both"
version = "0.1.0"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        // Create both src/lib.rs and src/main.rs
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "// library code").unwrap();
        fs::write(tmp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should prefer library when both exist
        assert_eq!(config.project_type, Some("library".to_string()));
        assert_eq!(config.language, Some("rust".to_string()));
        assert_eq!(config.framework, Some("cargo".to_string()));
    }

    #[test]
    fn test_cargo_toml_with_lib_section() {
        let tmp_dir = TempDir::new("test_cargo_lib_section").unwrap();
        
        // Create Cargo.toml with [lib] section
        let cargo_content = r#"
[package]
name = "test_lib"
version = "0.1.0"

[lib]
name = "test_lib"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should detect as library based on [lib] section
        assert_eq!(config.project_type, Some("library".to_string()));
    }

    #[test]
    fn test_cargo_toml_with_bin_section() {
        let tmp_dir = TempDir::new("test_cargo_bin_section").unwrap();
        
        // Create Cargo.toml with [[bin]] section
        let cargo_content = r#"
[package]
name = "test_bin"
version = "0.1.0"

[[bin]]
name = "test_bin"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should detect as binary based on [[bin]] section
        assert_eq!(config.project_type, Some("binary".to_string()));
    }

    #[test]
    fn test_cast_toml_takes_precedence_over_cargo_defaults() {
        let tmp_dir = TempDir::new("test_cast_precedence").unwrap();
        
        // Create Cargo.toml without metadata
        let cargo_content = r#"
[package]
name = "test"
version = "0.1.0"
"#;
        fs::write(tmp_dir.path().join("Cargo.toml"), cargo_content).unwrap();
        
        // Create Cast.toml with custom config
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "framework = \"dioxus\"\nproject_type = \"web_app\"",
        )
        .unwrap();
        
        let config = CastConfig::load_from_dir(tmp_dir.path()).unwrap();
        
        // Should use Cast.toml values, not defaults
        assert_eq!(config.framework, Some("dioxus".to_string()));
        assert_eq!(config.project_type, Some("web_app".to_string()));
        assert_eq!(config.language, None); // Not set in Cast.toml
    }
}
