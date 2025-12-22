use crate::sessions::SessionStartOptions;
use crate::{ci, projects, sessions};
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
    /// Run CI checks
    Ci,
    /// Run CD (Continuous Deployment)
    Cd,
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

#[derive(Error, Debug)]
pub enum ExecuteError {
    #[error("cast toml not found")]
    CastTomlNotFound,
    #[error("with-changes error: {0}")]
    WithChangesError(String),
    #[error("ci error: {0}")]
    CiError(#[from] ci::CiError),
}

pub fn execute(args: Args, entry_directory: &Path) -> Result<String, ExecuteError> {
    // Handle commands that don't require Cast.toml
    if let Commands::Project(ProjectCommands::WithChanges(cmd)) = &args.cmd {
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

    // Other commands require Cast.toml
    if let Some(working_directory) = find_cast_toml(entry_directory) {
        match &args.cmd {
            Commands::Session(session_command) => match session_command {
                SessionCommands::Start(start_session_command) => {
                    let _ = sessions::start(
                        working_directory,
                        Some(SessionStartOptions {
                            name: start_session_command.name.clone(),
                        }), // TODO: Refactor to remove this clone
                    );
                    Ok("Starting session.".into())
                }
                SessionCommands::Pause => {
                    let _ = sessions::pause(working_directory);
                    Ok("Pausing session.".into())
                }
                SessionCommands::Stop => {
                    let _ = sessions::stop(working_directory);
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
            Commands::Cd => Ok("starting CD".into()),
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
    fn it_runs_cd() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();

        let result = execute(Args { cmd: Commands::Cd }, tmp_dir.path()).unwrap();
        assert_eq!(result, "starting CD");
    }
}
