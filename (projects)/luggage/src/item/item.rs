use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Item<'a> {
    #[serde(borrow)]
    pub(crate) id: ItemId<'a>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemId<'a> {
    pub(crate) id: &'a str,
}
