use crate::args::{Commands, InstallSubcommands, ProjectCommands, SessionCommands};
use crate::command::Command;
use crate::commands;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FactoryError {
    #[error("Command requires Cast configuration")]
    RequiresCastConfig,
    #[error("Invalid command configuration: {0}")]
    InvalidConfiguration(String),
}

/// Creates a Command instance from Args
///
/// This factory function maps Args variants to their corresponding Command implementations,
/// following the Command/Executor pattern. It separates the concerns of command parsing
/// (handled by Args/clap) from command instantiation and execution.
///
/// # Arguments
///
/// * `command` - The parsed command from Args
/// * `requires_cast_config` - Whether Cast.toml is available in the working directory
///
/// # Returns
///
/// * `Ok(Box<dyn Command>)` - The instantiated command ready for execution
/// * `Err(FactoryError)` - If the command cannot be created (e.g., requires Cast.toml but it's not available)
///
/// # Examples
///
/// ```
/// use cast_core::command_factory::create_command;
/// use cast_core::args::Commands;
///
/// let command = Commands::Build;
/// let cmd = create_command(&command, true)?;
/// // cmd can now be executed: cmd.execute(working_directory)?
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn create_command(
    command: &Commands,
    requires_cast_config: bool,
) -> Result<Box<dyn Command>, FactoryError> {
    match command {
        // Commands that don't require Cast.toml
        Commands::Project(ProjectCommands::WithChanges(cmd)) => {
            Ok(Box::new(commands::project::WithChangesCommand {
                base: cmd.base.clone(),
                head: cmd.head.clone(),
            }))
        }
        Commands::Serve => Ok(Box::new(commands::serve::ServeCommand)),
        Commands::Install {
            subcommand,
            tool,
            skip,
            dry_run,
            force,
        } => match subcommand {
            Some(InstallSubcommands::Check { verbose, json }) => {
                Ok(Box::new(commands::install::CheckCommand {
                    verbose: *verbose,
                    json: *json,
                }))
            }
            Some(InstallSubcommands::List {
                required_only,
                all,
                json,
            }) => Ok(Box::new(commands::install::ListCommand {
                required_only: *required_only,
                all: *all,
                json: *json,
            })),
            None => Ok(Box::new(commands::install::InstallCommand {
                tool: tool.clone(),
                skip: skip.clone(),
                dry_run: *dry_run,
                force: *force,
            })),
        },
        Commands::Uninstall {
            tool,
            skip,
            dry_run,
            all,
        } => Ok(Box::new(commands::install::UninstallCommand {
            tool: tool.clone(),
            skip: skip.clone(),
            dry_run: *dry_run,
            all: *all,
        })),
        // Commands that require Cast.toml
        _ if !requires_cast_config => Err(FactoryError::RequiresCastConfig),
        Commands::Session(session_command) => match session_command {
            SessionCommands::Start(start_cmd) => Ok(Box::new(commands::session::StartCommand {
                name: start_cmd.name.clone(),
            })),
            SessionCommands::Pause => Ok(Box::new(commands::session::PauseCommand)),
            SessionCommands::Stop => Ok(Box::new(commands::session::StopCommand)),
        },
        Commands::Project(project_command) => match project_command {
            ProjectCommands::New(new_cmd) => Ok(Box::new(commands::project::NewCommand {
                name: new_cmd.name.clone(),
            })),
            ProjectCommands::WithChanges(_) => {
                // This should never happen as WithChanges is handled above
                Err(FactoryError::InvalidConfiguration(
                    "WithChanges should be handled before Cast.toml check".to_string(),
                ))
            }
        },
        Commands::Ci {
            check: _,
            fix,
            release,
            recursive,
            only_changed,
        } => {
            let mode = if *release {
                crate::ci::CiMode::Release
            } else if *fix {
                crate::ci::CiMode::Fix
            } else {
                crate::ci::CiMode::Check
            };

            Ok(Box::new(commands::ci::CiCommand {
                mode,
                recursive_depth: *recursive,
                only_changed: *only_changed,
            }))
        }
        Commands::Build => Ok(Box::new(commands::build::BuildCommand)),
        Commands::Test { coverage } => Ok(Box::new(commands::test::TestCommand {
            coverage: *coverage,
        })),
        Commands::Run => Ok(Box::new(commands::run::RunCommand)),
        Commands::Deploy => Ok(Box::new(commands::deploy::DeployCommand)),
        Commands::Cd => Ok(Box::new(commands::cd::CdCommand)),
        Commands::Publish => Ok(Box::new(commands::publish::PublishCommand)),
    }
}

