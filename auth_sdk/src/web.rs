use openidconnect::core::CoreIdTokenClaims;
use web_sys::UrlSearchParams;

use crate::flow::{self, OidcClient, PendingLogin};
use crate::provider::{
    AccessToken, AppState, AuthError, AuthProvider, CsrfTokenState, CsrfTokenWrapper, NonceWrapper,
    PkceVerifierWrapper, ProviderConfig,
};
use crate::redirect::{safe_return_path, strip_authorization_response};

const DEFAULT_APP_STATE_STORAGE_KEY: &str = "auth_app_state";

#[derive(Clone)]
pub struct WebAuthProvider {
    client: OidcClient,
    access_token: Option<AccessToken>,
    id_token_claims: Option<CoreIdTokenClaims>,
    return_to: Option<String>,
}

impl WebAuthProvider {
    /// Discover the provider and, when this page load is a sign-in callback,
    /// finish that sign-in.
    ///
    /// # Errors
    ///
    /// Any error of OIDC discovery, or of finishing the sign-in when the page
    /// URL carries an authorization response.
    pub async fn new(config: ProviderConfig) -> Result<WebAuthProvider, AuthError> {
        let client = flow::discover(&config).await?;

        let (access_token, id_token_claims, return_to) =
            if let Ok((authorization_code, state)) = fetch_code_and_state_from_browser() {
                let result = handle_redirect(&client, authorization_code, state).await;
                // The code is single-use: take it (and the state) out of the
                // address bar and drop the stored login state whether or not
                // the exchange worked, so a reload doesn't replay it.
                clear_authorization_response_from_browser();
                let app_state = fetch_app_state_from_browser().ok();
                remove_app_state_from_browser();
                let (token, claims) = result?;
                let return_to =
                    safe_return_path(app_state.as_ref().and_then(|s| s.return_to.as_deref()));
                (Some(token), claims, Some(return_to))
            } else {
                (None, None, None)
            };

        Ok(WebAuthProvider {
            client,
            access_token,
            id_token_claims,
            return_to,
        })
    }

    /// The same-origin path the user was on when they clicked Login.
    ///
    /// `Some` only on the page load that completed a sign-in redirect. The
    /// redirect URI is always the site root, so the app navigates here to
    /// put the user back where they started.
    #[must_use]
    pub fn return_to(&self) -> Option<&str> {
        self.return_to.as_deref()
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

        let (auth_url, pending) = flow::begin(&self.client);
        tracing::debug!("Generated PKCE verifier, CSRF token and nonce for login");

        store_app_state_in_browser(&AppState {
            return_to: fetch_current_path_from_browser(),
            csrf_token: Some(CsrfTokenWrapper::new(pending.csrf_token)),
            pkce_verifier: Some(PkceVerifierWrapper::new(pending.pkce_verifier)),
            nonce: Some(NonceWrapper::new(pending.nonce)),
        })?;

        tracing::debug!("Stored authentication state with nonce in browser session storage");

        redirect_browser(auth_url.as_ref())
    }

    fn logout(&self) -> Result<(), AuthError> {
        todo!()
    }

    fn user(&self) -> Option<crate::provider::User> {
        self.id_token_claims.as_ref().map(flow::user_from_claims)
    }

    fn access_token(&self) -> Option<crate::provider::AccessToken> {
        self.access_token.clone()
    }
}

#[must_use]
pub fn fetch_current_location_from_browser() -> Option<String> {
    if let Some(window) = web_sys::window() {
        return window.location().href().ok();
    }
    None
}
/// The site origin, such as `https://jimhillr-cahokia.pages.dev`.
///
/// Pass it to [`crate::redirect::redirect_url_for_origin`] for a stable
/// `redirect_url`.
#[must_use]
pub fn fetch_origin_from_browser() -> Option<String> {
    web_sys::window()?.location().origin().ok()
}

/// The current page as a same-origin path: pathname, query and hash.
fn fetch_current_path_from_browser() -> Option<String> {
    let location = web_sys::window()?.location();
    Some(format!(
        "{}{}{}",
        location.pathname().ok()?,
        location.search().ok()?,
        location.hash().ok()?
    ))
}

/// Replace the current history entry with the same URL minus the
/// authorization response (`code`, `state`, `error`, `error_description`).
fn clear_authorization_response_from_browser() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let location = window.location();
    let (Ok(pathname), Ok(search), Ok(hash)) =
        (location.pathname(), location.search(), location.hash())
    else {
        return;
    };
    let url = format!("{pathname}{}{hash}", strip_authorization_response(&search));
    if let Ok(history) = window.history()
        && history
            .replace_state_with_url(&web_sys::wasm_bindgen::JsValue::NULL, "", Some(&url))
            .is_err()
    {
        tracing::warn!("failed to clear the authorization response from the URL");
    }
}

fn remove_app_state_from_browser() {
    if let Some(Ok(Some(storage))) = web_sys::window().map(|w| w.session_storage()) {
        let _ = storage.remove_item(DEFAULT_APP_STATE_STORAGE_KEY);
    }
}

fn fetch_code_and_state_from_browser() -> Result<(String, CsrfTokenState), AuthError> {
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
    Ok((code, CsrfTokenState::new(state)))
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
        AuthError::SerializationError(format!("failed to serialize app state: {e}"))
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
            AuthError::SerializationError(format!("failed to deserialize app state: {e}"))
        }),
        _ => Err(AuthError::MissingStateData(
            "app state not found in session storage".to_string(),
        )),
    }
}

async fn handle_redirect(
    client: &OidcClient,
    authorization_code: String,
    state: CsrfTokenState,
) -> Result<(AccessToken, Option<CoreIdTokenClaims>), AuthError> {
    tracing::info!("Processing OIDC redirect callback");

    let app_state = fetch_app_state_from_browser()?;
    let pending = PendingLogin {
        csrf_token: app_state
            .csrf_token
            .ok_or_else(|| AuthError::MissingStateData("csrf_token".to_string()))?
            .0,
        pkce_verifier: app_state
            .pkce_verifier
            .ok_or_else(|| AuthError::MissingStateData("pkce_verifier".to_string()))?
            .0,
        nonce: app_state
            .nonce
            .ok_or_else(|| AuthError::MissingStateData("nonce".to_string()))?
            .0,
    };

    // flow::finish checks the state, sends the PKCE verifier, requires an
    // ID token and validates its nonce.
    let signed_in = flow::finish(client, &authorization_code, &state.0, pending).await?;
    Ok((signed_in.access_token, Some(signed_in.claims)))
}

#[cfg(test)]
mod tests {
    use super::*;

    // The flow itself is tested natively in `flow` and `native`; this only
    // checks that the web provider rejects a config with no issuer.
    #[tokio::test]
    async fn new_needs_an_issuer_url() {
        let config = ProviderConfig {
            client_id: "test_client_id".to_string(),
            auth_url: "https://example.com/authorize".to_string(),
            token_url: "https://example.com/token".to_string(),
            redirect_url: "https://example.com/callback".to_string(),
            issuer_url: None,
        };
        assert!(matches!(
            WebAuthProvider::new(config).await,
            Err(AuthError::ConfigError(_))
        ));
    }
}
