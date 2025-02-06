use serde::{Deserialize, Serialize};

pub type CubeId = String; // TODO: Formalize this as UUIDv7
pub type Urn = String; // TODO: Formalize this as more than string alias

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CubeHeader {
    pub id: CubeId,
    pub r#type: Urn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cube<T> {
    pub header: CubeHeader,
    pub content: Option<T>,
}
