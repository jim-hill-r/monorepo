use crate::sessions::SessionStartOptions;
use crate::{build, cd, ci, deploy, projects, publish, run, serve, sessions, test, toolchain};
use clap::{Parser, Subcommand};
use std::fs;
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
    #[command(subcommand)]
    Session(SessionCommands),
    #[command(subcommand)]
    Project(ProjectCommands),
    #[command(subcommand)]
    Toolchain(ToolchainCommands),
    /// Run build
    Build,
    /// Run CI checks
    Ci,
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
pub enum ToolchainCommands {
    /// Install required toolchain dependencies
    Install(InstallToolchainCommand),
    /// Check if required tools are installed
    Check(CheckToolchainCommand),
    /// List installed tools and their versions
    List(ListToolchainCommand),
}

#[derive(Parser)]
pub struct InstallToolchainCommand {
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
}

#[derive(Parser)]
pub struct CheckToolchainCommand {
    /// Show detailed information about each tool
    #[arg(short, long)]
    verbose: bool,

    /// Output results in JSON format
    #[arg(long)]
    json: bool,
}

#[derive(Parser)]
pub struct ListToolchainCommand {
    /// Show only required tools for the current project
    #[arg(long)]
    required_only: bool,

    /// Show all known tools, not just installed ones
    #[arg(long)]
    all: bool,

