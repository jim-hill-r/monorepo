pub mod closet;
pub mod cube;
use closet::closet::ClosetCreator;

use crate::closet::providers::surrealdb::SurrealDbClosetProvider;
use crate::cube::providers::resume::experience::Experience;

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    let provider =
        SurrealDbClosetProvider::new("127.0.0.1:8000", "root", "root", "test", "test").await?;

    let created = provider
        .create(Experience {
            title: "test",
            company: "test",
            timeframe: "test",
            description: vec!["test"],
        })
        .await?;
    dbg!(created);

    Ok(())
}
