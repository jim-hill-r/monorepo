use crate::{closet::closet::Closet, error::Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::BufReader};

const STARTUP_CONFIGURATION_ENV_VARIABLE_NAME: &str = "LUGGAGE_BELLHOP_STARTUP_CONFIG_PATH";

#[derive(Serialize, Deserialize, Default)]
pub struct StartupConfiguration {
    pub name: Option<String>,
    pub closet: Option<Closet>,
}

pub fn get_startup_configuration() -> Result<StartupConfiguration> {
    if let Ok(config_path) = env::var(STARTUP_CONFIGURATION_ENV_VARIABLE_NAME) {
        let file = File::open(config_path)?;
        let reader = BufReader::new(file);
        let startup_config: StartupConfiguration = serde_json::from_reader(reader)?;
        return Ok(startup_config);
    };

    return Ok(StartupConfiguration::default());
}

#[cfg(test)]
mod tests {
    use crate::{
        closet::closet::{ClosetBuiltinType, ClosetExecutionType},
        error::Result,
    };

    use super::*;

    #[test]
    fn get_default_config() -> Result<()> {
        let startup_config = get_startup_configuration()?;
        assert_eq!(startup_config.name, None);
        assert_eq!(startup_config.closet, None);

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

        let closet = startup_config.closet.expect("Closet should not be None");
        assert_eq!(closet.name, "second_local_db");
        assert_eq!(closet.execution_type, ClosetExecutionType::Builtin);
        assert_eq!(closet.builtin_type, Some(ClosetBuiltinType::LocalSurrealDb));
        unsafe {
            env::remove_var(STARTUP_CONFIGURATION_ENV_VARIABLE_NAME);
        }
        return Ok(());
    }

    #[test]
    fn parse_config() -> Result<()> {
        let input = include_str!("../../tests/assets/test_config.json");
        let startup_config: StartupConfiguration = serde_json::from_str(input)?;
        assert_eq!(startup_config.name, Some("local_surreal_db_test".into()));

        let closet = startup_config.closet.expect("Closet should not be None");
        assert_eq!(closet.name, "second_local_db");
        assert_eq!(closet.execution_type, ClosetExecutionType::Builtin);
        assert_eq!(closet.builtin_type, Some(ClosetBuiltinType::LocalSurrealDb));
        return Ok(());
    }
}