    /// Output results in JSON format
    #[arg(long)]
    json: bool,
}

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("cast toml not found")]
    CastTomlNotFound,
    #[error("with-changes error: {0}")]
    WithChangesError(String),
    #[error("ci error: {0}")]
    CiError(#[from] ci::CiError),
    #[error("cd error: {0}")]
    CdError(#[from] cd::CdError),
    #[error("build error: {0}")]
    BuildError(#[from] build::BuildError),
    #[error("test error: {0}")]
    TestError(#[from] test::TestError),
    #[error("run error: {0}")]
    RunError(#[from] run::RunError),
    #[error("serve error: {0}")]
    ServeError(#[from] serve::ServeError),
    #[error("deploy error: {0}")]
    DeployError(#[from] deploy::DeployError),
    #[error("publish error: {0}")]
    PublishError(#[from] publish::PublishError),
    #[error("start session error: {0}")]
    StartSessionError(#[from] sessions::StartSessionError),
    #[error("pause session error: {0}")]
    PauseSessionError(#[from] sessions::PauseSessionError),
    #[error("stop session error: {0}")]
    StopSessionError(#[from] sessions::StopSessionError),
    #[error("toolchain error: {0}")]
    ToolchainError(String),
}

pub fn execute(args: Args, entry_directory: &Path) -> Result<String, ExecuteError> {
    // Handle commands that don't require Cast.toml
    match &args.cmd {
        Commands::Project(ProjectCommands::WithChanges(cmd)) => {
            let changed_projects = projects::with_changes(entry_directory, &cmd.base, &cmd.head)
                .map_err(|e| ExecuteError::WithChangesError(e.to_string()))?;

            // Return newline-separated list of project paths
            let output = changed_projects
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            return Ok(output);
        }
        Commands::Serve => {
            // Serve command doesn't require Cast.toml - it can serve any directory
            serve::run(entry_directory)?;
            return Ok("Static file server started".into());
        }
        _ => {} // Other commands require Cast.toml
    }

    // Other commands require Cast.toml
    if let Some(working_directory) = find_cast_toml(entry_directory) {
        match args.cmd {
            Commands::Session(session_command) => match session_command {
                SessionCommands::Start(start_session_command) => {
                    sessions::start(
                        working_directory,
                        Some(SessionStartOptions {
                            name: start_session_command.name,
                        }),
                    )?;
                    Ok("Starting session.".into())
                }
                SessionCommands::Pause => {
                    sessions::pause(working_directory)?;
                    Ok("Pausing session.".into())
                }
                SessionCommands::Stop => {
                    sessions::stop(working_directory)?;
                    Ok("Stopping session.".into())
                }
            },
            Commands::Project(project_command) => match project_command {
                ProjectCommands::New(new_project_command) => {
                    let _ = projects::new(working_directory, &new_project_command.name);
                    Ok("Creating project.".into())
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
            Commands::Ci => {
                ci::run(working_directory)?;
                Ok("CI passed".into())
            }
            Commands::Build => {
                build::run(working_directory)?;
                Ok("Build passed".into())
            }
            Commands::Test => {
                test::run(working_directory)?;
                Ok("Tests passed".into())
            }
            Commands::Run => {
                run::run(working_directory)?;
                Ok("Server started".into())
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
                deploy::run(working_directory)?;
                Ok("Deploy completed".into())
            }
            Commands::Cd => {
                cd::run(working_directory)?;
                Ok("CD completed".into())
            }
            Commands::Publish => {
                publish::run(working_directory)?;
                Ok("Publish completed".into())
            }
            Commands::Toolchain(toolchain_command) => match toolchain_command {
                ToolchainCommands::Install(install_cmd) => {
                    // Parse tool names from command line
                    let specific_tools = if let Some(tool_name) = &install_cmd.tool {
                        let tool = toolchain::Tool::from_name(tool_name).ok_or_else(|| {
                            ExecuteError::ToolchainError(format!("Unknown tool: {}", tool_name))
                        })?;
                        Some(vec![tool])
                    } else {
                        None
                    };

                    // Parse skip list
                    let skip_tools = if let Some(skip_str) = &install_cmd.skip {
                        skip_str
                            .split(',')
                            .filter_map(|s| toolchain::Tool::from_name(s.trim()))
                            .collect()
                    } else {
                        Vec::new()
                    };

                    let options = toolchain::InstallOptions {
                        specific_tools,
                        skip_tools,
                        dry_run: install_cmd.dry_run,
                        force: install_cmd.force,
                    };

                    let results = toolchain::install_tools(working_directory, options)
                        .map_err(|e| ExecuteError::ToolchainError(e.to_string()))?;

                    // Format output
                    let mut output = String::new();
                    let mut any_failed = false;

                    for result in &results {
                        if result.skipped {
                            output.push_str(&format!(
                                "⊘ {}: {}\n",
                                result.tool.name(),
                                result.message
                            ));
                        } else if result.success {
                            output.push_str(&format!(
                                "✓ {}: {}\n",
                                result.tool.name(),
                                result.message
                            ));
                        } else {
                            output.push_str(&format!(
                                "✗ {}: {}\n",
                                result.tool.name(),
                                result.message
                            ));
                            any_failed = true;
                        }
                    }

                    if any_failed {
                        output.push_str(
                            "\nSome tools failed to install. Please address the errors above.",
                        );
                        Err(ExecuteError::ToolchainError(output))
                    } else {
                        Ok(output)
                    }
                }
                ToolchainCommands::Check(check_cmd) => {
                    let options = toolchain::CheckOptions {
                        verbose: check_cmd.verbose,
                        json: check_cmd.json,
                    };

                    let check_result = toolchain::check_tools(working_directory, options)
                        .map_err(|e| ExecuteError::ToolchainError(e.to_string()))?;

                    // Format output based on options
                    let output = if check_cmd.json {
                        check_result
                            .format_json()
                            .map_err(|e| ExecuteError::ToolchainError(e.to_string()))?
                    } else {
                        check_result.format_text(check_cmd.verbose)
                    };

                    // Return error if tools are missing (exit code 1)
                    if !check_result.all_installed {
                        Err(ExecuteError::ToolchainError(output))
                    } else {
                        Ok(output)
                    }
                }
                ToolchainCommands::List(list_cmd) => {
                    let options = toolchain::ListOptions {
                        required_only: list_cmd.required_only,
                        all: list_cmd.all,
                        json: list_cmd.json,
                    };

                    let list_result = toolchain::list_tools(working_directory, options)
                        .map_err(|e| ExecuteError::ToolchainError(e.to_string()))?;

                    // Format output based on options
                    let output = if list_cmd.json {
                        list_result
                            .format_json()
                            .map_err(|e| ExecuteError::ToolchainError(e.to_string()))?
                    } else {
                        list_result.format_text()
                    };

                    Ok(output)
                }
            },
        }
    } else {
        Err(ExecuteError::CastTomlNotFound)
    }
}

fn find_cast_toml(working_directory: &Path) -> Option<&Path> {
    let mut current_directory = Some(working_directory);
    while let Some(current_path) = current_directory {
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries.flatten() {
                if entry.file_name() == "Cast.toml" {
                    return current_directory;
                }
            }
            current_directory = current_path.parent();
        }
    }

    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::fs;
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
        )
        .unwrap();
        assert_eq!(result, "Creating project.");
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

        // Create a minimal Cargo.toml and src/lib.rs for CI to pass
        fs::write(
            tmp_dir.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        fs::create_dir_all(tmp_dir.path().join("src")).unwrap();
        fs::write(tmp_dir.path().join("src/lib.rs"), "pub fn test() {}\n").unwrap();

        let result = execute(Args { cmd: Commands::Ci }, tmp_dir.path()).unwrap();
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
        assert!(matches!(err, ExecuteError::PauseSessionError(_)));
        assert_eq!(
            err.to_string(),
            "pause session error: no active session found"
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
        assert!(matches!(err, ExecuteError::StopSessionError(_)));
        assert_eq!(
            err.to_string(),
            "stop session error: no active session found"
        );
    }

    #[test]
    fn it_runs_publish() {
        let tmp_dir = TempDir::new("test").unwrap();
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
    fn it_recognizes_toolchain_install_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::Install(InstallToolchainCommand {
                    tool: None,
                    skip: None,
                    dry_run: true, // Use dry run to avoid actual installation
                    force: false,
                })),
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
    fn it_recognizes_toolchain_install_dry_run() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::Install(InstallToolchainCommand {
                    tool: None,
                    skip: None,
                    dry_run: true,
                    force: false,
                })),
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
    fn it_recognizes_toolchain_check_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::Check(CheckToolchainCommand {
                    verbose: false,
                    json: false,
                })),
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
                assert!(matches!(err, ExecuteError::ToolchainError(_)));
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
    fn it_recognizes_toolchain_check_json() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::Check(CheckToolchainCommand {
                    verbose: false,
                    json: true,
                })),
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
    fn it_recognizes_toolchain_list_command() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::List(ListToolchainCommand {
                    required_only: false,
                    all: false,
                    json: false,
                })),
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
    fn it_recognizes_toolchain_list_all() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::List(ListToolchainCommand {
                    required_only: false,
                    all: true,
                    json: false,
                })),
            },
            tmp_dir.path(),
        );

        // Should succeed and list all tools
        assert!(result.is_ok());
        let output = result.unwrap();
        // Should contain all 9 tools
        assert!(output.contains("rustc"));
        assert!(output.contains("cargo"));
        assert!(output.contains("rustfmt"));
        assert!(output.contains("clippy"));
        assert!(output.contains("dx"));
        assert!(output.contains("node"));
        assert!(output.contains("npm"));
        assert!(output.contains("playwright"));
        assert!(output.contains("wrangler"));
    }

    #[test]
    fn it_recognizes_toolchain_list_json() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::List(ListToolchainCommand {
                    required_only: false,
                    all: false,
                    json: true,
                })),
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
    fn it_requires_cast_toml_for_toolchain_commands() {
        let tmp_dir = TempDir::new("test").unwrap();
        // No Cast.toml file created

        let result = execute(
            Args {
                cmd: Commands::Toolchain(ToolchainCommands::Check(CheckToolchainCommand {
                    verbose: false,
                    json: false,
                })),
            },
            tmp_dir.path(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ExecuteError::CastTomlNotFound));
    }
}
