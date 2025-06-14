use auth::{
    AuthorizationFlowConfig, WebAuthorizationFlowDispatcher, WebSessionStorageFingerprintStore,
    dispatch_code_request,
};
use dioxus::prelude::*;

// Note there is no client secret. This code is for server free authorization code flow.
const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/token";

#[component]
pub fn LoginCahokia() -> Element {
    // Programmatically navigate to the auth0 universal login
    let _ = dispatch_code_request(
        AuthorizationFlowConfig::new(
            CLIENT_ID,
            AUTH_URL,
            TOKEN_URL,
            "http://localhost:8080/oauth/code", // TODO: Inspect window for this
        ),
        WebSessionStorageFingerprintStore::new(),
        WebAuthorizationFlowDispatcher::new(),
    );

    // TODO: Do something with the result

    rsx! {}
}
