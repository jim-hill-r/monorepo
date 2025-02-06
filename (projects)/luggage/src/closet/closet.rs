use serde::{Deserialize, Serialize};

use crate::{
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

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
