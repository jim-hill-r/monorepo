use crate::{projects, sessions};
use clap::{Parser, Subcommand};
use std::io::Error; // TODO: Convert to better error handler
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

pub fn execute(args: Args, working_directory: &Path) -> Result<String, Error> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn it_starts_session() {
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
        assert_eq!(result, "Starting session.");
    }
    #[test]
    fn it_pauses_session() {
        let tmp_dir = TempDir::new("test").unwrap();
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
}
