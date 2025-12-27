use oauth2::{CsrfToken, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AuthError {
    #[error("parse error")]
    ParseError,
    #[error("token exchange error: {0}")]
    TokenExchangeError(String),
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

    /// Returns the secret access token string.
    pub fn secret(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_parse_error() {
        let error = AuthError::ParseError;
        assert_eq!(error.to_string(), "parse error");
    }

    #[test]
    fn test_auth_error_token_exchange_error_preserves_message() {
        let error_msg = "invalid_grant: The provided authorization code is invalid";
        let error = AuthError::TokenExchangeError(error_msg.to_string());
        assert_eq!(
            error.to_string(),
            format!("token exchange error: {}", error_msg)
        );
    }

    #[test]
    fn test_auth_error_unknown() {
        let error = AuthError::Unknown;
        assert_eq!(error.to_string(), "unknown error");
    }

    #[test]
    fn test_auth_error_can_be_cloned() {
        let error = AuthError::TokenExchangeError("test error".to_string());
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }
}
