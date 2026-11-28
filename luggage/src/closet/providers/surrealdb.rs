use serde::{Deserialize, Serialize};
use surrealdb::{
    Surreal,
    engine::{
        local::{Db, Mem},
        remote::ws::{Client, Ws},
    },
    opt::auth::Root,
};

use crate::{
    closet::closet::{ClosetCreator, ClosetDeleter, ClosetReader, ClosetUpdater},
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

/// The record a cube is stored as: its definition names the table and its id
/// is the record key.
fn record_of(header: &CubeHeader) -> (String, surrealdb::types::Uuid) {
    (header.definition.to_string(), header.id.into())
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
    ) -> Result<Self, LuggageError> {
        let db = Surreal::new::<Ws>(url).await?;

        db.signin(Root {
            username: username.to_owned(),
            password: password.to_owned(),
        })
        .await?;

        db.use_ns(namespace).use_db(database).await?;

        Ok(Self { db })
    }
}

impl SurrealDbClosetProvider<Db> {
    pub async fn new(namespace: &str, database: &str) -> Result<Self, LuggageError> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(namespace).use_db(database).await?;
        Ok(Self { db })
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
        let created: Option<serde_json::Value> = self
            .db
            .create(record_of(&cube.header))
            .content(serde_json::to_value(cube.content)?)
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
        let saved: Option<serde_json::Value> = self.db.select(record_of(&header)).await?;
        let content = saved.map(serde_json::from_value).transpose()?;
        Ok(Cube { header, content })
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
        let updated: Option<serde_json::Value> = self
            .db
            .update(record_of(&cube.header))
            .content(serde_json::to_value(cube.content)?)
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
        let _deleted: Option<serde_json::Value> = self.db.delete(record_of(&header)).await?;
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
