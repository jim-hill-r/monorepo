use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use urn::Urn;

pub fn router() -> Router {
    return Router::new()
        .route("/", get(health_check))
        .route("/v1/type", post(create_type))
        .route("/v1/closet", post(create_closet));
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

#[derive(Serialize, Deserialize)]
enum ClosetType {
    RemoteSurrealDb,
    LocalSurrealDb,
}

#[derive(Deserialize)]
struct CreateCloset {
    r#type: ClosetType,
}

#[derive(Serialize)]
struct Closet {
    r#type: ClosetType,
}

impl From<CreateCloset> for Closet {
    fn from(value: CreateCloset) -> Self {
        return Closet {
            r#type: value.r#type,
        };
    }
}

async fn create_closet(Json(payload): Json<CreateCloset>) -> (StatusCode, Json<Closet>) {
    let closet = payload.into();

    // TODO: Save to backend

    return (StatusCode::CREATED, Json(closet));
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use axum_test::TestServer;
    use convert_case::{Case, Casing};
    use schemars::{schema_for, JsonSchema};
    use serde_json::json;
    use urn::UrnBuilder;

    use super::*;

    #[derive(JsonSchema)]
    struct TestContent {
        name: String,
    }

    #[tokio::test]
    async fn create_type() -> Result<()> {
        let test_name = "create_type";
        let server = TestServer::new(router()).unwrap(); //TODO: Remove unwrap (use ?)
        let _response = server
            .post("/v1/type")
            .json(&json!({
                "urn": UrnBuilder::new(&test_name.to_case(Case::Kebab), "1234:5678").build()?,
                "schema": schema_for!(TestContent),
                "version": "1.0.0"
            }))
            .await;
        Ok(())
    }
}
