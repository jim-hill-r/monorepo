use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::core::LuggageId;

pub type CubeSchema = String; // Currently just JSONSchema, later expand to TOML, YAML, etc...

#[derive(Serialize, JsonSchema)]
pub struct CubeDefinition {
    pub id: String, // LuggageId // TODO: Return to LuggageId after add uuid1 feature to JsonSchema cargo
    pub schema: CubeSchema,
}

impl CubeRegistration for CubeDefinition {
    fn id() -> LuggageId {
        return Uuid::try_parse("0194f27d-d3d5-7960-b953-e5d3ea1047a6").unwrap_or_default();
    }
    fn schema() -> CubeSchema {
        return serde_json::to_string_pretty(&schema_for!(CubeDefinition)).unwrap_or_default();
    }
}

pub trait CubeRegistration {
    fn id() -> LuggageId;
    fn schema() -> CubeSchema;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CubeHeader {
    pub id: LuggageId,
    pub definition: LuggageId,
}

impl CubeHeader {
    pub fn new(definition: LuggageId) -> Self {
        CubeHeader {
            id: Uuid::now_v7(),
            definition,
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
