use oauth2::{CsrfToken, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("parse error")]
    ParseError,
    #[error("unknown error")]
    Unknown,
}

pub trait AuthProvider {
    fn is_authenticated(&self) -> bool;
    fn is_loading(&self) -> bool;
    fn error(&self) -> Option<AuthError>;
    fn login(&self) -> Result<(), AuthError>;
    fn logout(&self) -> Result<(), AuthError>;
    fn user(&self) -> Option<User>;
    fn access_token(&self) -> Option<AccessToken>;
}

#[derive(Debug)]
pub struct ProviderConfig {
    pub client_id: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub return_to: Option<String>,
    pub csrf_token: Option<CsrfToken>,
    pub pkce_verifier: Option<PkceCodeVerifier>,
}

pub struct User {
    pub name: String,
}

pub struct CsrfTokenState(pub(crate) String);

impl CsrfTokenState {
    pub fn new(state: String) -> CsrfTokenState {
        CsrfTokenState(state)
    }
}

#[derive(Clone)]
pub struct AccessToken(String);

impl AccessToken {
    pub fn new(value: String) -> AccessToken {
        AccessToken(value)
    }

    pub fn secret(&self) -> &str {
        &self.0
    }
}
