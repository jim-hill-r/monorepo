pub mod closet;
pub mod cube;
pub mod error;
pub mod item;

use crate::closet::closet::ClosetCreator;
use crate::closet::providers::surrealdb::SurrealDbClosetProvider;
use crate::cube::providers::resume::experience::Experience;
use crate::error::Result;
use crate::item::item::ItemId;

#[tokio::main]
async fn main() -> Result<()> {
    let provider =
        SurrealDbClosetProvider::new("127.0.0.1:8000", "root", "root", "test", "test").await?;

    let created = provider
        .create(Experience {
            id: ItemId { id: "test" },
            title: "test",
            company: "test",
            timeframe: "test",
            description: vec!["test"],
        })
        .await?;
    dbg!(created);

    Ok(())
}
