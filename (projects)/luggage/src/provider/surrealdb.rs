use super::abstractions::Reader;

pub struct SurrealDbProvider;

impl Reader for SurrealDbProvider {
    fn get(&self) -> String {
        return String::from("Hello World");
    }
}
