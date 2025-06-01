use cast::args::{Args, execute};
use clap::Parser;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::current_dir()?;
    let result_message = execute(Args::parse(), path.as_path())?;
    println!("{}", result_message);
    return Ok(());
}
