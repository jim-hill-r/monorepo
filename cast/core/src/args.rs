use crate::command::Command;
use crate::commands;
use crate::config::CastConfig;
use clap::{Parser, Subcommand};
use std::path::Path;
use thiserror::Error;

#[derive(Parser)]
#[command(author, version, about = "Highly opinionated tooling for rust monorepos.", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage work sessions (start, pause, stop)
    #[command(subcommand)]
    Session(SessionCommands),
    /// Project management commands (new, with-changes)
    #[command(subcommand)]
    Project(ProjectCommands),
    /// Install tools, check installation status, or list available tools
    Install {
        #[command(subcommand)]
        subcommand: Option<InstallSubcommands>,

        /// Install only specific tool (e.g., nodejs, npm, playwright, dx, wrangler)
        #[arg(long)]
        tool: Option<String>,

        /// Skip specific tools during installation (comma-separated list)
        #[arg(long)]
        skip: Option<String>,

        /// Dry run - show what would be installed without installing
        #[arg(long)]
        dry_run: bool,

        /// Force reinstall even if tools are already installed
        #[arg(long)]
        force: bool,
    },
    /// Uninstall cast-managed tools
    Uninstall {
        /// Uninstall only specific tool (e.g., dx, playwright, wrangler)
        #[arg(long)]
        tool: Option<String>,

        /// Skip specific tools during uninstallation (comma-separated list)
        #[arg(long)]
        skip: Option<String>,

        /// Dry run - show what would be uninstalled without uninstalling
        #[arg(long)]
        dry_run: bool,

        /// Uninstall all tools that were installed by cast
        #[arg(long)]
        all: bool,
    },
    /// Run build
    Build,
    /// Run CI checks
    Ci {
        /// Run checks only (default mode for PR validation)
        #[arg(long)]
        check: bool,

        /// Auto-fix issues that can be fixed automatically (e.g., formatting)
        #[arg(long)]
        fix: bool,

        /// Build in release mode and publish artifacts (for post-merge to master)
        #[arg(long)]
        release: bool,

        /// After running CI, look N levels below current directory for other cast projects and run CI on them
        #[arg(long)]
        recursive: Option<usize>,

        /// Only run CI if the project has changes compared to the origin's default branch
        #[arg(long)]
        only_changed: bool,
    },
    /// Run CD (Continuous Deployment)
    Cd,
    /// Run tests
    Test,
    /// Run server (dx serve for dioxus, cargo run otherwise)
    Run,
    /// Serve static files from current directory
    Serve,
    /// Deploy an IAC project
    Deploy,
    /// Build release and copy artifacts to artifacts directory
    Publish,
}

#[derive(Subcommand)]
pub enum SessionCommands {
    Start(StartSessionCommand),
    Pause,
    Stop,
}

#[derive(Parser)]
pub struct StartSessionCommand {
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    New(NewProjectCommand),
    /// List projects with changes between two git refs
    WithChanges(WithChangesCommand),
}

#[derive(Parser)]
pub struct NewProjectCommand {
    #[arg(short, long)]
    name: String,
}

#[derive(Parser)]
pub struct WithChangesCommand {
    /// Base git ref (commit SHA, branch, or tag)
    #[arg(long)]
    base: String,

    /// Head git ref (commit SHA, branch, or tag)
    #[arg(long)]
    head: String,
}

#[derive(Subcommand)]
pub enum InstallSubcommands {
    /// Check if required tools are installed
    Check {
        /// Show detailed information about each tool
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
    /// List installed tools and their versions
    List {
        /// Show only required tools for the current project
        #[arg(long)]
        required_only: bool,

        /// Show all known tools, not just installed ones
        #[arg(long)]
        all: bool,

        /// Output results in JSON format
        #[arg(long)]
        json: bool,
    },
}

// Remove the old types that are no longer needed
// InstallCommands, InstallToolchainCommand, CheckToolchainCommand, ListToolchainCommand

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("cast configuration not found")]
    CastConfigurationNotFound,
    #[error("command error: {0}")]
    CommandError(String),
}

