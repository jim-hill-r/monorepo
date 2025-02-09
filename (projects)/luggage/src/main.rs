use crate::error::Result;
use bellhop::bellhop::{listener, router};

pub mod bellhop;
pub mod closet;
pub mod cube;
pub mod error;

#[tokio::main]
async fn main() -> Result<()> {
    axum::serve(listener().await, router(None).await?)
        .await
        .unwrap();
    return Ok(());
}
