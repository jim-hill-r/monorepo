//! Sign-in for native apps (desktop and mobile), following RFC 8252: the
//! app opens the provider's authorize URL in the system browser and gets the
//! callback back through a redirect URI it owns.
//!
//! This module is the platform-free half: start a sign-in, then finish it
//! from the callback URL. How the URL is opened and how the callback comes
//! back (a loopback listener on desktop, a custom URL scheme on mobile) is
//! the app's or a later module's job; see
//! `docs/plans/2026-11-native-sign-in.md`.
//!
//! ```no_run
//! # async fn example() -> Result<(), auth_sdk::provider::AuthError> {
//! use auth_sdk::native::NativeAuthClient;
//! use auth_sdk::provider::ProviderConfig;
//!
//! let client = NativeAuthClient::new(ProviderConfig {
//!     client_id: "public-client-id".to_string(),
//!     auth_url: String::new(),
//!     token_url: String::new(),
//!     redirect_url: "http://127.0.0.1:53682/callback".to_string(),
//!     issuer_url: Some("https://example.us.auth0.com/".to_string()),
//! })
//! .await?;
//! let request = client.begin_login();
//! // Open request.authorize_url in the system browser, then wait for the
//! // callback URL the browser is sent to.
//! # let callback_url = String::new();
//! let signed_in = client.complete_login(request.pending, &callback_url).await?;
//! println!("signed in as {}", signed_in.user.sub);
//! # Ok(())
//! # }
//! ```

use crate::flow::{self, OidcClient};
pub use crate::flow::{PendingLogin, SignedIn, parse_callback};
use crate::provider::{AuthError, ProviderConfig};

/// An OIDC client for a native app's public client ID.
#[derive(Clone)]
pub struct NativeAuthClient {
    client: OidcClient,
    redirect_url: String,
}

/// A started sign-in.
#[derive(Debug)]
pub struct LoginRequest {
    /// Open this in the system browser (never an embedded web view, which
    /// RFC 8252 forbids because the app could read the user's password).
    pub authorize_url: String,
    /// Pass this to [`NativeAuthClient::complete_login`] with the callback.
    pub pending: PendingLogin,
}

impl NativeAuthClient {
    /// Discover the provider's endpoints from `config.issuer_url`.
    ///
    /// `config.redirect_url` is the app's own redirect URI, such as
    /// `http://127.0.0.1:<port>/callback` or a custom scheme, and must be an
    /// Allowed Callback URL of the provider's native application.
    ///
    /// # Errors
    ///
    /// [`AuthError::ConfigError`] without an issuer URL,
    /// [`AuthError::ParseError`] for an issuer or redirect URL that doesn't
    /// parse, and [`AuthError::OidcDiscoveryFailed`] when discovery fails or
    /// the published issuer doesn't match exactly.
    pub async fn new(config: ProviderConfig) -> Result<Self, AuthError> {
        let client = flow::discover(&config).await?;
        Ok(Self {
            client,
            redirect_url: config.redirect_url,
        })
    }

    /// The redirect URI this client signs in with.
    #[must_use]
    pub fn redirect_url(&self) -> &str {
        &self.redirect_url
    }

    /// Start a sign-in with fresh PKCE, `state` and nonce values.
    #[must_use]
    pub fn begin_login(&self) -> LoginRequest {
        let (url, pending) = flow::begin(&self.client);
        LoginRequest {
            authorize_url: url.to_string(),
            pending,
        }
    }