/// Helper function to determine if a command requires Cast.toml
///
/// Some commands (like Install, Serve, and project WithChanges) can run
/// without Cast.toml, while others require it to determine project configuration.
pub fn requires_cast_config(command: &Commands) -> bool {
    !matches!(
        command,
        Commands::Install { .. }
            | Commands::Uninstall { .. }
            | Commands::Serve
            | Commands::Project(ProjectCommands::WithChanges(_))
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::args::{
        NewProjectCommand, StartSessionCommand, WithChangesCommand as ArgsWithChangesCommand,
    };

    #[test]
    fn test_create_build_command() {
        let cmd = Commands::Build;
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_build_command_without_cast_config() {
        let cmd = Commands::Build;
        let result = create_command(&cmd, false);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, FactoryError::RequiresCastConfig));
        }
    }

    #[test]
    fn test_create_test_command() {
        let cmd = Commands::Test { coverage: false };
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_ci_command_check_mode() {
        let cmd = Commands::Ci {
            check: true,
            fix: false,
            release: false,
            recursive: None,
            only_changed: false,
        };
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_ci_command_fix_mode() {
        let cmd = Commands::Ci {
            check: false,
            fix: true,
            release: false,
            recursive: None,
            only_changed: false,
        };
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_ci_command_release_mode() {
        let cmd = Commands::Ci {
            check: false,
            fix: false,
            release: true,
            recursive: Some(2),
            only_changed: true,
        };
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_session_start_command() {
        let cmd = Commands::Session(SessionCommands::Start(StartSessionCommand {
            name: Some("test-session".to_string()),
        }));
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_session_pause_command() {
        let cmd = Commands::Session(SessionCommands::Pause);
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_session_stop_command() {
        let cmd = Commands::Session(SessionCommands::Stop);
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_new_command() {
        let cmd = Commands::Project(ProjectCommands::New(NewProjectCommand {
            name: "test-project".to_string(),
        }));
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_project_with_changes_command() {
        let cmd = Commands::Project(ProjectCommands::WithChanges(ArgsWithChangesCommand {
            base: "main".to_string(),
            head: "feature".to_string(),
        }));
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_install_command() {
        let cmd = Commands::Install {
            subcommand: None,
            tool: Some("dx".to_string()),
            skip: None,
            dry_run: false,
            force: false,
        };
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_install_check_command() {
        let cmd = Commands::Install {
            subcommand: Some(InstallSubcommands::Check {
                verbose: true,
                json: false,
            }),
            tool: None,
            skip: None,
            dry_run: false,
            force: false,
        };
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_install_list_command() {
        let cmd = Commands::Install {
            subcommand: Some(InstallSubcommands::List {
                required_only: false,
                all: true,
                json: false,
            }),
            tool: None,
            skip: None,
            dry_run: false,
            force: false,
        };
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_uninstall_command() {
        let cmd = Commands::Uninstall {
            tool: Some("dx".to_string()),
            skip: None,
            dry_run: true,
            all: false,
        };
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_serve_command() {
        let cmd = Commands::Serve;
        let result = create_command(&cmd, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_deploy_command() {
        let cmd = Commands::Deploy;
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_cd_command() {
        let cmd = Commands::Cd;
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_publish_command() {
        let cmd = Commands::Publish;
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_run_command() {
        let cmd = Commands::Run;
        let result = create_command(&cmd, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_requires_cast_config_for_build() {
        let cmd = Commands::Build;
        assert!(requires_cast_config(&cmd));
    }

    #[test]
    fn test_does_not_require_cast_config_for_install() {
        let cmd = Commands::Install {
            subcommand: None,
            tool: None,
            skip: None,
            dry_run: false,
            force: false,
        };
        assert!(!requires_cast_config(&cmd));
    }

    #[test]
    fn test_does_not_require_cast_config_for_serve() {
        let cmd = Commands::Serve;
        assert!(!requires_cast_config(&cmd));
    }

    #[test]
    fn test_does_not_require_cast_config_for_with_changes() {
        let cmd = Commands::Project(ProjectCommands::WithChanges(ArgsWithChangesCommand {
            base: "main".to_string(),
            head: "feature".to_string(),
        }));
        assert!(!requires_cast_config(&cmd));
    }

    #[test]
    fn test_requires_cast_config_for_session() {
        let cmd = Commands::Session(SessionCommands::Start(StartSessionCommand { name: None }));
        assert!(requires_cast_config(&cmd));
    }
}
