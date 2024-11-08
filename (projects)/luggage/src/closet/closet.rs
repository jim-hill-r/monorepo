use serde::{Deserialize, Serialize};
use surrealdb::{Error, RecordId}; // TODO: Remove coupling with surrealdb

#[derive(Debug, Deserialize)]
pub struct Record {
    id: RecordId, // TODO: Remove coupling with surrealdb
}

pub trait ClosetCreator {
    async fn create<I>(&self, item: I) -> Result<Option<Record>, Error>
    where
        I: Serialize + 'static; // TODO: Remove coupling with surrealdb
}

pub trait ClosetReader {
    fn read(&self) -> String;
}

pub trait ClosetUpdater {
    fn update(&self) -> String;
}

pub trait ClosetDeleter {
    fn delete(&self) -> String;
}
