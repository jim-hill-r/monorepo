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
    /// Optional issuer URL for OIDC discovery.
    /// When provided, the provider can use OIDC discovery to automatically
    /// fetch endpoints from {issuer_url}/.well-known/openid-configuration
    /// This is preferred over explicit endpoint URLs for OIDC providers like Auth0.
    pub issuer_url: Option<String>,
}

/// Internal wrapper for CSRF token that doesn't expose oauth2 types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfTokenWrapper(pub(crate) String);

impl CsrfTokenWrapper {
    #[allow(dead_code)]
    pub(crate) fn new(value: String) -> Self {
        CsrfTokenWrapper(value)
    }
}

/// Internal wrapper for PKCE code verifier that doesn't expose oauth2 types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceVerifierWrapper(pub(crate) String);

impl PkceVerifierWrapper {
    #[allow(dead_code)]
    pub(crate) fn new(value: String) -> Self {
        PkceVerifierWrapper(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
    pub return_to: Option<String>,
    pub csrf_token: Option<CsrfTokenWrapper>,
    pub pkce_verifier: Option<PkceVerifierWrapper>,
}

pub struct User {
    pub name: String,
}

pub struct CsrfTokenState(
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] pub(crate) String,
);

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

    #[test]
    fn test_app_state_serialization() {
        let app_state = AppState {
            return_to: Some("https://example.com".to_string()),
            csrf_token: None,
            pkce_verifier: None,
        };
        let serialized = serde_json::to_string(&app_state).unwrap();
        let deserialized: AppState = serde_json::from_str(&serialized).unwrap();
        assert_eq!(app_state.return_to, deserialized.return_to);
    }

    #[test]
    fn test_csrf_token_state_new() {
        let state = CsrfTokenState::new("test_state_value".to_string());
        assert_eq!(state.0, "test_state_value");
    }

    #[test]
    fn test_access_token_secret() {
        let token = AccessToken::new("secret_token_value".to_string());
        assert_eq!(token.secret(), "secret_token_value");
    }

    #[test]
    fn test_access_token_can_be_cloned() {
        let token = AccessToken::new("test_token".to_string());
        let cloned = token.clone();
        assert_eq!(token.secret(), cloned.secret());
    }

    #[test]
    fn test_provider_config_with_oauth2_endpoints() {
        // Test backward compatibility: creating config with explicit OAuth2 endpoints
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.auth_url, "https://example.com/authorize");
        assert_eq!(config.token_url, "https://example.com/token");
        assert_eq!(config.redirect_url, "https://example.com/callback");
        assert!(config.issuer_url.is_none());
    }

    #[test]
    fn test_provider_config_with_issuer_url() {
        // Test new OIDC discovery: creating config with issuer URL
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: Some("https://example.com".to_string()),
        };
        assert_eq!(config.client_id, "test_client_id");
        assert_eq!(config.issuer_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_provider_config_debug() {
        // Verify ProviderConfig implements Debug
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: Some("https://example.com".to_string()),
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("test_client_id"));
        assert!(debug_str.contains("https://example.com"));
    }
}
