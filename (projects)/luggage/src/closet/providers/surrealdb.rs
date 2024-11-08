use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Error, RecordId, Surreal,
};

use crate::{
    closet::closet::{ClosetCreator, ClosetReader},
    error::LuggageError,
    item::item::{Item, ItemId},
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

impl From<Record> for Item<'_> {
    fn from(record: Record) -> Self {
        return Item {
            id: ItemId {
                id: "TODO", // TODO: Figure out how to represent an ID in surrealdb
            },
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
    async fn create<I>(&self, item: I) -> Result<Option<Item>, LuggageError>
    where
        I: Serialize + 'static,
    {
        let created: Option<Record> = self.db.create("experience").content(item).await?;
        return match created {
            Some(record) => Ok(Some(Item::from(record))),
            None => Err(LuggageError::Unknown),
        };
    }
}

impl ClosetReader for SurrealDbClosetProvider {
    fn read(&self) -> String {
        return String::from("Hello World");
    }
}
