use provider::abstractions::Reader;
use provider::surrealdb::SurrealDbProvider;

pub mod provider;

fn main() {
    let provider = SurrealDbProvider;
    println!("{}", provider.get());
}
