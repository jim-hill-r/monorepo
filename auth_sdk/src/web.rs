use openidconnect::{
    AuthorizationCode, ClientId, CsrfToken, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreProviderMetadata},
};
use web_sys::UrlSearchParams;

use crate::provider::{
    AccessToken, AppState, AuthError, AuthProvider, CsrfTokenState, CsrfTokenWrapper, NonceWrapper,
    PkceVerifierWrapper, ProviderConfig,
};

const DEFAULT_APP_STATE_STORAGE_KEY: &str = "auth_app_state";

#[derive(Clone)]
pub struct WebAuthProvider {
    client: CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
        EndpointMaybeSet,
    >,
    provider_metadata: Option<CoreProviderMetadata>,
    access_token: Option<AccessToken>,
    id_token_claims: Option<CoreIdTokenClaims>,
}

impl WebAuthProvider {
    pub async fn new(config: ProviderConfig) -> Result<WebAuthProvider, AuthError> {
        // OIDC discovery is required in openidconnect v4+
        let issuer_url = config.issuer_url.clone().ok_or_else(|| {
            tracing::error!("issuer_url is required for OIDC discovery");
            AuthError::ConfigError("issuer_url is required".to_string())
        })?;

        tracing::info!("Using OIDC discovery with issuer: {}", issuer_url);
        let (client, provider_metadata) =
            Self::create_client_from_discovery(issuer_url, &config).await?;

        let (access_token, id_token_claims) =
            if let Ok((authorization_code, state)) = fetch_code_and_state_from_browser() {
                let (token, claims) = handle_redirect(&client, authorization_code, state).await?;
                (Some(token), claims)
            } else {
                (None, None)
            };

        Ok(WebAuthProvider {
            client,
            provider_metadata,
            access_token,
            id_token_claims,
        })
    }

    /// Create a CoreClient using OIDC discovery
    async fn create_client_from_discovery(
        issuer_url: String,
        config: &ProviderConfig,
    ) -> Result<
        (
            CoreClient<
                EndpointSet,
                EndpointNotSet,
                EndpointNotSet,
                EndpointNotSet,
                EndpointSet,
                EndpointMaybeSet,
            >,
            Option<CoreProviderMetadata>,
        ),
        AuthError,
    > {
        tracing::debug!(
            "Fetching OIDC discovery document from: {}/.well-known/openid-configuration",
            issuer_url
        );

        let issuer = IssuerUrl::new(issuer_url).map_err(|_| AuthError::ParseError)?;

        // Create HTTP client for discovery
        let http_client = reqwest::Client::new();

        // Fetch provider metadata from discovery document
        let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch OIDC discovery document: {}", e);
                AuthError::OidcDiscoveryFailed(e.to_string())
            })?;

        tracing::debug!("Successfully fetched OIDC discovery document");

        // Extract the token endpoint from provider metadata - it's required for token exchange
        let token_endpoint = provider_metadata
            .token_endpoint()
            .ok_or_else(|| {
                AuthError::OidcDiscoveryFailed(
                    "Provider metadata missing required token_endpoint".to_string(),
                )
            })?
            .clone();

        // Create client from discovered metadata
        let client = CoreClient::from_provider_metadata(
            provider_metadata.clone(),
            ClientId::new(config.client_id.clone()),
            None, // No client secret for public clients (PKCE is used instead)
        )
        // Set redirect URI for the OAuth2 flow
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_url.clone()).map_err(|_| AuthError::ParseError)?,
        )
        // Explicitly set token endpoint to upgrade type state from EndpointMaybeSet to EndpointSet
        // This is required for methods like exchange_code() to be available
        .set_token_uri(token_endpoint);

        tracing::info!("OIDC client created successfully from discovery");
        Ok((client, Some(provider_metadata)))
    }
}

impl AuthProvider for WebAuthProvider {
    fn is_authenticated(&self) -> bool {
        todo!()
    }