    /// Finish a sign-in from the URL the provider redirected the browser to.
    ///
    /// Fails if the URL isn't this client's redirect URI, carries an
    /// `error`, has the wrong `state`, or the token response has no valid ID
    /// token for this sign-in's nonce.
    ///
    /// # Errors
    ///
    /// Any error of [`parse_callback`], [`AuthError::CsrfValidationFailed`],
    /// [`AuthError::TokenExchangeError`], [`AuthError::HttpClientError`] or
    /// [`AuthError::IdTokenValidationFailed`].
    pub async fn complete_login(
        &self,
        pending: PendingLogin,
        callback_url: &str,
    ) -> Result<SignedIn, AuthError> {
        let (code, state) = parse_callback(&self.redirect_url, callback_url)?;
        flow::finish(&self.client, &code, &state, pending).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// A fake OIDC provider on 127.0.0.1: discovery, an empty key set, and
    /// a token endpoint that answers with `token_body`.
    async fn fake_provider(token_status: u16, token_body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let base = format!(
            "http://{}",
            listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
        );
        let issuer = format!("{base}/");
        let discovery = serde_json::json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{base}/authorize"),
            "token_endpoint": format!("{base}/oauth/token"),
            "jwks_uri": format!("{base}/.well-known/jwks.json"),
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
        })
        .to_string();
        let routes: HashMap<&'static str, (u16, String)> = HashMap::from([
            ("/.well-known/openid-configuration", (200, discovery)),
            (
                "/.well-known/jwks.json",
                (200, r#"{"keys":[]}"#.to_string()),
            ),
            ("/oauth/token", (token_status, token_body.to_string())),
        ]);
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0; 16 * 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let (status, body) = routes.get(path).cloned().unwrap_or((404, "{}".to_string()));
                let response = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        issuer
    }

    fn config(issuer: String) -> ProviderConfig {
        ProviderConfig {
            client_id: "native-client".to_string(),
            auth_url: String::new(),
            token_url: String::new(),
            redirect_url: "http://127.0.0.1:53682/callback".to_string(),
            issuer_url: Some(issuer),
        }
    }

    async fn client(token_status: u16, token_body: &'static str) -> NativeAuthClient {
        let issuer = fake_provider(token_status, token_body).await;
        NativeAuthClient::new(config(issuer))
            .await
            .unwrap_or_else(|e| panic!("discovery failed: {e}"))
    }

    fn query(url: &str) -> HashMap<String, String> {
        openidconnect::url::Url::parse(url)
            .unwrap_or_else(|e| panic!("{e}"))
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    #[tokio::test]
    async fn authorize_url_carries_pkce_state_nonce_and_redirect() {
        let client = client(200, "{}").await;
        let request = client.begin_login();
        let params = query(&request.authorize_url);
        assert!(request.authorize_url.contains("/authorize?"));
        assert_eq!(params["client_id"], "native-client");
        assert_eq!(params["redirect_uri"], "http://127.0.0.1:53682/callback");
        assert_eq!(params["response_type"], "code");
        assert_eq!(params["code_challenge_method"], "S256");
        assert_eq!(params["state"], request.pending.csrf_token);
        assert_eq!(params["nonce"], request.pending.nonce);
        assert!(params["scope"].split(' ').any(|s| s == "openid"));
        assert!(!params.contains_key("code_verifier"));
    }

    #[tokio::test]
    async fn every_sign_in_gets_fresh_secrets() {
        let client = client(200, "{}").await;
        let first = client.begin_login().pending;
        let second = client.begin_login().pending;
        assert_ne!(first.csrf_token, second.csrf_token);
        assert_ne!(first.pkce_verifier, second.pkce_verifier);
        assert_ne!(first.nonce, second.nonce);
    }

    #[tokio::test]
    async fn wrong_state_is_rejected_before_the_token_exchange() {
        let client = client(500, "{}").await;
        let request = client.begin_login();
        let result = client
            .complete_login(
                request.pending,
                "http://127.0.0.1:53682/callback?code=c&state=forged",
            )
            .await;
        assert!(matches!(result, Err(AuthError::CsrfValidationFailed)));
    }

    #[tokio::test]
    async fn token_response_without_an_id_token_is_rejected() {
        let client = client(
            200,
            r#"{"access_token":"at","token_type":"Bearer","expires_in":60}"#,
        )
        .await;
        let request = client.begin_login();
        let callback = format!(
            "http://127.0.0.1:53682/callback?code=c&state={}",
            request.pending.csrf_token
        );
        let result = client.complete_login(request.pending, &callback).await;
        assert!(matches!(
            result,
            Err(AuthError::IdTokenValidationFailed(ref m)) if m == "no ID token in the token response"
        ));
    }

    #[tokio::test]
    async fn token_endpoint_error_is_a_token_exchange_error() {
        let client = client(400, r#"{"error":"invalid_grant"}"#).await;
        let request = client.begin_login();
        let callback = format!(
            "http://127.0.0.1:53682/callback?code=c&state={}",
            request.pending.csrf_token
        );
        let result = client.complete_login(request.pending, &callback).await;
        assert!(
            matches!(result, Err(AuthError::TokenExchangeError(ref m)) if m.contains("invalid_grant")),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn provider_error_in_the_callback_is_reported() {
        let client = client(200, "{}").await;
        let request = client.begin_login();
        let result = client
            .complete_login(
                request.pending,
                "http://127.0.0.1:53682/callback?error=access_denied",
            )
            .await;
        assert!(
            matches!(result, Err(AuthError::AuthorizationDenied(ref m)) if m == "access_denied")
        );
    }

    #[tokio::test]
    async fn discovery_needs_an_issuer() {
        let mut config = config(String::new());
        config.issuer_url = None;
        let result = NativeAuthClient::new(config).await;
        assert!(matches!(result, Err(AuthError::ConfigError(_))));
    }

    #[tokio::test]
    async fn discovery_rejects_an_issuer_that_does_not_match() {
        let issuer = fake_provider(200, "{}").await;
        // Without the trailing slash the published issuer doesn't match.
        let result = NativeAuthClient::new(config(issuer.trim_end_matches('/').to_string())).await;
        assert!(matches!(result, Err(AuthError::OidcDiscoveryFailed(_))));
    }
}
