use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Error, Surreal,
};

use crate::closet::closet::{ClosetCreator, ClosetReader, Record};

pub struct SurrealDbClosetProvider {
    pub db: Surreal<Client>, // TODO: Make this private and expose all queries via the closet traits
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
    async fn create<I>(&self, item: I) -> Result<Option<Record>, Error>
    where
        I: Serialize + 'static,
    {
        let created: Option<Record> = self.db.create("experience").content(item).await?;
        return Ok(created);
    }
}

impl ClosetReader for SurrealDbClosetProvider {
    fn read(&self) -> String {
        return String::from("Hello World");
    }
}