    fn is_loading(&self) -> bool {
        todo!()
    }

    fn error(&self) -> Option<crate::provider::AuthError> {
        todo!()
    }

    fn login(&self) -> Result<(), AuthError> {
        tracing::info!("Initiating OIDC login flow with nonce validation");

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        tracing::debug!("Generated PKCE challenge and verifier for login");

        // Generate the full authorization URL with nonce for OIDC.
        let (auth_url, csrf_token, nonce) = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            // Set the desired scopes.
            // Note: "openid" scope is required for OIDC and to get ID token with user claims.
            // "profile" and "email" scopes request additional standard claims.
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            // Offline access results in refresh token being provided
            .add_scope(Scope::new("offline_access".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        tracing::debug!("Generated CSRF token and nonce for login");

        store_app_state_in_browser(&AppState {
            return_to: fetch_current_location_from_browser(),
            csrf_token: Some(CsrfTokenWrapper::new(csrf_token.secret().to_string())),
            pkce_verifier: Some(PkceVerifierWrapper::new(pkce_verifier.secret().to_string())),
            nonce: Some(NonceWrapper::new(nonce.secret().to_string())),
        })?;

        tracing::debug!("Stored authentication state with nonce in browser session storage");

        redirect_browser(auth_url.as_ref())
    }

    fn logout(&self) -> Result<(), AuthError> {
        todo!()
    }

    fn user(&self) -> Option<crate::provider::User> {
        // Extract user information from ID token claims
        self.id_token_claims.as_ref().map(|claims| {
            tracing::debug!("Extracting user information from ID token claims");

            crate::provider::User {
                sub: claims.subject().to_string(),
                name: claims
                    .name()
                    .and_then(|n| n.get(None).map(|localized| localized.to_string())),
                email: claims.email().map(|e| e.to_string()),
                email_verified: claims.email_verified(),
                picture: claims
                    .picture()
                    .and_then(|p| p.get(None).map(|localized| localized.to_string())),
                preferred_username: claims.preferred_username().map(|u| u.to_string()),
            }
        })
    }

    fn access_token(&self) -> Option<crate::provider::AccessToken> {
        self.access_token.clone()
    }
}

pub fn fetch_current_location_from_browser() -> Option<String> {
    if let Some(window) = web_sys::window() {
        return window.location().href().ok();
    }
    None
}
fn fetch_code_and_state_from_browser() -> Result<(AuthorizationCode, CsrfTokenState), AuthError> {
    let window = web_sys::window()
        .ok_or_else(|| AuthError::BrowserApiError("window object not available".to_string()))?;
    let search = window
        .location()
        .search()
        .map_err(|_| AuthError::BrowserApiError("failed to get location search".to_string()))?;
    let params = UrlSearchParams::new_with_str(&search)
        .map_err(|_| AuthError::BrowserApiError("failed to parse URL search params".to_string()))?;
    let code = params
        .get("code")
        .ok_or_else(|| AuthError::MissingUrlParameter("code".to_string()))?;
    let state = params
        .get("state")
        .ok_or_else(|| AuthError::MissingUrlParameter("state".to_string()))?;
    Ok((AuthorizationCode::new(code), CsrfTokenState::new(state)))
}

fn redirect_browser(url: &str) -> Result<(), AuthError> {
    let window = web_sys::window()
        .ok_or_else(|| AuthError::BrowserApiError("window object not available".to_string()))?;
    window
        .open_with_url_and_target_and_features(url, "_self", "")
        .map(|_| ())
        .map_err(|_| AuthError::BrowserApiError("failed to redirect browser".to_string()))
}

fn store_app_state_in_browser(app_state: &AppState) -> Result<(), AuthError> {
    let window = web_sys::window()
        .ok_or_else(|| AuthError::BrowserApiError("window object not available".to_string()))?;
    let storage = window
        .session_storage()
        .map_err(|_| AuthError::BrowserApiError("failed to access session storage".to_string()))?
        .ok_or_else(|| AuthError::BrowserApiError("session storage not available".to_string()))?;
    let json = serde_json::to_string(app_state).map_err(|e| {
        AuthError::SerializationError(format!("failed to serialize app state: {}", e))
    })?;
    storage
        .set_item(DEFAULT_APP_STATE_STORAGE_KEY, &json)
        .map_err(|_| {
            AuthError::BrowserApiError("failed to store app state in session storage".to_string())
        })
}

fn fetch_app_state_from_browser() -> Result<AppState, AuthError> {
    let window = web_sys::window()
        .ok_or_else(|| AuthError::BrowserApiError("window object not available".to_string()))?;
    let storage = window
        .session_storage()
        .map_err(|_| AuthError::BrowserApiError("failed to access session storage".to_string()))?
        .ok_or_else(|| AuthError::BrowserApiError("session storage not available".to_string()))?;
    let item = storage
        .get_item(DEFAULT_APP_STATE_STORAGE_KEY)
        .map_err(|_| {
            AuthError::BrowserApiError(
                "failed to retrieve app state from session storage".to_string(),
            )
        })?;
    match item {
        Some(json) => serde_json::from_str(&json).map_err(|e| {
            AuthError::SerializationError(format!("failed to deserialize app state: {}", e))
        }),
        _ => Err(AuthError::MissingStateData(
            "app state not found in session storage".to_string(),
        )),
    }
}

async fn handle_redirect(
    client: &CoreClient<
        EndpointSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointNotSet,
        EndpointSet,
        EndpointMaybeSet,
    >,
    authorization_code: AuthorizationCode,
    state: CsrfTokenState,
) -> Result<(AccessToken, Option<CoreIdTokenClaims>), AuthError> {
    tracing::info!("Processing OIDC redirect callback");

    let app_state = fetch_app_state_from_browser()?;
    tracing::debug!("Retrieved authentication state from browser session storage");

    let csrf_token_wrapper = app_state
        .csrf_token
        .ok_or_else(|| AuthError::MissingStateData("csrf_token".to_string()))?;
    let pkce_verifier_wrapper = app_state
        .pkce_verifier
        .ok_or_else(|| AuthError::MissingStateData("pkce_verifier".to_string()))?;
    let nonce_wrapper = app_state
        .nonce
        .ok_or_else(|| AuthError::MissingStateData("nonce".to_string()))?;

    // Security validations:
    // 1. CSRF token validation (state parameter) - protects against CSRF attacks
    // 2. PKCE verification - protects against authorization code interception
    // 3. Nonce validation - protects against replay attacks (validated below)
    // 4. No redirect following in HTTP client - prevents SSRF attacks
    if state.0 != csrf_token_wrapper.0 {
        tracing::warn!("CSRF token validation failed - potential CSRF attack detected");
        return Err(AuthError::CsrfValidationFailed);
    }

    tracing::debug!("CSRF token validation successful");

    // Following redirects opens the client up to SSRF vulnerabilities.
    // OAuth2 RFC 6749 requires that token endpoints MUST NOT redirect.
    // This policy prevents SSRF attacks where a malicious authorization server
    // could redirect the client to internal resources.
    //
    // Note: On wasm32, reqwest uses the browser's fetch API which has built-in
    // redirect handling and doesn't support custom redirect policies. The browser's
    // same-origin policy provides some protection, but developers should ensure
    // their OAuth2 server endpoints follow the RFC 6749 requirement of not redirecting.
    #[cfg(not(target_arch = "wasm32"))]
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| {
            AuthError::HttpClientError(format!("failed to configure HTTP client: {}", e))
        })?;

    #[cfg(target_arch = "wasm32")]
    let http_client = reqwest::Client::new();

    // Convert wrapper type back to oauth2 type for token exchange
    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_wrapper.0);
    tracing::debug!("Prepared PKCE verifier for token exchange");

