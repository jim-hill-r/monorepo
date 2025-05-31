use crate::{projects, sessions};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Error, ErrorKind}; // TODO: Convert to better error handler
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Session {
        #[command(subcommand)]
        cmd: SessionCommands,
    },
    Project {
        #[command(subcommand)]
        cmd: ProjectCommands,
    },
}

#[derive(Subcommand)]
pub enum SessionCommands {
    Start,
    Pause,
    Stop,
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    New {
        #[arg(short, long)]
        name: String,
    },
}

pub fn execute(args: Args, entry_directory: &Path) -> Result<String, Error> {
    let working_directory = find_cast_toml(entry_directory);
    if let Some(working_directory) = working_directory {
        return match &args.cmd {
            Commands::Session { cmd } => match cmd {
                SessionCommands::Start => {
                    let _ = sessions::start(working_directory);
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
            Commands::Project { cmd } => match cmd {
                ProjectCommands::New { name } => {
                    let _ = projects::new(working_directory, name);
                    Ok("Creating project.".into())
                }
            },
        };
    } else {
        Ok("Could not find Cast.toml. Exiting.".into())
    }
}

fn find_cast_toml(working_directory: &Path) -> Option<&Path> {
    let mut current_directory = Some(working_directory);
    while let Some(current_path) = current_directory {
        if let Ok(entries) = fs::read_dir(current_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if entry.file_name() == "Cast.toml" {
                        return current_directory;
                    }
                }
            }
            current_directory = current_path.parent();
        }
    }

    return None;
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
                cmd: Commands::Session {
                    cmd: SessionCommands::Start,
                },
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Could not find Cast.toml. Exiting.");
    }

    #[test]
    fn it_starts_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        let result = execute(
            Args {
                cmd: Commands::Session {
                    cmd: SessionCommands::Start,
                },
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
                cmd: Commands::Session {
                    cmd: SessionCommands::Pause,
                },
            },
            tmp_dir.path(),
        )
        .unwrap();
        assert_eq!(result, "Pausing session.");
    }
    #[test]
    fn it_ends_session() {
        let tmp_dir = TempDir::new("test").unwrap();
        fs::write(tmp_dir.path().join("Cast.toml"), "").unwrap();
        let result = execute(
            Args {
                cmd: Commands::Session {
                    cmd: SessionCommands::Stop,
                },
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
                cmd: Commands::Project {
                    cmd: ProjectCommands::New {
                        name: "test".into(),
                    },
                },
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
}
