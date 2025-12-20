use crate::error::Result;
use bellhop::bellhop::{app, listener};
use uuid::Uuid;

pub mod bellhop;
pub mod closet;
pub mod core;
pub mod cube;
pub mod error;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", Uuid::now_v7());
    axum::serve(listener().await, app(None).await?)
        .await
        .unwrap();
    return Ok(());
}
