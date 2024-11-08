use serde::Serialize;

use crate::error::LuggageError;
use crate::item::item::Item;

pub trait ClosetCreator {
    async fn create<I>(&self, item: I) -> Result<Option<Item>, LuggageError>
    where
        I: Serialize + 'static;
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
