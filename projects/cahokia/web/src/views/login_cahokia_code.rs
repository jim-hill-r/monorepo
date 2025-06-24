use auth::{
    authorization_flow::{AuthorizationFlowConfig, trade_code_for_token},
    web::{WebSessionStorageFingerprintStore, fetch_code_and_state_from_browser},
};
use dioxus::{logger::tracing, prelude::*};

// TODO: Recover code from the redirected url params
// TODO: Recover state from the redirected url params and compare with csrf_token
// TODO: Use the code to fetch the token
// TODO: Store the token securely for use elsewhere (need to research this).

// TODO: These values are repeated in two locations. Ensure these are pulled from environment
// Note there is no client secret. This code is for server free authorization code flow.
const CLIENT_ID: &str = "6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E";
const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/token";
const REDIRECT_ENDPOINT: &str = "http://localhost:8080/login/cahokia/code"; // TODO: make this an actual endpoint and use window in library to get host

#[component]
pub fn LoginCahokiaCode() -> Element {
    let success = use_resource(move || async move {
        let (authorization_code, state) = match fetch_code_and_state_from_browser() {
            Ok(params) => params,
            _ => {
                return "fetch failed.".to_string();
            }
        };
        tracing::debug!("code success!");
        let _token = match trade_code_for_token(
            AuthorizationFlowConfig::new(CLIENT_ID, AUTH_URL, TOKEN_URL, REDIRECT_ENDPOINT),
            WebSessionStorageFingerprintStore::new(),
            authorization_code,
            state,
        )
        .await
        {
            Ok(_token) => {
                return "token fetched!".to_string();
            }
            _ => {
                return "trade failed.".to_string();
            }
        };

        // TODO: Save token and redirect back to original page
    });

    rsx! {
        if let Some(success) = &*success.read() {
            "{success}"
        } else {
            "Loading..."
        }
    }
}
