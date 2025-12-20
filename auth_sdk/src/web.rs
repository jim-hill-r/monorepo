use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, EndpointNotSet, EndpointSet,
    PkceCodeChallenge, RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use web_sys::UrlSearchParams;

use crate::provider::{
    AccessToken, AppState, AuthError, AuthProvider, CsrfTokenState, ProviderConfig,
};

const DEFAULT_APP_STATE_STORAGE_KEY: &str = "auth_app_state";

#[derive(Clone)]
pub struct WebAuthProvider {
    client: BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    access_token: Option<AccessToken>,
}

impl WebAuthProvider {
    pub async fn new(config: ProviderConfig) -> Result<WebAuthProvider, AuthError> {
        let client = BasicClient::new(ClientId::new(config.client_id))
            .set_auth_uri(AuthUrl::new(config.auth_url).map_err(|_| AuthError::ParseError)?)
            .set_token_uri(TokenUrl::new(config.token_url).map_err(|_| AuthError::ParseError)?)
            .set_redirect_uri(
                RedirectUrl::new(config.redirect_url).map_err(|_| AuthError::ParseError)?,
            );
        let access_token =
            if let Ok((authorization_code, state)) = fetch_code_and_state_from_browser() {
                Some(handle_redirect(&client, authorization_code, state).await?)
            } else {
                None
            };

        Ok(WebAuthProvider {
            client,
            access_token,
        })
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
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate the full authorization URL.
        let (auth_url, csrf_token) = self
            .client
            .authorize_url(CsrfToken::new_random)
            // Set the desired scopes.
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            // Offline access results in refresh token being provided
            .add_scope(Scope::new("offline_access".to_string()))
            // Set the PKCE code challenge.
            .set_pkce_challenge(pkce_challenge)
            .url();

        store_app_state_in_browser(&AppState {
            return_to: fetch_current_location_from_browser(),
            csrf_token: Some(csrf_token),
            pkce_verifier: Some(pkce_verifier),
        })?;

        redirect_browser(&auth_url.to_string())
    }

    fn logout(&self) -> Result<(), AuthError> {
        todo!()
    }

    fn user(&self) -> Option<crate::provider::User> {
        todo!()
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
    let window = web_sys::window().ok_or(AuthError::Unknown)?;
    let search = window.location().search().map_err(|_| AuthError::Unknown)?;
    let params = UrlSearchParams::new_with_str(&search).map_err(|_| AuthError::Unknown)?;
    let code = params.get("code").ok_or(AuthError::Unknown)?;
    let state = params.get("state").ok_or(AuthError::Unknown)?;
    return Ok((AuthorizationCode::new(code), CsrfTokenState::new(state)));
}

fn redirect_browser(url: &str) -> Result<(), AuthError> {
    let window = web_sys::window().ok_or(AuthError::Unknown)?;
    window
        .open_with_url_and_target_and_features(url, "_self", "")
        .map(|_| ())
        .map_err(|_| AuthError::Unknown)
}

fn store_app_state_in_browser(app_state: &AppState) -> Result<(), AuthError> {
    let window = web_sys::window().ok_or(AuthError::Unknown)?;
    let storage = window
        .session_storage()
        .map_err(|_| AuthError::Unknown)?
        .ok_or(AuthError::Unknown)?;
    let json = serde_json::to_string(app_state).map_err(|_| AuthError::Unknown)?;
    Ok(storage
        .set_item(DEFAULT_APP_STATE_STORAGE_KEY, &json)
        .map_err(|_| AuthError::Unknown)?)
}

fn fetch_app_state_from_browser() -> Result<AppState, AuthError> {
    let window = web_sys::window().ok_or(AuthError::Unknown)?;
    let storage = window
        .session_storage()
        .map_err(|_| AuthError::Unknown)?
        .ok_or(AuthError::Unknown)?;
    let item = storage
        .get_item(DEFAULT_APP_STATE_STORAGE_KEY)
        .map_err(|_| AuthError::Unknown)?;
    match item {
        Some(json) => serde_json::from_str(&json).map_err(|_| AuthError::Unknown),
        _ => Err(AuthError::Unknown),
    }
}

async fn handle_redirect(
    client: &BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    authorization_code: AuthorizationCode,
    state: CsrfTokenState,
) -> Result<AccessToken, AuthError> {
    let app_state = fetch_app_state_from_browser()?;
    let csrf_token = app_state.csrf_token.ok_or(AuthError::Unknown)?;
    let pkce_verifier = app_state.pkce_verifier.ok_or(AuthError::Unknown)?;
    // TODO: Ensure no additional verifications are needed at this point.
    if &state.0 != csrf_token.secret() {
        tracing::debug!("Failed checks");
        return Err(AuthError::Unknown);
    }

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        // TODO: Ensure this policy is none for all situations
        //.redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| AuthError::Unknown)?;

    // Now you can exchange it for an access token.
    let token_result = client
        .exchange_code(authorization_code)
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        // TODO: Use #[from] to do this mapping to prevent discarding error data
        .await
        .map_err(|_| AuthError::Unknown)?;

    // Unwrapping token_result will either produce a Token or a RequestTokenError.
    Ok(AccessToken::new(
        token_result.access_token().clone().into_secret(),
    ))
}
