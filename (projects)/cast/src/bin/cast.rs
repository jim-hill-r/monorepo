use cast::{execute, Root};
use clap::Parser;

fn main() {
    println!("Executing cast command...");
    execute(Root::parse());
}
