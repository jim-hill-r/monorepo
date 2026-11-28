//! The OIDC authorization code flow, shared by every platform.
//!
//! Nothing here touches a browser or an OS API, so it builds and is tested
//! natively. `web` and `native` only add how the authorize URL is opened and
//! how the callback comes back: the browser's address bar and session
//! storage for `web`, a loopback listener or a custom URL scheme for
//! `native`.
//!
//! Security checks live here once, so no platform can skip one:
//! PKCE, the `state` (CSRF) check, a required ID token whose nonce, issuer,
//! audience and signature `openidconnect` validates, and (off wasm32) an HTTP
//! client that never follows redirects.

use openidconnect::{
    AuthorizationCode, ClientId, CsrfToken, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenClaims, CoreProviderMetadata},
    reqwest,
    url::Url,
};

use crate::provider::{AccessToken, AuthError, ProviderConfig, User};

// The CoreClient with OIDC discovery endpoints configured. Type parameters
// are endpoint states: auth URL set, device auth, introspection and
// revocation not set, token URL set (needed by exchange_code), userinfo
// maybe set.
pub(crate) type OidcClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointMaybeSet,
>;

/// The scopes every sign-in asks for. `openid` is what makes it OIDC and
/// returns an ID token; `profile` and `email` fill [`User`].
pub(crate) const SCOPES: [&str; 6] = [
    "openid",
    "profile",
    "email",
    "read",
    "write",
    "offline_access",
];

/// The secrets one sign-in attempt needs to finish: the `state` sent to the
/// provider, the PKCE verifier and the nonce.
///
/// Keep it in memory (or, on the web, session storage) between starting a
/// sign-in and its callback, and use it once.
pub struct PendingLogin {
    pub(crate) csrf_token: String,
    pub(crate) pkce_verifier: String,
    pub(crate) nonce: String,
}

impl std::fmt::Debug for PendingLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PendingLogin { .. }")
    }
}

/// A finished sign-in: the access token and who signed in.
#[derive(Clone)]
pub struct SignedIn {
    pub access_token: AccessToken,
    pub user: User,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub(crate) claims: CoreIdTokenClaims,
}

impl std::fmt::Debug for SignedIn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignedIn")
            .field("user", &self.user)
            .finish_non_exhaustive()
    }
}

/// The HTTP client for discovery and the token exchange.
///
/// Off wasm32 it never follows redirects: RFC 6749 says token endpoints must
/// not redirect, and following one would let a hostile server point the
/// client at internal addresses (SSRF). On wasm32 reqwest uses the browser's
/// fetch, which has no redirect policy to set.
#[cfg_attr(
    target_arch = "wasm32",
    allow(
        clippy::unnecessary_wraps,
        reason = "only the native client can fail to build"
    )
)]
pub(crate) fn http_client() -> Result<reqwest::Client, AuthError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| {
                AuthError::HttpClientError(format!("failed to configure HTTP client: {e}"))
            })
    }
    #[cfg(target_arch = "wasm32")]
    {
        Ok(reqwest::Client::new())
    }
}

/// Fetch `{issuer_url}/.well-known/openid-configuration` and build a client
/// for `config`'s public client ID and redirect URI.
pub(crate) async fn discover(config: &ProviderConfig) -> Result<OidcClient, AuthError> {
    let issuer_url = config.issuer_url.clone().ok_or_else(|| {
        tracing::error!("issuer_url is required for OIDC discovery");
        AuthError::ConfigError("issuer_url is required".to_string())
    })?;
    tracing::info!("Using OIDC discovery with issuer: {issuer_url}");
    let issuer = IssuerUrl::new(issuer_url).map_err(|_| AuthError::ParseError)?;
    let redirect_url =
        RedirectUrl::new(config.redirect_url.clone()).map_err(|_| AuthError::ParseError)?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer, &http_client()?)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch OIDC discovery document: {e}");
            AuthError::OidcDiscoveryFailed(e.to_string())
        })?;

    let token_endpoint = provider_metadata
        .token_endpoint()
        .ok_or_else(|| {
            AuthError::OidcDiscoveryFailed(
                "Provider metadata missing required token_endpoint".to_string(),
            )
        })?
        .clone();

    // No client secret: these are public clients, protected by PKCE.
    Ok(CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(config.client_id.clone()),
        None,
    )
    .set_redirect_uri(redirect_url)
    .set_token_uri(token_endpoint))
}

