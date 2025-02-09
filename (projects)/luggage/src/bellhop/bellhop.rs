use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use surrealdb::engine::local::Db;
use urn::Urn;

use crate::{
    closet::{
        closet::{Closet, ClosetCreator, ClosetReader, ClosetType},
        providers::surrealdb::SurrealDbClosetProvider,
    },
    error::LuggageError,
};

use super::startup::StartupConfiguration;

pub trait AppState: Clone + Send + Sync + 'static {
    type C: ClosetCreator + ClosetReader;

    fn closet_provider(&self) -> &Self::C;
}

#[derive(Clone)]
pub struct LocalSurrealDbAppState {
    pub closet_provider: SurrealDbClosetProvider<Db>,
}

impl LocalSurrealDbAppState {
    async fn new() -> Result<Self, LuggageError> {
        return Ok(LocalSurrealDbAppState {
            closet_provider: SurrealDbClosetProvider::<Db>::new("bellhop", "bellhop").await?,
        });
    }
}

impl AppState for LocalSurrealDbAppState {
    type C = SurrealDbClosetProvider<Db>;

    fn closet_provider(&self) -> &Self::C {
        return &self.closet_provider;
    }
}

pub async fn router(config: Option<StartupConfiguration>) -> Result<Router, LuggageError> {
    let state = if let Some(c) = config {
        match c.closet_type {
            ClosetType::LocalSurrealDb => LocalSurrealDbAppState::new().await?,
            ClosetType::RemoteSurrealDb => LocalSurrealDbAppState::new().await?, // TODO: Implement remote
        }
    } else {
        LocalSurrealDbAppState::new().await?
    };

    return Ok(build(state).await);
}

async fn build<S: AppState>(state: S) -> Router {
    let router = Router::new()
        .route("/", get(health_check))
        .route("/v1/type", post(create_type))
        .route("/v1/closet", post(create_closet::<S>))
        .with_state(state);

    return router;
}

pub async fn listener() -> tokio::net::TcpListener {
    return tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
}
type LuggageSchema = String; // Currently just JSONSchema, later expand to TOML, YAML, etc...
#[derive(Serialize)]
struct Health {
    description: String,
}

async fn health_check() -> Json<Health> {
    Json(Health {
        description: "belayon".into(),
    })
}

#[derive(Deserialize)]
struct CreateType {
    urn: Urn,
    schema: LuggageSchema,
    version: Version,
}

#[derive(Serialize)]
struct Type {
    urn: Urn,
    schema: LuggageSchema,
    version: Version,
}

impl From<CreateType> for Type {
    fn from(value: CreateType) -> Self {
        return Type {
            urn: value.urn,
            schema: value.schema,
            version: value.version,
        };
    }
}

async fn create_type(Json(payload): Json<CreateType>) -> (StatusCode, Json<Type>) {
    let r#type = payload.into();

    // TODO: Save to backend

    return (StatusCode::CREATED, Json(r#type));
}

#[derive(Deserialize, Clone)]
struct CreateCloset {
    r#type: ClosetType,
}

impl From<CreateCloset> for Closet {
    fn from(value: CreateCloset) -> Self {
        return Closet {
            r#type: value.r#type,
        };
    }
}

async fn create_closet<S: AppState>(
    State(state): State<S>,
    Json(payload): Json<CreateCloset>,
) -> (StatusCode, Json<Closet>) {
    let provider = state.closet_provider();
    let closet: Closet = payload.into();
    let _ = provider.create(closet.clone().into()).await;
    return (StatusCode::CREATED, Json(closet));
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use axum_test::TestServer;
    use convert_case::{Case, Casing};
    use schemars::{schema_for, JsonSchema};
    use serde_json::json;

    use super::*;

    #[derive(JsonSchema)]
    struct TestContent {
        name: String,
    }

    #[tokio::test]
    async fn create_type() -> Result<()> {
        let test_name = "create_type";
        let server = TestServer::new(router(None).await?).unwrap(); //TODO: Remove unwrap (use ?)
        let _response = server
            .post("/v1/type")
            .json(&json!({
                "urn": format!("lug:://{}",test_name.to_case(Case::Kebab)),
                "schema": schema_for!(TestContent),
                "version": "1.0.0"
            }))
            .await;
        Ok(())
    }

    // TODO: Write create_closet test
}
