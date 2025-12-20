use std::collections::HashMap;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use uuid::Uuid;

use crate::{
    closet::{
        closet::{Closet, ClosetBuiltinType, ClosetCreator},
        providers::surrealdb::SurrealDbClosetProvider,
    },
    core::core::LuggageId,
    cube::cube::{Cube, CubeHeader},
    error::LuggageError,
};

use super::startup::StartupConfiguration;

#[derive(Clone)]
pub struct AppState {
    pub root_closet_id: Uuid,
    pub closet_registry: HashMap<Uuid, Closet>,
    pub closet_providers: HashMap<Uuid, SurrealDbClosetProvider<Db>>, // TODO: Make the provider abstract
}

pub async fn app(config: Option<StartupConfiguration>) -> Result<Router, LuggageError> {
    let root_closet_id = Uuid::now_v7();
    let root_closet = Closet::default();
    let root_closet_provider = SurrealDbClosetProvider::<Db>::new("bellhop", "bellhop").await?;
    let mut closet_registry: HashMap<Uuid, Closet> = HashMap::new();
    closet_registry.insert(root_closet_id, root_closet);
    let mut closet_providers: HashMap<Uuid, SurrealDbClosetProvider<Db>> = HashMap::new();
    closet_providers.insert(root_closet_id, root_closet_provider);

    if let Some(c) = config {
        if let Some(closet) = c.closet {
            let additional_closet_id = Uuid::now_v7();
            closet_registry.insert(additional_closet_id, closet.clone());
            if let Some(t) = closet.builtin_type {
                let closet_provider = match t {
                    ClosetBuiltinType::LocalSurrealDb => {
                        SurrealDbClosetProvider::<Db>::new("bellhop", "bellhop").await?
                    }
                    ClosetBuiltinType::RemoteSurrealDb => {
                        // TODO: Actually connect to a remote db
                        SurrealDbClosetProvider::<Db>::new("bellhop", "bellhop").await?
                    }
                };
                closet_providers.insert(additional_closet_id, closet_provider);
            }
        }
    }

    return Ok(router(AppState {
        root_closet_id,
        closet_registry,
        closet_providers,
    })
    .await);
}

async fn router(state: AppState) -> Router {
    let router = Router::new()
        .route("/", get(health_check))
        .route("/v1/cube", post(create_cube))
        .with_state(state);

    return router;
}

pub async fn listener() -> tokio::net::TcpListener {
    return tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
}

#[derive(Serialize)]
struct Health {
    description: String,
}

async fn health_check() -> Json<Health> {
    Json(Health {
        description: "belayon".into(),
    })
}

#[derive(Deserialize, Serialize)]
struct BellhopHeader {
    closet_id: Option<LuggageId>,
}

#[derive(Deserialize, Serialize)]
struct CreateCube {
    bellhop_header: BellhopHeader,
    cube_header: CubeHeader,
    content: String,
}

async fn create_cube(
    State(state): State<AppState>,
    Json(payload): Json<CreateCube>,
) -> (StatusCode, Json<CubeHeader>) {
    if let Some(provider) = state.closet_providers.get(&state.root_closet_id) {
        let cube = Cube::new(payload.cube_header.clone(), payload.content);
        let _ = provider.create(cube).await;
        return (StatusCode::CREATED, Json(payload.cube_header));
    }
    return (StatusCode::INTERNAL_SERVER_ERROR, Json(payload.cube_header));
}

#[cfg(test)]
mod tests {
    use crate::{
        closet::closet::Closet,
        cube::cube::{CubeDefinition, CubeRegistration, CubeSchema},
        error::Result,
    };
    use axum_test::TestServer;
    use schemars::{schema_for, JsonSchema};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    #[derive(Serialize, JsonSchema)]
    struct TestContent {
        name: String,
        todos: Vec<String>,
    }

    impl CubeRegistration for TestContent {
        fn id() -> LuggageId {
            return Uuid::try_parse("0194f2fe-6f7a-7dd2-8af3-d6d4c9a2f74a").unwrap_or_default();
        }
        fn schema() -> CubeSchema {
            return serde_json::to_string_pretty(&schema_for!(TestContent)).unwrap_or_default();
        }
    }

    #[tokio::test]
    async fn create_cube_definition() -> Result<()> {
        let _test_name = "create_cube_definition";
        let server = TestServer::new(app(None).await?).unwrap(); //TODO: Remove unwrap (use ?)
        let cube = CreateCube {
            bellhop_header: BellhopHeader { closet_id: None },
            cube_header: CubeHeader {
                id: Uuid::now_v7(),
                definition: CubeDefinition::id(),
            },
            content: json!({
                "id": CubeDefinition::id(),
                "schema": CubeDefinition::schema()
            })
            .to_string(),
        };
        let response = server.post("/v1/cube").json(&cube).await;
        response.assert_status(StatusCode::CREATED);
        Ok(())
    }

    #[tokio::test]
    async fn create_closet() -> Result<()> {
        let _test_name = "create_closet";
        let server = TestServer::new(app(None).await?).unwrap(); //TODO: Remove unwrap (use ?)
        let cube = CreateCube {
            bellhop_header: BellhopHeader { closet_id: None },
            cube_header: CubeHeader {
                id: Uuid::now_v7(),
                definition: Closet::id(),
            },
            content: json!({
                "id": Closet::id(),
                "schema": Closet::schema()
            })
            .to_string(),
        };
        let response = server.post("/v1/cube").json(&cube).await;
        response.assert_status(StatusCode::CREATED);
        Ok(())
    }

    #[tokio::test]
    async fn create_test_content_in_default_provider() -> Result<()> {
        let test_name = "create_test_content_using_default_provider";
        let server = TestServer::new(app(None).await?).unwrap(); //TODO: Remove unwrap (use ?)
        let test_cube_definition_id = Uuid::now_v7();
        let test_cube_definition = CreateCube {
            bellhop_header: BellhopHeader { closet_id: None },
            cube_header: CubeHeader {
                id: test_cube_definition_id,
                definition: CubeDefinition::id(),
            },
            content: json!({
                "id": TestContent::id(),
                "schema": TestContent::schema()
            })
            .to_string(),
        };
        let definition_response = server.post("/v1/cube").json(&test_cube_definition).await;
        definition_response.assert_status(StatusCode::CREATED);

        let test_cube_id = Uuid::now_v7();
        let test_content = TestContent {
            name: test_name.into(),
            todos: vec!["todo_one".into(), "todo_two".into()],
        };
        let test_cube = CreateCube {
            bellhop_header: BellhopHeader { closet_id: None },
            cube_header: CubeHeader {
                id: test_cube_id,
                definition: TestContent::id(),
            },
            content: serde_json::to_string(&test_content)
                .expect("test_content should serialize to json."),
        };
        let test_response = server.post("/v1/cube").json(&test_cube_definition).await;
        test_response.assert_status(StatusCode::CREATED);

        Ok(())
    }
}
