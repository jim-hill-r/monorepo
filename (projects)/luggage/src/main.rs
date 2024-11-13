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
                id: String::from("test"),
                r#type: String::from("Experience"),
            },
            title: String::from("test"),
            company: String::from("test"),
            timeframe: String::from("test"),
            description: vec![String::from("test")],
        })
        .await?;
    dbg!(created);

    let cube = provider.read(ItemHeader {
        id: String::from("test"),
        r#type: String::from("Experience"),
    });
    dbg!(cube);

    Ok(())
}