    // Now you can exchange it for an access token and ID token.
    tracing::debug!("Initiating token exchange with authorization server");
    let token_result = client
        .exchange_code(authorization_code)
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| {
            tracing::error!("Token exchange failed: {}", e);
            AuthError::TokenExchangeError(e.to_string())
        })?;

    tracing::info!("Token exchange successful - access token and ID token obtained");

    // Validate the ID token with nonce verification and extract claims
    let id_token_claims = if let Some(id_token) = token_result.id_token() {
        tracing::debug!("Validating ID token with nonce");

        // Convert wrapper back to openidconnect Nonce type
        let nonce = Nonce::new(nonce_wrapper.0);

        // Create ID token verifier and validate the token
        let id_token_verifier = client.id_token_verifier();
        let claims = id_token.claims(&id_token_verifier, &nonce).map_err(|e| {
            tracing::error!("ID token validation failed: {}", e);
            AuthError::IdTokenValidationFailed(e.to_string())
        })?;

        tracing::info!("ID token validation successful - nonce verified, claims extracted");
        Some(claims.clone())
    } else {
        tracing::warn!("No ID token returned by server - nonce validation skipped");
        None
    };

    // Return both the access token and the ID token claims
    Ok((
        AccessToken::new(token_result.access_token().clone().into_secret()),
        id_token_claims,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client_from_explicit_endpoints() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };

        let result = WebAuthProvider::create_client_from_explicit_endpoints(&config);
        assert!(result.is_ok());

        let (client, metadata) = result.unwrap();
        assert!(
            metadata.is_none(),
            "Metadata should be None when using explicit endpoints"
        );

        // Verify client was created (we can't easily test the internal state, but we can verify it doesn't panic)
        drop(client);
    }

    #[test]
    fn test_create_client_from_explicit_endpoints_with_invalid_auth_url() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "not a valid url".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };

        let result = WebAuthProvider::create_client_from_explicit_endpoints(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ParseError));
    }

    #[test]
    fn test_create_client_from_explicit_endpoints_with_invalid_token_url() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "not a valid url".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };

        let result = WebAuthProvider::create_client_from_explicit_endpoints(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ParseError));
    }

    #[test]
    fn test_create_client_from_explicit_endpoints_with_invalid_redirect_url() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "not a valid url".to_string(),
            issuer_url: None,
        };

        let result = WebAuthProvider::create_client_from_explicit_endpoints(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ParseError));
    }

    // Note: Testing create_client_from_discovery requires mocking HTTP requests,
    // which is complex in wasm32 environment. The discovery functionality will be
    // tested through integration tests or manual testing with a real OIDC provider.
    // For unit tests, we verify that:
    // 1. Invalid issuer URLs are rejected with ParseError
    #[tokio::test]
    async fn test_create_client_from_discovery_with_invalid_issuer_url() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };

        let result =
            WebAuthProvider::create_client_from_discovery("not a valid url".to_string(), &config)
                .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::ParseError));
    }

    #[test]
    fn test_nonce_can_be_created() {
        // Test that we can create a Nonce for OIDC flows
        let nonce = Nonce::new_random();
        assert!(!nonce.secret().is_empty());
    }

    #[test]
    fn test_nonce_wrapper_conversion() {
        // Test that we can convert between Nonce and NonceWrapper
        let nonce = Nonce::new("test_nonce_value".to_string());
        let wrapper = NonceWrapper::new(nonce.secret().to_string());
        assert_eq!(wrapper.0, "test_nonce_value");

        // And convert back
        let nonce_from_wrapper = Nonce::new(wrapper.0);
        assert_eq!(nonce_from_wrapper.secret(), "test_nonce_value");
    }

    #[test]
    fn test_core_authentication_flow_available() {
        // Test that CoreAuthenticationFlow is available for use in authorize_url
        // This is a compile-time check that the type is imported correctly
        let _flow = CoreAuthenticationFlow::AuthorizationCode;
    }
}
