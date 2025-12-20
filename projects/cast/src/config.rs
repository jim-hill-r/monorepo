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
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_parse_empty_config() {
        let config: CastConfig = toml::from_str("").unwrap();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
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
        };
        
        config.save(&config_path).unwrap();
        
        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, Some(true));
        assert_eq!(loaded_config.proof_of_concept, None);
    }

    #[test]
    fn test_save_config_with_none() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");
        
        let config = CastConfig {
            exemplar: None,
            proof_of_concept: None,
        };
        
        config.save(&config_path).unwrap();
        
        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, None);
        assert_eq!(loaded_config.proof_of_concept, None);
    }

    #[test]
    fn test_default_config() {
        let config = CastConfig::default();
        assert_eq!(config.exemplar, None);
        assert_eq!(config.proof_of_concept, None);
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
        let config: CastConfig = toml::from_str("exemplar = true\nproof_of_concept = false").unwrap();
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
        };
        
        config.save(&config_path).unwrap();
        
        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.proof_of_concept, Some(true));
        assert_eq!(loaded_config.exemplar, None);
    }

    #[test]
    fn test_save_and_load_config_with_both_fields() {
        let tmp_dir = TempDir::new("test_config").unwrap();
        let config_path = tmp_dir.path().join("Cast.toml");
        
        let config = CastConfig {
            exemplar: Some(false),
            proof_of_concept: Some(true),
        };
        
        config.save(&config_path).unwrap();
        
        let loaded_config = CastConfig::load(&config_path).unwrap();
        assert_eq!(loaded_config.exemplar, Some(false));
        assert_eq!(loaded_config.proof_of_concept, Some(true));
    }
}
