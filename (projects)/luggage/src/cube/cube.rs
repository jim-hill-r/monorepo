use serde::{Deserialize, Serialize};
use urn::Urn;
use uuid::Uuid;

pub type CubeId = Uuid; // V7
pub type CubeType = Urn; // URN

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CubeHeader {
    pub id: CubeId,
    pub r#type: CubeType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cube<T> {
    pub header: CubeHeader,
    pub content: Option<T>,
}
