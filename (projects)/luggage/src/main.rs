pub mod bellhop;
pub mod closet;
pub mod cube;
pub mod error;

use bellhop::bellhop::start;

#[tokio::main]
async fn main() {
    let (listener, app) = start().await;
    // TODO: Figure out how to get the serve function inside of the library
    axum::serve(listener, app).await.unwrap();
}
