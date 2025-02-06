use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
    Error, // TODO: Convert errors to luggage errors
    RecordId,
    Surreal,
};

use crate::{
    closet::closet::{ClosetCreator, ClosetReader},
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

#[derive(Debug, Deserialize)]
struct Record {
    id: RecordId,
}

impl From<Error> for LuggageError {
    fn from(_value: Error) -> Self {
        return LuggageError::Unknown;
    }
}

pub struct SurrealDbClosetProvider<T>
where
    T: surrealdb::Connection,
{
    db: Surreal<T>,
}

impl SurrealDbClosetProvider<Client> {
    pub async fn new<'a>(
        url: &'a str,
        username: &'a str,
        password: &'a str,
        namespace: &'a str,
        database: &'a str,
    ) -> Result<Self, Error> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root { username, password }).await?;

        db.use_ns(namespace).use_db(database).await?;

        return Ok(Self { db });
    }
}

impl SurrealDbClosetProvider<Db> {
    pub async fn new<'a>(namespace: &str, database: &'a str) -> Result<Self, Error> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(namespace).use_db(database).await?;
        return Ok(Self { db });
    }
}

impl<C> ClosetCreator for SurrealDbClosetProvider<C>
where
    C: surrealdb::Connection,
{
    async fn create<T>(&self, cube: Cube<T>) -> Result<CubeHeader, LuggageError>
    where
        T: Serialize + 'static,
    {
        let created: Option<Record> = self
            .db
            .create((&cube.header.r#type, &cube.header.id))
            .content(cube.content)
            .await?;
        dbg!(&created);
        match created {
            Some(_) => Ok(cube.header),
            None => Err(LuggageError::Unknown),
        }
    }
}

impl<C> ClosetReader for SurrealDbClosetProvider<C>
where
    C: surrealdb::Connection,
{
    async fn read<T>(&self, header: CubeHeader) -> Result<Cube<T>, LuggageError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let saved_content: Option<T> = self.db.select((&header.r#type, &header.id)).await?;
        return Ok(Cube {
            header,
            content: saved_content,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use surrealdb::engine::local::Db;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestContent {
        name: String,
    }

    #[tokio::test]
    async fn create_then_read_test_content() -> Result<()> {
        let test_name = "create_then_read_test_content";
        let test_cube = Cube {
            header: CubeHeader {
                id: test_name.into(),
                r#type: "Test".into(),
            },
            content: Some(TestContent {
                name: "test".into(),
            }),
        };
        let closet = SurrealDbClosetProvider::<Db>::new(test_name, "test").await?;
        let saved_header = closet.create(test_cube.clone()).await?;
        dbg!(&saved_header);
        let saved_cube: Cube<TestContent> = closet.read(saved_header).await?;
        assert_eq!(&test_cube.content, &saved_cube.content);
        Ok(())
    }
}