/// Start a sign-in: the URL to send the user to, and what finishing it needs.
pub(crate) fn begin(client: &OidcClient) -> (Url, PendingLogin) {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );
    for scope in SCOPES {
        request = request.add_scope(Scope::new(scope.to_string()));
    }
    let (url, csrf_token, nonce) = request.set_pkce_challenge(pkce_challenge).url();
    (
        url,
        PendingLogin {
            csrf_token: csrf_token.secret().clone(),
            pkce_verifier: pkce_verifier.secret().clone(),
            nonce: nonce.secret().clone(),
        },
    )
}

/// Finish a sign-in: check `state`, exchange the code with the PKCE verifier,
/// and validate the ID token against the nonce.
pub(crate) async fn finish(
    client: &OidcClient,
    code: &str,
    state: &str,
    pending: PendingLogin,
) -> Result<SignedIn, AuthError> {
    check_state(&pending.csrf_token, state)?;

    let token_response = client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pending.pkce_verifier))
        .request_async(&http_client()?)
        .await
        .map_err(|e| {
            tracing::error!("Token exchange failed: {e}");
            AuthError::TokenExchangeError(e.to_string())
        })?;

    let id_token = require_id_token(token_response.id_token())?;
    let claims = id_token
        .claims(&client.id_token_verifier(), &Nonce::new(pending.nonce))
        .map_err(|e| {
            tracing::error!("ID token validation failed: {e}");
            AuthError::IdTokenValidationFailed(e.to_string())
        })?
        .clone();
    tracing::info!("Sign-in finished: ID token validated");

    Ok(SignedIn {
        access_token: AccessToken::new(token_response.access_token().secret().clone()),
        user: user_from_claims(&claims),
        claims,
    })
}

/// The `state` the provider sent back must be the one we sent.
pub(crate) fn check_state(expected: &str, returned: &str) -> Result<(), AuthError> {
    if expected.is_empty() || expected != returned {
        tracing::warn!("CSRF token validation failed - potential CSRF attack detected");
        return Err(AuthError::CsrfValidationFailed);
    }
    Ok(())
}

/// An OIDC token response must carry an ID token, since we always
/// ask for `openid`. A missing one means a misbehaving or spoofed token
/// endpoint, and signing in without it would skip the nonce check.
pub(crate) fn require_id_token<T>(id_token: Option<T>) -> Result<T, AuthError> {
    id_token.ok_or_else(|| {
        tracing::error!("No ID token in the token response");
        AuthError::IdTokenValidationFailed("no ID token in the token response".to_string())
    })
}

/// The standard claims of a validated ID token.
pub(crate) fn user_from_claims(claims: &CoreIdTokenClaims) -> User {
    User {
        sub: claims.subject().to_string(),
        name: claims
            .name()
            .and_then(|n| n.get(None).map(|v| v.as_str().to_string())),
        email: claims.email().map(|v| v.as_str().to_string()),
        email_verified: claims.email_verified(),
        picture: claims
            .picture()
            .and_then(|p| p.get(None).map(|v| v.as_str().to_string())),
        preferred_username: claims.preferred_username().map(|v| v.as_str().to_string()),
    }
}

