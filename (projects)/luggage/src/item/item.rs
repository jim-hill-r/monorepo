use serde::{Deserialize, Serialize};

pub trait LuggageItem {
    fn item_header(&self) -> &ItemHeader;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemHeader {
    pub(crate) id: String,
    pub(crate) r#type: String,
}
