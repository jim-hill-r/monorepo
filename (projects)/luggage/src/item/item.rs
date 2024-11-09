use serde::{Deserialize, Serialize};

pub trait LuggageItem {
    fn item_header(&self) -> &ItemHeader;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemHeader<'a> {
    pub(crate) id: &'a str,
    pub(crate) r#type: &'a str,
}
