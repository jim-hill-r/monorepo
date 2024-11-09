use serde::{Deserialize, Serialize};

use crate::item::item::{ItemHeader, LuggageItem};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperienceCube<'a> {
    pub item_header: ItemHeader<'a>,
    pub title: &'a str,
    pub company: &'a str,
    pub timeframe: &'a str,
    pub description: Vec<&'a str>,
}

impl LuggageItem for ExperienceCube<'_> {
    fn item_header(&self) -> &ItemHeader {
        return &self.item_header;
    }
}
