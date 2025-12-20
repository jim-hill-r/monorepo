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
    closet::closet::{ClosetCreator, ClosetDeleter, ClosetReader, ClosetUpdater},
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

#[allow(dead_code)] // TODO: Figure out how to write create without an unused id field
#[derive(Debug, Deserialize)]
struct Record {
    id: RecordId,
}

impl From<surrealdb::Error> for LuggageError {
    fn from(_value: surrealdb::Error) -> Self {
        return LuggageError::Unknown;
    }
}

#[derive(Clone)]
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
        T: Serialize + Send + 'static,
    {
        let created: Option<Record> = self
            .db
            .create((cube.header.definition, cube.header.id))
            .content(cube.content)
            .await?;
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
        T: for<'a> Deserialize<'a> + Send,
    {
        let saved_content: Option<T> = self.db.select((header.definition, header.id)).await?;
        return Ok(Cube {
            header,
            content: saved_content,
        });
    }
}

impl<C> ClosetUpdater for SurrealDbClosetProvider<C>
where
    C: surrealdb::Connection,
{
    async fn update<T>(&self, cube: Cube<T>) -> Result<CubeHeader, LuggageError>
    where
        T: Serialize + Send + 'static,
    {
        let updated: Option<Record> = self
            .db
            .update((cube.header.definition, cube.header.id))
            .content(cube.content)
            .await?;
        match updated {
            Some(_) => Ok(cube.header),
            None => Err(LuggageError::Unknown),
        }
    }
}

impl<C> ClosetDeleter for SurrealDbClosetProvider<C>
where
    C: surrealdb::Connection,
{
    async fn delete(&self, header: CubeHeader) -> Result<(), LuggageError> {
        let _deleted: Option<Record> = self.db.delete((header.definition, header.id)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use surrealdb::engine::local::Db;
    use uuid::Uuid;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestContent {
        name: String,
    }

    #[tokio::test]
    async fn create_then_read_test_content() -> Result<()> {
        let test_name = "create_then_read_test_content";
        let test_cube = Cube {
            header: CubeHeader::new(Uuid::now_v7()),
            content: Some(TestContent {
                name: "test".into(),
            }),
        };
        let closet = SurrealDbClosetProvider::<Db>::new(test_name, "test").await?;
        let saved_header = closet.create(test_cube.clone()).await?;
        let saved_cube: Cube<TestContent> = closet.read(saved_header).await?;
        assert_eq!(&test_cube.content, &saved_cube.content);
        Ok(())
    }

    #[tokio::test]
    async fn create_then_update_test_content() -> Result<()> {
        let test_name = "create_then_update_test_content";
        let test_cube = Cube {
            header: CubeHeader::new(Uuid::now_v7()),
            content: Some(TestContent {
                name: "original".into(),
            }),
        };
        let closet = SurrealDbClosetProvider::<Db>::new(test_name, "test").await?;
        let saved_header = closet.create(test_cube.clone()).await?;

        // Update the content
        let updated_cube = Cube {
            header: saved_header.clone(),
            content: Some(TestContent {
                name: "updated".into(),
            }),
        };
        let updated_header = closet.update(updated_cube.clone()).await?;
        assert_eq!(saved_header.id, updated_header.id);

        // Read back and verify the update
        let read_cube: Cube<TestContent> = closet.read(updated_header).await?;
        assert_eq!(&updated_cube.content, &read_cube.content);
        Ok(())
    }

    #[tokio::test]
    async fn create_then_delete_test_content() -> Result<()> {
        let test_name = "create_then_delete_test_content";
        let test_cube = Cube {
            header: CubeHeader::new(Uuid::now_v7()),
            content: Some(TestContent {
                name: "to_be_deleted".into(),
            }),
        };
        let closet = SurrealDbClosetProvider::<Db>::new(test_name, "test").await?;
        let saved_header = closet.create(test_cube.clone()).await?;

        // Delete the content
        closet.delete(saved_header.clone()).await?;

        // Read back and verify it's deleted (content should be None)
        let read_cube: Cube<TestContent> = closet.read(saved_header).await?;
        assert_eq!(None, read_cube.content);
        Ok(())
    }
}
