use dioxus::prelude::*;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    Scope, TokenUrl,
    basic::BasicClient,
    url::{ParseError, Url},
};
use thiserror::Error;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

#[component]
pub fn Navbar() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div {
            id: "navbar",
            // TODO: Replace count with actual login logic
            button { onclick: move |_| count += 1, "Login" }
        }
    }
}

#[derive(Error, Debug)]
enum PrepareAuthorizationFlowError {}

// TODO: Replace ParseError with PrepareAuthorizationFlowError
fn prepare_authorization_flow() -> Result<(Url, CsrfToken, PkceCodeVerifier), ParseError> {
    let client = BasicClient::new(ClientId::new("client_id".to_string()))
        .set_client_secret(ClientSecret::new("client_secret".to_string()))
        .set_auth_uri(AuthUrl::new("http://authorize".to_string())?)
        .set_token_uri(TokenUrl::new("http://token".to_string())?)
        // Set the URL the user will be redirected to after the authorization process.
        .set_redirect_uri(RedirectUrl::new("http://redirect".to_string())?);

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

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    Ok((auth_url, csrf_token, pkce_verifier))
}
