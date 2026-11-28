pub mod auth0;
pub mod flow;
pub mod provider;
pub mod redirect;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod web;
