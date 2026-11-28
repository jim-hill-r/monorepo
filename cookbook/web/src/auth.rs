//! Sign-in with Auth0 in the header. Nothing on the site needs it yet.

use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::{AuthError, AuthProvider};
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::WebAuthProvider;

/// Auth0 tenant. `ProviderConfig::auth0` builds the issuer (with the trailing
/// slash OIDC discovery needs), authorize and token URLs from it.
#[cfg(any(target_arch = "wasm32", test))]
pub const AUTH0_DOMAIN: &str = "dev-jdadpn4pckxevrv5.us.auth0.com";
#[cfg(target_arch = "wasm32")]
pub const CLIENT_ID: &str = "savzmZnyHcvewGkQX8aaInwPFonC9k2x";

/// Format authentication errors into user-friendly messages
#[cfg(target_arch = "wasm32")]
fn format_auth_error(err: &AuthError) -> String {
    match err {
        AuthError::ParseError => "Unable to process authentication".to_string(),
        AuthError::TokenExchangeError(_) => "Login failed. Please try again".to_string(),
        AuthError::BrowserApiError(_) => "Browser compatibility issue".to_string(),
        AuthError::MissingUrlParameter(_) => "Authentication incomplete".to_string(),
        AuthError::SerializationError(_) => "Unable to process authentication".to_string(),
        AuthError::MissingStateData(_) => "Authentication incomplete".to_string(),
        AuthError::CsrfValidationFailed => "Security check failed. Please try again".to_string(),
        AuthError::OidcDiscoveryFailed(_) => "Authentication service unavailable".to_string(),
        AuthError::IdTokenValidationFailed(_) => "Login verification failed".to_string(),
        AuthError::HttpClientError(_) => "Network error. Please check your connection".to_string(),
        AuthError::ConfigError(_) => "Authentication not configured".to_string(),
        AuthError::AuthorizationDenied(_) => "Sign-in was cancelled".to_string(),
        AuthError::CallbackMismatch(_) => "Authentication incomplete".to_string(),
        AuthError::LoopbackError(_) => "Unable to process authentication".to_string(),
        AuthError::SignInTimedOut(_) => "Sign-in timed out. Please try again".to_string(),
        AuthError::Unknown => "An unexpected error occurred".to_string(),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn render_auth_section() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();

    match &*auth_state {
        Some(Ok(provider)) => {
            let provider = provider.clone();
            rsx! {
                button {
                    onclick: move |_| {
                        // Silently handle login errors - the auth provider handles redirects
                        let _ = provider.login();
                    },
                    "Login"
                }
            }
        }
        Some(Err(err)) => {
            let user_message = format_auth_error(err);
            rsx! {
                div {
                    class: "error",
                    "{user_message}"
                }
            }
        }
        None => rsx! {
            div { "Loading authentication..." }
        },
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render_auth_section() -> Element {
    rsx! { div { "Login (test mode)" } }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_format_auth_error_user_friendly() {
        use auth_sdk::provider::AuthError;

        // Test that all error messages are concise and user-friendly
        let test_cases = vec![
            (AuthError::ParseError, "Unable to process authentication"),
            (
                AuthError::TokenExchangeError("detailed error".to_string()),
                "Login failed. Please try again",
            ),
            (
                AuthError::BrowserApiError("browser error".to_string()),
                "Browser compatibility issue",
            ),
            (
                AuthError::MissingUrlParameter("code".to_string()),
                "Authentication incomplete",
            ),
            (
                AuthError::SerializationError("json error".to_string()),
                "Unable to process authentication",
            ),
            (
                AuthError::MissingStateData("state".to_string()),
                "Authentication incomplete",
            ),
            (
                AuthError::CsrfValidationFailed,
                "Security check failed. Please try again",
            ),
            (
                AuthError::OidcDiscoveryFailed("discovery error".to_string()),
                "Authentication service unavailable",
            ),
            (
                AuthError::IdTokenValidationFailed("validation error".to_string()),
                "Login verification failed",
            ),
            (
                AuthError::HttpClientError("network error".to_string()),
                "Network error. Please check your connection",
            ),
            (
                AuthError::ConfigError("config error".to_string()),
                "Authentication not configured",
            ),
            (
                AuthError::LoopbackError("port taken".to_string()),
                "Unable to process authentication",
            ),
            (
                AuthError::SignInTimedOut("no callback".to_string()),
                "Sign-in timed out. Please try again",
            ),
            (AuthError::Unknown, "An unexpected error occurred"),
        ];

        for (error, expected_message) in test_cases {
            let message = format_auth_error(&error);
            assert_eq!(
                message, expected_message,
                "Error message should be user-friendly"
            );

            // Verify messages are reasonably short (less than 100 characters)
            assert!(
                message.len() < 100,
                "Error message '{}' should be concise (less than 100 chars), got {} chars",
                message,
                message.len()
            );
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_format_auth_error_no_technical_details() {
        use auth_sdk::provider::AuthError;

        // Verify that formatted messages don't contain technical details from the original error
        let technical_error = AuthError::OidcDiscoveryFailed(
            "Failed to fetch https://example.com/.well-known/openid-configuration: NetworkError"
                .to_string(),
        );
        let message = format_auth_error(&technical_error);

        // Should not contain URLs or technical terms
        assert!(!message.contains("https://"), "Should not contain URLs");
        assert!(
            !message.contains(".well-known"),
            "Should not contain technical paths"
        );
        assert!(
            !message.contains("NetworkError"),
            "Should not contain technical error types"
        );

        // Should be a simple, user-friendly message
        assert_eq!(message, "Authentication service unavailable");
    }
}
