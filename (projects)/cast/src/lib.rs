use chrono::prelude::*;
use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, write};
use std::io::Error; // TODO: Convert to better error handler
use std::path::Path;
use uuid::Uuid;

const SESSIONS_DIRECTORY: &str = ".cast/sessions";
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
}

#[derive(Subcommand)]
enum SessionCommands {
    Start,
    Pause,
    Stop,
}

pub fn execute(args: Args, cast_directory: &Path) -> Result<String, Error> {
    return match &args.cmd {
        Commands::Session { cmd } => match cmd {
            SessionCommands::Start => {
                let sessions_directory = cast_directory.join(SESSIONS_DIRECTORY);
                create_dir_all(&sessions_directory)?;

                let id = Uuid::now_v7();
                write(
                    sessions_directory.join(id.to_string()),
                    Utc::now().to_string(),
                )?;
                Ok("Starting session.".to_string())
            }
            SessionCommands::Pause => Ok("Pausing session.".to_string()),
            SessionCommands::Stop => Ok("Stopping session.".to_string()),
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
}