/// The `code` and `state` of an authorization callback URL.
///
/// `redirect_url` is the redirect URI the sign-in was started with. The
/// callback must have the same scheme, host, port and path, so a URL meant
/// for another app or page is never treated as ours. An `error` parameter
/// (the user cancelled, or the provider refused) is returned as
/// [`AuthError::AuthorizationDenied`].
///
/// # Errors
///
/// [`AuthError::ParseError`] for a URL that doesn't parse,
/// [`AuthError::CallbackMismatch`] for a callback to another place,
/// [`AuthError::AuthorizationDenied`] for an `error` response, and
/// [`AuthError::MissingUrlParameter`] when `code` or `state` is missing.
pub fn parse_callback(
    redirect_url: &str,
    callback_url: &str,
) -> Result<(String, String), AuthError> {
    let expected = Url::parse(redirect_url).map_err(|_| AuthError::ParseError)?;
    let callback = Url::parse(callback_url).map_err(|_| AuthError::ParseError)?;
    if callback.scheme() != expected.scheme()
        || callback.host_str() != expected.host_str()
        || callback.port_or_known_default() != expected.port_or_known_default()
        || callback.path() != expected.path()
    {
        return Err(AuthError::CallbackMismatch(format!(
            "expected a callback to {}",
            expected.as_str().trim_end_matches('?')
        )));
    }

    let mut code = None;
    let mut state = None;
    let mut error = None;
    let mut error_description = None;
    for (name, value) in callback.query_pairs() {
        match name.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => error = Some(value.into_owned()),
            "error_description" => error_description = Some(value.into_owned()),
            _ => {}
        }
    }
    if let Some(error) = error {
        return Err(AuthError::AuthorizationDenied(match error_description {
            Some(description) => format!("{error}: {description}"),
            None => error,
        }));
    }
    Ok((
        code.ok_or_else(|| AuthError::MissingUrlParameter("code".to_string()))?,
        state.ok_or_else(|| AuthError::MissingUrlParameter("state".to_string()))?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REDIRECT: &str = "http://127.0.0.1:53682/callback";

    #[test]
    fn state_must_match() {
        assert!(check_state("abc", "abc").is_ok());
        assert!(matches!(
            check_state("abc", "abd"),
            Err(AuthError::CsrfValidationFailed)
        ));
    }

    #[test]
    fn an_empty_expected_state_never_matches() {
        assert!(matches!(
            check_state("", ""),
            Err(AuthError::CsrfValidationFailed)
        ));
    }

    #[test]
    fn a_missing_id_token_is_an_error() {
        let result = require_id_token::<()>(None);
        assert!(matches!(
            result,
            Err(AuthError::IdTokenValidationFailed(ref m)) if m == "no ID token in the token response"
        ));
        assert_eq!(require_id_token(Some(7)).ok(), Some(7));
    }

    #[test]
    fn callback_gives_code_and_state() {
        let parsed = parse_callback(REDIRECT, &format!("{REDIRECT}?code=c%2F1&state=s1"));
        assert_eq!(parsed.ok(), Some(("c/1".to_string(), "s1".to_string())));
    }

    #[test]
    fn callback_error_is_authorization_denied() {
        let result = parse_callback(
            REDIRECT,
            &format!("{REDIRECT}?error=access_denied&error_description=User%20cancelled&state=s"),
        );
        assert!(matches!(
            result,
            Err(AuthError::AuthorizationDenied(ref m)) if m == "access_denied: User cancelled"
        ));
    }

    #[test]
    fn callback_without_code_or_state_is_rejected() {
        assert!(matches!(
            parse_callback(REDIRECT, &format!("{REDIRECT}?state=s")),
            Err(AuthError::MissingUrlParameter(ref p)) if p == "code"
        ));
        assert!(matches!(
            parse_callback(REDIRECT, &format!("{REDIRECT}?code=c")),
            Err(AuthError::MissingUrlParameter(ref p)) if p == "state"
        ));
    }

    #[test]
    fn callback_to_another_place_is_rejected() {
        for other in [
            "http://127.0.0.1:53683/callback?code=c&state=s",
            "http://localhost:53682/callback?code=c&state=s",
            "https://127.0.0.1:53682/callback?code=c&state=s",
            "http://127.0.0.1:53682/other?code=c&state=s",
            "com.example.app://callback?code=c&state=s",
        ] {
            assert!(
                matches!(
                    parse_callback(REDIRECT, other),
                    Err(AuthError::CallbackMismatch(_))
                ),
                "{other} should be rejected"
            );
        }
    }

    #[test]
    fn custom_scheme_callbacks_work() {
        let redirect = "com.jimhillr.cahokia://auth/callback";
        let parsed = parse_callback(redirect, &format!("{redirect}?code=c&state=s"));
        assert_eq!(parsed.ok(), Some(("c".to_string(), "s".to_string())));
    }

    #[test]
    fn unparsable_urls_are_parse_errors() {
        assert!(matches!(
            parse_callback("not a url", REDIRECT),
            Err(AuthError::ParseError)
        ));
        assert!(matches!(
            parse_callback(REDIRECT, "not a url"),
            Err(AuthError::ParseError)
        ));
    }

    #[test]
    fn pending_login_debug_hides_its_secrets() {
        let pending = PendingLogin {
            csrf_token: "csrf-secret".to_string(),
            pkce_verifier: "pkce-secret".to_string(),
            nonce: "nonce-secret".to_string(),
        };
        let shown = format!("{pending:?}");
        assert!(!shown.contains("secret"), "{shown}");
    }
}
