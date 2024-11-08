use serde::Serialize;

use crate::item::item::ItemId;

#[derive(Debug, Serialize)]
pub struct Experience<'a> {
    pub id: ItemId<'a>,
    pub title: &'a str,
    pub company: &'a str,
    pub timeframe: &'a str,
    pub description: Vec<&'a str>,
}
