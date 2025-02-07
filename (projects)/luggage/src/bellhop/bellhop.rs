use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

use semver::Version;
use serde::{Deserialize, Serialize};
use urn::Urn;

type LuggageSchema = String; // Currently just JSONSchema, later expand to TOML, YAML, etc...
#[derive(Serialize)]
struct Health {
    description: String,
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

pub fn router() -> Router {
    return Router::new()
        .route("/", get(health_check))
        .route("/v1/type", post(create_type));
}

pub async fn listener() -> tokio::net::TcpListener {
    return tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
}

async fn health_check() -> Json<Health> {
    Json(Health {
        description: "Belay On".into(),
    })
}

async fn create_type(Json(payload): Json<CreateType>) -> (StatusCode, Json<Type>) {
    let r#type = payload.into();

    // TODO: Save to backend

    return (StatusCode::CREATED, Json(r#type));
}

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use axum_test::TestServer;
    use convert_case::{Case, Casing};
    use serde_json::json;
    use urn::UrnBuilder;

    use super::*;

    #[tokio::test]
    async fn create_type() -> Result<()> {
        let test_name = "create_type";
        let server = TestServer::new(router()).unwrap(); //TODO: Remove unwrap (use ?)
        let response = server
            .post("/v1/type")
            .json(&json!({
                "urn": UrnBuilder::new(&test_name.to_case(Case::Kebab), "1234:5678").build()?,
                "schema": "TODO",
                "version": "1.0.0"
            }))
            .await;
        dbg!(response);
        assert!(false);
        Ok(())
    }
}
