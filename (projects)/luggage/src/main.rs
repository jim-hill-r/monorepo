use bellhop::bellhop::{listener, router};

pub mod bellhop;
pub mod closet;
pub mod cube;
pub mod error;

#[tokio::main]
async fn main() {
    axum::serve(listener().await, router()).await.unwrap();
}
