use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
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

/// OpenAPI documentation for Bellhop API
#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        create_cube,
    ),
    components(
        schemas(Health, CreateCube, BellhopHeader, CubeHeader)
    ),
    tags(
        (name = "bellhop", description = "Bellhop API endpoints")
    ),
    info(
        title = "Bellhop API",
        version = "1.0.0",
        description = "API for managing cubes and closets in the Luggage portable data platform",
    )
)]
struct ApiDoc;

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

    if let Some(c) = config
        && let Some(closet) = c.closet
    {
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

    Ok(router(AppState {
        root_closet_id,
        closet_registry,
        closet_providers,
    })
    .await)
}

async fn router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(health_check))
        .route("/v1/cube", post(create_cube))
        .with_state(state)
}

pub async fn listener() -> tokio::net::TcpListener {
    tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap()
}

#[derive(Serialize, utoipa::ToSchema)]
struct Health {
    description: String,
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service is healthy", body = Health)
    ),
    tag = "bellhop"
)]
async fn health_check() -> Json<Health> {
    Json(Health {
        description: "belayon".into(),
    })
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
struct BellhopHeader {
    #[schema(value_type = Option<String>)]
    closet_id: Option<LuggageId>,
}

#[derive(Deserialize, Serialize, utoipa::ToSchema)]
struct CreateCube {
    bellhop_header: BellhopHeader,
    cube_header: CubeHeader,
    content: String,
}

/// Create a new cube
#[utoipa::path(
    post,
    path = "/v1/cube",
    request_body = CreateCube,
    responses(
        (status = 201, description = "Cube created successfully", body = CubeHeader),
        (status = 500, description = "Internal server error", body = CubeHeader)
    ),
    tag = "bellhop"
)]
async fn create_cube(
    State(state): State<AppState>,
    Json(payload): Json<CreateCube>,
) -> (StatusCode, Json<CubeHeader>) {
    if let Some(provider) = state.closet_providers.get(&state.root_closet_id) {
        let cube = Cube::new(payload.cube_header.clone(), payload.content);
        let _ = provider.create(cube).await;
        (StatusCode::CREATED, Json(payload.cube_header))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(payload.cube_header))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        closet::closet::Closet,
        cube::cube::{CubeDefinition, CubeRegistration, CubeSchema},
        error::Result,
    };
    use axum_test::TestServer;
    use schemars::{JsonSchema, schema_for};
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
            Uuid::try_parse("0194f2fe-6f7a-7dd2-8af3-d6d4c9a2f74a").unwrap_or_default()
        }
        fn schema() -> CubeSchema {
            serde_json::to_string_pretty(&schema_for!(TestContent)).unwrap_or_default()
        }
    }

    #[tokio::test]
    async fn create_cube_definition() -> Result<()> {
        let _test_name = "create_cube_definition";
        let server = TestServer::new(app(None).await?)?;
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
        let server = TestServer::new(app(None).await?)?;
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
        let server = TestServer::new(app(None).await?)?;
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

    #[tokio::test]
    async fn swagger_ui_is_accessible() -> Result<()> {
        let server = TestServer::new(app(None).await?).unwrap();
        let response = server.get("/swagger-ui/").await;
        response.assert_status(StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn openapi_json_is_accessible() -> Result<()> {
        let server = TestServer::new(app(None).await?).unwrap();
        let response = server.get("/api-docs/openapi.json").await;
        response.assert_status(StatusCode::OK);

        // Verify it's valid JSON
        let json_text = response.text();
        let openapi_spec: serde_json::Value =
            serde_json::from_str(&json_text).expect("OpenAPI spec should be valid JSON");

        // Verify basic OpenAPI structure
        assert!(
            openapi_spec.get("openapi").is_some(),
            "OpenAPI version should be present"
        );
        assert!(
            openapi_spec.get("info").is_some(),
            "Info section should be present"
        );
        assert!(
            openapi_spec.get("paths").is_some(),
            "Paths section should be present"
        );

        Ok(())
    }
}
