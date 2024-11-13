use serde::{Deserialize, Serialize};

use crate::item::item::{ItemHeader, LuggageItem};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperienceCube {
    pub item_header: ItemHeader,
    pub title: String,
    pub company: String,
    pub timeframe: String,
    pub description: Vec<String>,
}

impl LuggageItem for ExperienceCube {
    fn item_header(&self) -> &ItemHeader {
        return &self.item_header;
    }
}