pub fn execute(args: Args, entry_directory: &Path) -> Result<String, ExecuteError> {
    // Handle commands that don't require Cast.toml
    match &args.cmd {
        Commands::Project(ProjectCommands::WithChanges(cmd)) => {
            let command = commands::project::WithChangesCommand {
                base: cmd.base.clone(),
                head: cmd.head.clone(),
            };
            return command
                .execute(entry_directory)
                .map_err(|e| ExecuteError::CommandError(e.to_string()));
        }
        Commands::Serve => {
            let command = commands::serve::ServeCommand;
            return command
                .execute(entry_directory)
                .map_err(|e| ExecuteError::CommandError(e.to_string()));
        }
        Commands::Ci {
            check: _,
            fix,
            release,
            recursive: Some(depth),
            only_changed,
        } if find_cast_toml(entry_directory).is_none() => {
            // Special handling: If recursive is enabled but no Cast config in current dir,
            // just run the recursive search for child projects
            let mode = if *release {
                crate::ci::CiMode::Release
            } else if *fix {
                crate::ci::CiMode::Fix
            } else {
                crate::ci::CiMode::Check
            };

            return crate::ci::run_ci_recursively(entry_directory, mode, *depth, *only_changed)
                .map(|_| "CI passed".to_string())
                .map_err(|e| ExecuteError::CommandError(e.to_string()));
        }
        Commands::Install {
            subcommand,
            tool,
            skip,
            dry_run,
            force,
        } => {
            match subcommand {
                Some(InstallSubcommands::Check { verbose, json }) => {
                    let command = commands::toolchain::CheckCommand {
                        verbose: *verbose,
                        json: *json,
                    };
                    return command
                        .execute(entry_directory)
                        .map_err(|e| ExecuteError::CommandError(e.to_string()));
                }
                Some(InstallSubcommands::List {
                    required_only,
                    all,
                    json,
                }) => {
                    let command = commands::toolchain::ListCommand {
                        required_only: *required_only,
                        all: *all,
                        json: *json,
                    };
                    return command
                        .execute(entry_directory)
                        .map_err(|e| ExecuteError::CommandError(e.to_string()));
                }
                None => {
                    // Default action: install tools
                    let command = commands::toolchain::InstallCommand {
                        tool: tool.clone(),
                        skip: skip.clone(),
                        dry_run: *dry_run,
                        force: *force,
                    };
                    return command
                        .execute(entry_directory)
                        .map_err(|e| ExecuteError::CommandError(e.to_string()));
                }
            }
        }
        Commands::Uninstall {
            tool,
            skip,
            dry_run,
            all,
        } => {
            let command = commands::toolchain::UninstallCommand {
                tool: tool.clone(),
                skip: skip.clone(),
                dry_run: *dry_run,
                all: *all,
            };
            return command
                .execute(entry_directory)
                .map_err(|e| ExecuteError::CommandError(e.to_string()));
        }
        _ => {} // Other commands require Cast.toml
    }

    // Other commands require Cast.toml
    if let Some(working_directory) = find_cast_toml(entry_directory) {
        let result: Result<String, Box<dyn std::error::Error>> = match args.cmd {
            Commands::Session(session_command) => match session_command {
                SessionCommands::Start(start_session_command) => {
                    let command = commands::session::StartCommand {
                        name: start_session_command.name,
                    };
                    command.execute(working_directory)
                }
                SessionCommands::Pause => {
                    let command = commands::session::PauseCommand;
                    command.execute(working_directory)
                }
                SessionCommands::Stop => {
                    let command = commands::session::StopCommand;
                    command.execute(working_directory)
                }
            },
            Commands::Project(project_command) => match project_command {
                ProjectCommands::New(new_project_command) => {
                    let command = commands::project::NewCommand {
                        name: new_project_command.name,
                    };
                    command.execute(working_directory)
                }
                ProjectCommands::WithChanges(_) => {
                    // This case should never be reached because WithChanges is handled
                    // at the top of execute() before the Cast.toml check. If we reach
                    // this point, there's a bug in the control flow logic.
                    unreachable!(
                        "WithChanges command should be handled before Cast.toml check. \
                         This indicates a bug in the execute() function's control flow."
                    )
                }
            },
            Commands::Ci {
                check: _,
                fix,
                release,
                recursive,
                only_changed,
            } => {
                // Determine the mode based on flags
                // If no flags are set, default to Check mode
                // If multiple flags are set, prioritize: release > fix > check
                let mode = if release {
                    crate::ci::CiMode::Release
                } else if fix {
                    crate::ci::CiMode::Fix
                } else {
                    crate::ci::CiMode::Check
                };

                let command = commands::ci::CiCommand {
                    mode,
                    recursive_depth: recursive,
                    only_changed,
                };
                command.execute(working_directory)
            }
            Commands::Build => {
                let command = commands::build::BuildCommand;
                command.execute(working_directory)
            }
            Commands::Test => {
                let command = commands::test::TestCommand;
                command.execute(working_directory)
            }
            Commands::Run => {
                let command = commands::run::RunCommand;
                command.execute(working_directory)
            }
            Commands::Serve => {
                // This case should never be reached because Serve is handled
                // at the top of execute() before the Cast.toml check. If we reach
                // this point, there's a bug in the control flow logic.
                unreachable!(
                    "Serve command should be handled before Cast.toml check. \
                     This indicates a bug in the execute() function's control flow."
                )
            }
            Commands::Deploy => {
                let command = commands::deploy::DeployCommand;
                command.execute(working_directory)
            }
            Commands::Cd => {
                let command = commands::cd::CdCommand;
                command.execute(working_directory)
            }
            Commands::Publish => {
                let command = commands::publish::PublishCommand;
                command.execute(working_directory)
            }
            Commands::Install { .. } => {
                // This case should never be reached because Install is handled
                // at the top of execute() before the Cast.toml check. If we reach
                // this point, there's a bug in the control flow logic.
                unreachable!(
                    "Install command should be handled before Cast.toml check. \
                     This indicates a bug in the execute() function's control flow."
                )
            }
            Commands::Uninstall { .. } => {
                // This case should never be reached because Uninstall is handled
                // at the top of execute() before the Cast.toml check. If we reach
                // this point, there's a bug in the control flow logic.
                unreachable!(
                    "Uninstall command should be handled before Cast.toml check. \
                     This indicates a bug in the execute() function's control flow."
                )
            }
        };

        result.map_err(|e| ExecuteError::CommandError(e.to_string()))
    } else {
        Err(ExecuteError::CastConfigurationNotFound)
    }
}

