pub mod auth0;
pub mod provider;
pub mod redirect;

#[cfg(target_arch = "wasm32")]
pub mod web;
