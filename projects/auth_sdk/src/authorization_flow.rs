use oauth2::{
    AuthUrl, ClientId, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RefreshToken, Scope, StandardTokenResponse, TokenUrl,
    basic::{BasicClient, BasicTokenType},
    url::{ParseError, Url},
};
use oauth2::{AuthorizationCode, reqwest};

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct AuthorizationFlowConfig<'a> {
    client_id: &'a str,
    auth_url: &'a str,
    token_url: &'a str,
    redirect_url: &'a str,
}

impl AuthorizationFlowConfig<'_> {
    pub fn new<'a>(
        client_id: &'a str,
        auth_url: &'a str,
        token_url: &'a str,
        redirect_url: &'a str,
    ) -> AuthorizationFlowConfig<'a> {
        AuthorizationFlowConfig {
            client_id,
            auth_url,
            token_url,
            redirect_url,
        }
    }
}

pub struct CsrfTokenState(String);

impl CsrfTokenState {
    pub fn new(state: String) -> CsrfTokenState {
        CsrfTokenState(state)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Fingerprint {
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

#[derive(Error, Debug)]
pub enum FingerprintGetError {
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum FingerprintSetError {
    #[error("unknown error")]
    Unknown,
}

pub trait FingerprintStore {
    fn get(&self) -> Result<Fingerprint, FingerprintGetError>;
    fn set(&self, fingerprint: Fingerprint) -> Result<(), FingerprintSetError>;
}

#[derive(Error, Debug)]
pub enum TokenGetError {
    #[error("unknown error")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum TokenSetError {
    #[error("unknown error")]
    Unknown,
}
pub trait TokenStore {
    fn get(&self) -> Result<RefreshToken, FingerprintGetError>;
    fn set(&self, fingerprint: RefreshToken) -> Result<(), FingerprintSetError>;
}

#[derive(Error, Debug)]
pub enum AuthorizationFlowDispatchError {
    #[error("unknown executor error")]
    Unknown,
}

/// The executor should use some mechanism to redirect to url and fetch a code.
/// In browser, you can redirect and extract from the query params
/// In native, TODO.
pub trait AuthorizationFlowDispatcher {
    fn dispatch(&self, authorization_url: Url) -> Result<(), AuthorizationFlowDispatchError>;
}

#[derive(Error, Debug)]
pub enum DispatchFlowError {
    #[error("parse failed")]
    ParseFailed(#[from] ParseError),
    #[error("fingerprint set failed")]
    FingerprintSetFailed(#[from] FingerprintSetError),
    #[error("executor failed")]
    DispatchFailed(#[from] AuthorizationFlowDispatchError),
}

/// Abstract function for executing the first part of the flow to get the code
/// This leverages a store because depending on the platform you might navigate away from the app
/// The dispatcher needs to send a request to the auth_url in a platform specific way
pub fn dispatch_code_request(
    config: AuthorizationFlowConfig,
    store: impl FingerprintStore,
    dispatcher: impl AuthorizationFlowDispatcher,
) -> Result<(), DispatchFlowError> {
    let (auth_url, fingerprint) = setup_flow(config)?;
    store.set(fingerprint)?;
    let _ = dispatcher.dispatch(auth_url);
    return Ok(());
}

#[derive(Error, Debug)]
pub enum ExchangeCodeForTokenError {
    #[error("fingerprint get failed")]
    FingerprintGetFailed(#[from] FingerprintGetError),
    #[error("code verification failed")]
    CodeVerificationFaield(#[from] VerifyAuthorizationCodeError),
    #[error("request for token failed")]
    RequestForTokenFailed(#[from] AuthorizationFlowTokenError),
}

pub async fn exchange_code_for_token(
    config: AuthorizationFlowConfig<'_>,
    store: impl FingerprintStore,
    code: AuthorizationCode,
    state: CsrfTokenState,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, ExchangeCodeForTokenError>
{
    tracing::debug!("{:?}", code);
    let fingerprint = store.get()?;
    tracing::debug!("{:?}", fingerprint);
    verify_code_response(&code, state, fingerprint.csrf_token)?;
    let token = request_token(config, code, fingerprint.pkce_verifier).await?;
    Ok(token)
}

#[derive(Error, Debug)]
enum VerifyAuthorizationCodeError {
    #[error("state mismatch")]
    StateMismatch,
}
fn verify_code_response(
    _authorization_code: &AuthorizationCode,
    state: CsrfTokenState,
    csrf_token: CsrfToken,
) -> Result<(), VerifyAuthorizationCodeError> {
    if &state.0 != csrf_token.secret() {
        tracing::debug!("Failed checks");
        return Err(VerifyAuthorizationCodeError::StateMismatch);
    }
    // TODO: Ensure no additional verifications are needed at this point.
    Ok(())
}

/// Setup flow by leveraging specific implementation details in dependent crate (ie oauth2)
fn setup_flow(config: AuthorizationFlowConfig) -> Result<(Url, Fingerprint), ParseError> {
    let client = BasicClient::new(ClientId::new(config.client_id.into()))
        .set_auth_uri(AuthUrl::new(config.auth_url.into())?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.into())?);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Offline access results in refresh token being provided
        .add_scope(Scope::new("offline_access".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok((
        auth_url,
        Fingerprint {
            csrf_token,
            pkce_verifier,
        },
    ))
}

#[derive(Error, Debug)]
enum AuthorizationFlowTokenError {
    #[error("parse error")]
    ParseError(#[from] ParseError),
    #[error("token retrieval failed")]
    TokenRetrievalFailed, // TODO: Learn how to use #[from] here
}
async fn request_token(
    config: AuthorizationFlowConfig<'_>,
    authorization_code: AuthorizationCode,
    pkce_verifier: PkceCodeVerifier,
) -> Result<StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>, AuthorizationFlowTokenError>
{
    let client = BasicClient::new(ClientId::new(config.client_id.into()))
        .set_token_uri(TokenUrl::new(config.token_url.into())?)
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.into())?);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        // TODO: Ensure this policy is none for all situations
        //.redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build"); // TODO: Don't panic, return error

    // Now you can exchange it for an access token.
    let token_result = client
        .exchange_code(authorization_code)
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        // TODO: Use #[from] to do this mapping to prevent discarding error data
        .await
        .map_err(|_| AuthorizationFlowTokenError::TokenRetrievalFailed)?;

    // Unwrapping token_result will either produce a Token or a RequestTokenError.
    Ok(token_result)
}