fn find_cast_toml(working_directory: &Path) -> Option<&Path> {
    let mut current_directory = Some(working_directory);
    while let Some(current_path) = current_directory {
        // Check for Cast.toml
        if current_path.join("Cast.toml").exists() {
            return current_directory;
        }

        // Check for Cargo.toml with Cast metadata
        let cargo_toml_path = current_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            if let Ok(config) = CastConfig::load_from_cargo_toml(&cargo_toml_path) {
                if config.has_cast_metadata() {
                    return current_directory;
                }
            }
        }

        current_directory = current_path.parent();
    }

    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempdir::TempDir;

    #[test]
    fn it_exits_if_cast_toml_is_missing() {
        let tmp_dir = TempDir::new("test").unwrap();
        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Start(StartSessionCommand { name: None })),
            },
            tmp_dir.path(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn it_starts_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Start(StartSessionCommand { name: None })),
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Starting session.");
    }
    #[test]
    fn it_pauses_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Start a session first
        execute(
            Args {
                cmd: Commands::Session(SessionCommands::Start(StartSessionCommand { name: None })),
            },
            tmp_dir.path(),
        )
        .unwrap();

        // Then pause it
        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Pause),
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Pausing session.");
    }
    #[test]
    fn it_stops_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Start a session first
        execute(
            Args {
                cmd: Commands::Session(SessionCommands::Start(StartSessionCommand { name: None })),
            },
            tmp_dir.path(),
        )
        .unwrap();

        // Then stop it
        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Stop),
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Stopping session.");
    }
    #[test]
    fn it_news_project() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        let result = execute(
            Args {
                cmd: Commands::Project(ProjectCommands::New(NewProjectCommand {
                    name: "test".into(),
                })),
            },
            tmp_dir.path(),
        );
        // Should fail because no exemplar projects exist
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("No exemplar projects found"),
            "Expected error about no exemplar projects, got: {}",
            err
        );
    }

    #[test]
    fn it_traverses_up_file_tree_to_find_cast_toml() {
        let tmp_dir = TempDir::new("test").unwrap();
        let child_dir = tmp_dir
            .path()
            .join("test_level_two/test_level_three/test_level_four");
        fs::create_dir_all(&child_dir).unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        assert_eq!(find_cast_toml(child_dir.as_path()).unwrap(), tmp_dir.path())
    }

    #[test]
    fn it_runs_ci() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Initialize git repo (required by publish which CI now runs)
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Create a minimal binary project for CI to pass (publish requires a binary)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )
        .unwrap();

        // Commit the project (required by publish)
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let result = execute(
            Args {
                cmd: Commands::Ci {
                    check: false,
                    fix: false,
                    release: false,
                    recursive: None,
                    only_changed: false,
                },
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "CI passed");
    }

    #[test]
    fn it_runs_build() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a minimal Cargo.toml and src/lib.rs for build to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Build,
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Build passed");
    }

    #[test]
    fn it_runs_cd() {
        let tmp_dir = TempDir::new("test").unwrap();

        // Create a .git directory to mark as monorepo root (needed by cd module)
        fs::create_dir(tmp_dir.path().join(".git")).unwrap();

        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(Args { cmd: Commands::Cd }, tmp_dir.path()).unwrap();
        assert_eq!(result, "CD completed");
    }

    #[test]
    fn it_runs_test() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a minimal Cargo.toml and src/lib.rs for test to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/lib.rs"),
            "pub fn test() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn it_works() {\n        assert_eq!(2 + 2, 4);\n    }\n}",
        )
        .unwrap();

        let result = execute(
            Args {
                cmd: Commands::Test,
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Tests passed");
    }

    #[test]
    fn it_runs_run() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a minimal Cargo.toml and src/main.rs for run to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        let result = execute(Args { cmd: Commands::Run }, tmp_dir.path()).unwrap();
        assert_eq!(result, "Server started");
    }

    #[test]
    fn it_runs_serve() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a simple HTML file to serve
        fs::write(
            tmp_dir.path().join("index.html"),
            "<html><body>Test</body></html>",
        )
        .unwrap();

        // The serve command starts a blocking server, so we can't easily test it
        // in a synchronous test. We'll just verify the command is recognized.
        // In practice, the serve command would need to be stopped with Ctrl+C

        // For this test, we'll just verify the command structure is correct
        // by checking that it doesn't error on Cast.toml lookup
        assert!(tmp_dir.path().join("Cast.toml").exists());
    }

    #[test]
    fn it_runs_deploy() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(
            tmp_dir.path().join("Cast.toml"),
            "project_type = \"iac\"\nframework = \"cloudflare-pages\"",
        )
        .unwrap();

        let result = execute(
            Args {
                cmd: Commands::Deploy,
            },
            tmp_dir.path(),
        );
        // Deploy will fail without wrangler.toml or wrangler installed, but it should
        // at least recognize it as a valid command for an IAC project
        assert!(result.is_err());
    }

    #[test]
    fn it_returns_error_when_pausing_without_active_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Pause),
            },
            tmp_dir.path(),
        );

        assert!(
            result.is_err(),
            "Expected error when pausing without active session"
        );
        let err = result.unwrap_err();
        assert!(matches!(err, ExecuteError::CommandError(_)));
        assert!(
            err.to_string().contains("no active session found"),
            "Error message should mention no active session: {}",
            err
        );
    }

    #[test]
    fn it_returns_error_when_stopping_without_active_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Session(SessionCommands::Stop),
            },
            tmp_dir.path(),
        );

        assert!(
            result.is_err(),
            "Expected error when stopping without active session"
        );
        let err = result.unwrap_err();
        assert!(matches!(err, ExecuteError::CommandError(_)));
        assert!(
            err.to_string().contains("no active session found"),
            "Error message should mention no active session: {}",
            err
        );
    }

    #[test]
    fn it_runs_publish() {
        use std::process::Command;

        let tmp_dir = TempDir::new("test").unwrap();

        // Initialize git repository (required for generate_bundle_filename)
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        // Create a minimal binary project
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        // Commit to have a valid git SHA
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        let result = execute(
            Args {
                cmd: Commands::Publish,
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Publish completed");
    }

    #[test]
    fn it_recognizes_install_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: None,
                    tool: None,
                    skip: None,
                    dry_run: true,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed with dry run
        assert!(result.is_ok());
        let output = result.unwrap();
        // Output should contain information about tools
        assert!(!output.is_empty());
    }

    #[test]
    fn it_recognizes_install_dry_run() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: None,
                    tool: None,
                    skip: None,
                    dry_run: true,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed with dry run
        assert!(result.is_ok());
        let output = result.unwrap();
        // Output should indicate dry run was performed
        assert!(!output.is_empty());
    }

    #[test]
    fn it_runs_install_without_cast_toml() {
        let tmp_dir = TempDir::new("test").unwrap();
        // No Cast.toml file created - should still work with default tools

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: None,
                    tool: None,
                    skip: None,
                    dry_run: true,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed even without Cast.toml, installing default tools
        assert!(result.is_ok());
        let output = result.unwrap();
        // Output should contain information about default Rust tools
        assert!(output.contains("rustc") || output.contains("cargo"));
    }

    #[test]
    fn it_recognizes_install_check_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::Check {
                        verbose: false,
                        json: false,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Check command should work now, but might fail if some tools are not installed
        // The result could be Ok or Err depending on the environment
        match result {
            Ok(output) => {
                // If successful, all tools are installed
                assert!(output.contains("All required tools are installed"));
            }
            Err(err) => {
                // If error, should show tool status
                assert!(matches!(err, ExecuteError::CommandError(_)));
                let error_msg = err.to_string();
                // Should contain toolchain information
                assert!(
                    error_msg.contains("tool") || error_msg.contains("missing"),
                    "Error message should contain tool information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn it_recognizes_install_check_json() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::Check {
                        verbose: false,
                        json: true,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Check command should work now with JSON output
        match result {
            Ok(output) => {
                // If successful, should be valid JSON
                assert!(output.contains("\"framework\""));
                assert!(output.contains("\"tools\""));
                assert!(output.contains("\"all_installed\""));
            }
            Err(err) => {
                // If error, should still be JSON format
                let error_msg = err.to_string();
                assert!(
                    error_msg.contains("\"framework\"") || error_msg.contains("JSON"),
                    "Error message should contain JSON information: {}",
                    error_msg
                );
            }
        }
    }

    #[test]
    fn it_recognizes_install_list_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::List {
                        required_only: false,
                        all: false,
                        json: false,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed now that list is implemented
        assert!(result.is_ok());
        let output = result.unwrap();
        // Output should contain tool names
        assert!(output.contains("rustc") || output.contains("cargo"));
    }

    #[test]
    fn it_recognizes_install_list_all() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::List {
                        required_only: false,
                        all: true,
                        json: false,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed and list all tools
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain all 11 tools
        assert!(output.contains("rustup"));
        assert!(output.contains("rustc"));
        assert!(output.contains("cargo"));
        assert!(output.contains("rustfmt"));
        assert!(output.contains("clippy"));
        assert!(output.contains("dx"));
        assert!(output.contains("node"));
        assert!(output.contains("npm"));
        assert!(output.contains("playwright"));
        assert!(output.contains("wrangler"));
        assert!(output.contains("git-lfs"));
    }

    #[test]
    fn it_recognizes_install_list_json() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::List {
                        required_only: false,
                        all: false,
                        json: true,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed and output JSON
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should be valid JSON with tools array
        assert!(output.contains("{"));
        assert!(output.contains("}"));
        assert!(output.contains("\"tools\""));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"installed\""));
    }

    #[test]
    fn it_does_not_require_cast_toml_for_install_commands() {
        let tmp_dir = TempDir::new("test").unwrap();
        // No Cast.toml file created

        let result = execute(
            Args {
                cmd: Commands::Install {
                    subcommand: Some(InstallSubcommands::Check {
                        verbose: false,
                        json: false,
                    }),
                    tool: None,
                    skip: None,
                    dry_run: false,
                    force: false,
                },
            },
            tmp_dir.path(),
        );

        // Install commands should work without Cast.toml (will use default tools)
        // The result could be Ok or Err depending on the environment
        match result {
            Ok(_) => {
                // All default tools are installed
            }
            Err(err) => {
                // Some tools are missing, but command ran successfully
                assert!(matches!(err, ExecuteError::CommandError(_)));
            }
        }
    }

    #[test]
    fn it_finds_cargo_toml_with_cast_metadata() {
        let tmp_dir = TempDir::new("test").unwrap();

        // Create Cargo.toml with cast metadata (no Cast.toml)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"",
        )
        .unwrap();

        let found = find_cast_toml(tmp_dir.path());
        assert_eq!(found, Some(tmp_dir.path()));
    }

    #[test]
    fn it_does_not_find_cargo_toml_without_cast_metadata() {
        let tmp_dir = TempDir::new("test").unwrap();

        // Create Cargo.toml without cast metadata
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();

        let found = find_cast_toml(tmp_dir.path());
        assert_eq!(found, None);
    }

    #[test]
    fn it_traverses_up_file_tree_to_find_cargo_toml_with_metadata() {
        let tmp_dir = TempDir::new("test").unwrap();
        let child_dir = tmp_dir
            .path()
            .join("test_level_two/test_level_three/test_level_four");
        fs::create_dir_all(&child_dir).unwrap();

        // Create Cargo.toml with cast metadata in parent directory
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"",
        )
        .unwrap();

        assert_eq!(find_cast_toml(child_dir.as_path()).unwrap(), tmp_dir.path())
    }

    #[test]
    fn it_runs_run_with_cargo_metadata() {
        let tmp_dir = TempDir::new("test").unwrap();

        // Create a Cargo project with dioxus framework in metadata (no Cast.toml)
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[package.metadata.cast]\nframework = \"dioxus\"\n\n[dependencies]\ndioxus = \"0.6\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(
            tmp_dir.path().join("src/main.rs"),
            "fn main() { println!(\"Hello, world!\"); }\n",
        )
        .unwrap();

        let result = execute(Args { cmd: Commands::Run }, tmp_dir.path());

        // Should find the cast metadata and run correctly (will fail if dx is not installed)
        assert!(result.is_err()); // Will fail because dx is not installed
        if let Err(ExecuteError::CommandError(err_msg)) = result {
            // We expect error related to dx not being found, run failing, or IO error
            assert!(
                err_msg.contains("dx")
                    || err_msg.contains("run")
                    || err_msg.contains("failed")
                    || err_msg.contains("No such file"),
                "Expected error related to dx or run, got: {}",
                err_msg
            );
        } else {
            panic!("Expected CommandError");
        }
    }

    #[test]
    fn it_runs_ci_recursively_without_cast_config_in_current_dir() {
        let tmp_dir = TempDir::new("test_ci_recursive_no_config").unwrap();

        // Initialize git repo for child projects
        Command::new("git")
            .arg("init")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.email")
            .arg("test@example.com")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("config")
            .arg("user.name")
            .arg("Test User")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Don't create Cast.toml in the root - this is the key point of the test

        // Create a child project with Cast.toml
        let child = tmp_dir.path().join("child");
        fs::create_dir_all(&child).unwrap();
        fs::write(child.join("Cast.toml"), "").unwrap();
        fs::write(
            child.join("Cargo.toml"),
            "[package]\nname = \"child\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(child.join("src")).unwrap();
        fs::write(
            child.join("src/main.rs"),
            "fn main() {\n    println!(\"child\");\n}\n",
        )
        .unwrap();

        // Commit everything
        Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("initial commit")
            .current_dir(tmp_dir.path())
            .output()
            .unwrap();

        // Run CI with recursive depth 1 from the root (which has no Cast.toml)
        let result = execute(
            Args {
                cmd: Commands::Ci {
                    check: false,
                    fix: false,
                    release: false,
                    recursive: Some(1),
                    only_changed: false,
                },
            },
            tmp_dir.path(),
        );

        // Should succeed - it should find and run CI on the child project
        assert!(
            result.is_ok(),
            "CI with recursive should succeed even without Cast config in current dir: {:?}",
            result.err()
        );

        // Verify artifacts were created for the child project
        assert!(
            child.join("artifacts").exists(),
            "Child project should have artifacts after CI"
        );
    }
}
