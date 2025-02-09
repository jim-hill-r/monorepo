use serde::{Deserialize, Serialize};

use crate::{
    cube::cube::{Cube, CubeDefinition, CubeHeader},
    error::LuggageError,
};

// TODO: Figure out how to make this extensible for third-parties
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum ClosetType {
    RemoteSurrealDb,
    LocalSurrealDb,
}

#[derive(Serialize, Clone)]
pub struct Closet {
    pub r#type: ClosetType,
}

impl Into<Cube<Closet>> for Closet {
    fn into(self) -> Cube<Closet> {
        return Cube::new(CubeHeader::new(Closet::urn()), self);
    }
}

impl CubeDefinition for Closet {
    fn urn() -> crate::cube::cube::CubeType {
        return "lug://closet-definition".into();
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
