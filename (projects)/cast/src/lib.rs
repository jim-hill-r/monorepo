use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Root {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Timer,
}

pub fn execute(root: Root) {
    match &root.command {
        Command::Timer => {
            println!("Starting 1 hour work session.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_starts_timer() {
        let root = Root {
            command: Command::Timer,
        };
        execute(root);
    }
}
