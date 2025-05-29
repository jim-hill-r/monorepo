use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    core::core::LuggageId,
    cube::cube::{Cube, CubeHeader, CubeRegistration, CubeSchema},
    error::LuggageError,
};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema, Default)]
pub enum ClosetBuiltinType {
    #[default]
    LocalSurrealDb,
    RemoteSurrealDb,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, JsonSchema, Default)]
pub enum ClosetExecutionType {
    #[default]
    Builtin,
    Plugin,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, JsonSchema)]
pub struct Closet {
    pub name: String,
    pub execution_type: ClosetExecutionType,
    pub builtin_type: Option<ClosetBuiltinType>,
}

impl Default for Closet {
    fn default() -> Self {
        Self {
            name: "default-luggage-closet".into(),
            execution_type: Default::default(),
            builtin_type: Default::default(),
        }
    }
}

impl CubeRegistration for Closet {
    fn id() -> LuggageId {
        return Uuid::try_parse("0194f284-68d6-7072-805f-878ac5e94c7e").unwrap_or_default();
    }
    fn schema() -> CubeSchema {
        return serde_json::to_string_pretty(&schema_for!(Closet)).unwrap_or_default();
    }
}

pub trait ClosetCreator {
    fn create<T>(
        &self,
        cube: Cube<T>,
    ) -> impl std::future::Future<Output = Result<CubeHeader, LuggageError>> + Send
    where
        T: Serialize + Send + 'static;
}

pub trait ClosetReader {
    fn read<T>(
        &self,
        header: CubeHeader,
    ) -> impl std::future::Future<Output = Result<Cube<T>, LuggageError>> + Send
    where
        T: for<'a> Deserialize<'a> + Send;
}

pub trait ClosetUpdater {
    // TODO
    // fn update(&self) -> String;
}

pub trait ClosetDeleter {
    // TODO
    // fn delete(&self) -> String;
}
