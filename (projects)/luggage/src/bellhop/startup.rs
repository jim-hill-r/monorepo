use crate::{closet::closet::ClosetType, error::Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::BufReader};

const STARTUP_CONFIGURATION_ENV_VARIABLE_NAME: &str = "LUGGAGE_BELLHOP_STARTUP_CONFIG_PATH";

#[derive(Serialize, Deserialize)]
pub struct StartupConfiguration {
    pub name: Option<String>,
    pub closet_type: ClosetType,
}

impl StartupConfiguration {
    pub fn new() -> Self {
        return Self {
            name: Some("default".into()),
            closet_type: ClosetType::LocalSurrealDb,
        };
    }
}

pub fn get_startup_configuration() -> Result<StartupConfiguration> {
    if let Ok(config_path) = env::var(STARTUP_CONFIGURATION_ENV_VARIABLE_NAME) {
        let file = File::open(config_path)?;
        let reader = BufReader::new(file);
        let startup_config: StartupConfiguration = serde_json::from_reader(reader)?;
        return Ok(startup_config);
    };

    return Ok(StartupConfiguration::new());
}

#[cfg(test)]
mod tests {
    use crate::error::Result;

    use super::*;

    #[test]
    fn get_default_config() -> Result<()> {
        let startup_config = get_startup_configuration()?;
        assert_eq!(startup_config.name, Some("default".into()));
        assert_eq!(startup_config.closet_type, ClosetType::LocalSurrealDb);
        return Ok(());
    }

    #[ignore = "unsafe env changes"]
    #[test]
    fn get_config_from_env() -> Result<()> {
        unsafe {
            env::set_var(
                STARTUP_CONFIGURATION_ENV_VARIABLE_NAME,
                "./tests/assets/test_config.json",
            );
        }
        let startup_config = get_startup_configuration()?;
        assert_eq!(startup_config.name, Some("local_surreal_db_test".into()));
        assert_eq!(startup_config.closet_type, ClosetType::LocalSurrealDb);
        unsafe {
            env::remove_var(STARTUP_CONFIGURATION_ENV_VARIABLE_NAME);
        }
        return Ok(());
    }

    #[test]
    fn parse_config() -> Result<()> {
        let input = include_str!("../../tests/assets/test_config.json");
        let startup_config: StartupConfiguration = serde_json::from_str(input)?;
        assert_eq!(startup_config.closet_type, ClosetType::LocalSurrealDb);
        return Ok(());
    }
}
