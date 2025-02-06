use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::{
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

pub trait ClosetCreator {
    async fn create<T>(&self, cube: Cube<T>) -> Result<CubeHeader, LuggageError>
    where
        T: Serialize + 'static;
}

pub trait ClosetReader {
    async fn read<T>(&self, header: CubeHeader) -> Result<Cube<T>, LuggageError>
    where
        T: for<'a> Deserialize<'a>;
}

pub trait ClosetUpdater {
    // TODO
    // fn update(&self) -> String;
}

pub trait ClosetDeleter {
    // TODO
    // fn delete(&self) -> String;
}
