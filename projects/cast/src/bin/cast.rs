use cast::args::{execute, Args};
use clap::Parser;
use std::env;

fn main() {
    println!("Executing cast command...");
    let path = env::current_dir().unwrap();
    let result_message = execute(Args::parse(), path.as_path()).unwrap(); // TODO: handle errors
    println!("{}", result_message);
}
