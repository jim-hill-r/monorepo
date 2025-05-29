use clap::{Parser, Subcommand};
use std::io::Error; // TODO: Convert to better error handler
use std::path::Path;

pub mod sessions;

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
                let _ = sessions::start(cast_directory);
                Ok("Starting session.".to_string())
            }
            SessionCommands::Pause => {
                let _ = sessions::pause(cast_directory);
                Ok("Pausing session.".to_string())
            }
            SessionCommands::Stop => {
                let _ = sessions::stop(cast_directory);
                Ok("Stopping session.".to_string())
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
}
