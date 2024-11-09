use serde::{Deserialize, Serialize};

use crate::{
    error::LuggageError,
    item::item::{ItemHeader, LuggageItem},
};

pub trait ClosetCreator {
    async fn create<I>(&self, item: I) -> Result<Option<ItemHeader>, LuggageError>
    where
        I: Serialize + LuggageItem + 'static;
}

pub trait ClosetReader {
    fn read(&self, item_header: ItemHeader) -> String;

    // async fn read<'a, I>(&self, item: ItemHeader) -> Result<Option<I>, LuggageError>
    // where
    //     I: Deserialize<'a> + 'static;
}

pub trait ClosetUpdater {
    fn update(&self) -> String;
}

pub trait ClosetDeleter {
    fn delete(&self) -> String;
}
