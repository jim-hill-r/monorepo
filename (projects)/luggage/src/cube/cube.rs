use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type CubeId = Uuid; // V7
pub type CubeType = String; // Unique URI // TODO: Define/document schema

pub trait CubeDefinition {
    fn urn() -> CubeType;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CubeHeader {
    pub id: CubeId,
    pub r#type: CubeType,
}

impl CubeHeader {
    pub fn new(r#type: CubeType) -> Self {
        CubeHeader {
            id: Uuid::now_v7(),
            r#type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cube<T> {
    pub header: CubeHeader,
    pub content: Option<T>,
}

impl<T> Cube<T> {
    pub fn new(header: CubeHeader, content: T) -> Self {
        return Cube::<T> {
            header,
            content: Some(content),
        };
    }
}
