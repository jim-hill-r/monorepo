pub mod closet;
pub mod cube;
pub mod error;
pub mod item;

use closet::closet::ClosetReader;
use item::item::ItemHeader;

use crate::closet::closet::ClosetCreator;
use crate::closet::providers::surrealdb::SurrealDbClosetProvider;
use crate::cube::providers::resume::experience::ExperienceCube;
use crate::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let provider =
        SurrealDbClosetProvider::new("127.0.0.1:8000", "root", "root", "test", "test").await?;

    let created = provider
        .create(ExperienceCube {
            item_header: ItemHeader {
                id: "test",
                r#type: "Experience",
            },
            title: "test",
            company: "test",
            timeframe: "test",
            description: vec!["test"],
        })
        .await?;
    dbg!(created);

    let cube = provider.read(ItemHeader {
        id: "test",
        r#type: "Experience",
    });
    dbg!(cube);

    Ok(())
}
