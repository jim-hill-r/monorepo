use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Error, RecordId, Surreal,
};

use crate::{
    closet::closet::{ClosetCreator, ClosetReader},
    error::LuggageError,
    item::item::ItemHeader,
};

#[derive(Debug, Deserialize)]
struct Record {
    id: RecordId,
}

impl From<Error> for LuggageError {
    fn from(_value: Error) -> Self {
        return LuggageError::Unknown;
    }
}

impl From<Record> for ItemHeader<'_> {
    fn from(record: Record) -> Self {
        return ItemHeader {
            id: "TODO",
            r#type: "TODO",
        };
    }
}

pub struct SurrealDbClosetProvider {
    db: Surreal<Client>,
}

impl SurrealDbClosetProvider {
    pub async fn new<'a>(
        url: &'a str,
        username: &'a str,
        password: &'a str,
        namespace: &'a str,
        database: &'a str,
    ) -> surrealdb::Result<Self> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root { username, password }).await?;

        db.use_ns(namespace).use_db(database).await?;

        return Ok(Self { db });
    }
}

impl ClosetCreator for SurrealDbClosetProvider {
    async fn create<I>(&self, item: I) -> Result<Option<ItemHeader>, LuggageError>
    where
        I: Serialize + 'static,
    {
        let created: Option<Record> = self.db.create("experience").content(item).await?;
        return match created {
            Some(record) => Ok(Some(ItemHeader::from(record))),
            None => Err(LuggageError::Unknown),
        };
    }
}

impl ClosetReader for SurrealDbClosetProvider {
    fn read(&self, _item_header: ItemHeader) -> String {
        return String::from("Hello World");
    }

    // async fn read<'a, I>(&self, item_header: ItemHeader<'a>) -> Result<Option<I>, LuggageError>
    // where
    //     I: Deserialize<'a> + 'static,
    // {
    //     self.db
    //         .query("SELECT * FROM type::table($table)")
    //         .bind(("table", item_header.r#type))
    //         .await?
    // }
}
