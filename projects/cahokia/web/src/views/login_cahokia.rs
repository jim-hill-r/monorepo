use auth_sdk::authorization_flow::{dispatch_code_request, AuthorizationFlowConfig};
use auth_sdk::web::{WebAuthorizationFlowDispatcher, WebSessionStorageFingerprintStore};
use dioxus::prelude::*;

// Note there is no client secret. This code is for server free authorization code flow.
const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";
const REDIRECT_ENDPOINT: &str = "http://local.app.cahokia.com:8080/login/cahokia/code"; // TODO: make this an actual endpoint and use window in library to get host

#[component]
pub fn LoginCahokia() -> Element {
    // Programmatically navigate to the auth0 universal login
    let _ = dispatch_code_request(
        AuthorizationFlowConfig::new(CLIENT_ID, AUTH_URL, TOKEN_URL, REDIRECT_ENDPOINT),
        WebSessionStorageFingerprintStore::new(),
        WebAuthorizationFlowDispatcher::new(),
    );

    // TODO: Do something with the result

    rsx! {}
}
