use oauth2::{
    AuthUrl, ClientId, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, StandardTokenResponse, TokenUrl,
    basic::{BasicClient, BasicTokenType},
    url::{ParseError, Url},
};
use oauth2::{AuthorizationCode, reqwest};

use thiserror::Error;

// Note there is no client secret. This code is for server free authorization code flow.
const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/token";

#[derive(Error, Debug)]
pub enum AuthorizeError {
    #[error("parse error")]
    ParseError(#[from] ParseError),
    #[error("window not found")]
    WindowNotFound,
    #[error("redirect failed")]
    RedirectFailed,
    #[error("unknown authorization error")]
    Unknown,
}

pub fn authorize() -> Result<(), AuthorizeError> {
    let redirect_url = RedirectUrl::new("http://localhost:8080/oauth/code".into())?; // TODO: Fetch location from window and use it
    if let Ok((auth_url, csrf_token, pkce_verifier)) = prepare_authorization_flow(redirect_url) {
        // TODO: Save the csrf_token and pkce_verifier somewhere so upon return from redirect we can use them.

        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                return Err(AuthorizeError::WindowNotFound);
            }
        };
        let w = match window.open_with_url_and_target_and_features(
            &auth_url.to_string(),
            "_self",
            "",
        ) {
            Ok(Some(w)) => w,
            Ok(None) => {
                return Err(AuthorizeError::WindowNotFound);
            }
            Err(e) => {
                return Err(AuthorizeError::RedirectFailed);
            }
        };
        return Ok(());
    }
    return Err(AuthorizeError::Unknown);
}

#[derive(Error, Debug)]
enum PrepareAuthorizationFlowError {}
// TODO: Replace ParseError with PrepareAuthorizationFlowError
fn prepare_authorization_flow(
    redirect_url: RedirectUrl,
) -> Result<(Url, CsrfToken, PkceCodeVerifier), ParseError> {
    let client = BasicClient::new(ClientId::new(CLIENT_ID.into()))
        .set_auth_uri(AuthUrl::new(AUTH_URL.into())?)
        .set_redirect_uri(redirect_url);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok((auth_url, csrf_token, pkce_verifier))
}
#[derive(Error, Debug)]
enum VerifyAuthorizationCodeError {}
fn verify_authorization_code(
    authorization_code: AuthorizationCode,
    csrf_token: CsrfToken,
) -> Result<(), VerifyAuthorizationCodeError> {
    // TODO: Implement this verification

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_token`.
    Ok(())
}
#[derive(Error, Debug)]
enum AuthorizationFlowTokenError {
    #[error("parse error")]
    ParseError(#[from] ParseError),
    #[error("token retrieval failed")]
    TokenRetrievalFailed, // TODO: Learn how to use #[from] here
}
async fn authorization_flow_token(
    authorization_code: AuthorizationCode,
    pkce_verifier: PkceCodeVerifier,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, AuthorizationFlowTokenError>
{
    let client = BasicClient::new(ClientId::new(CLIENT_ID.into()))
        .set_token_uri(TokenUrl::new(TOKEN_URL.into())?);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        //.redirect(reqwest::redirect::Policy::none()) // TODO: Use a WASM supported http_client
        .build()
        .expect("Client should build");

    // Now you can trade it for an access token.
    let token_result = client
        .exchange_code(authorization_code)
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        // TODO: Use #[from] to do this mapping to prevent discarding error data
        .map_err(|_| AuthorizationFlowTokenError::TokenRetrievalFailed)?;

    // Unwrapping token_result will either produce a Token or a RequestTokenError.
    Ok(token_result)
}
