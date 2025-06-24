use oauth2::{AuthorizationCode, url::Url};
use thiserror::Error;
use web_sys::{UrlSearchParams, wasm_bindgen::JsValue};

use crate::authorization_flow::{
    AuthorizationFlowDispatchError, AuthorizationFlowDispatcher, CsrfTokenState, Fingerprint,
    FingerprintGetError, FingerprintSetError, FingerprintStore,
};

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("code not found")]
    CodeNotFound,
    #[error("state not found")]
    StateNotFound,
    #[error("unknown fetch error")]
    Unknown,
}
pub fn fetch_code_and_state_from_browser() -> Result<(AuthorizationCode, CsrfTokenState), FetchError>
{
    let window = web_sys::window().ok_or(FetchError::Unknown)?;
    let search = window
        .location()
        .search()
        .map_err(|_| FetchError::Unknown)?;
    let params = UrlSearchParams::new_with_str(&search).map_err(|_| FetchError::Unknown)?;
    let code = params.get("code").ok_or(FetchError::CodeNotFound)?;
    let state = params.get("state").ok_or(FetchError::StateNotFound)?;
    return Ok((AuthorizationCode::new(code), CsrfTokenState::new(state)));
}

pub struct WebSessionStorageFingerprintStore {}

impl WebSessionStorageFingerprintStore {
    pub fn new() -> WebSessionStorageFingerprintStore {
        WebSessionStorageFingerprintStore {}
    }
}

impl FingerprintStore for WebSessionStorageFingerprintStore {
    fn get(&self) -> Result<Fingerprint, FingerprintGetError> {
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                return Err(FingerprintGetError::Unknown); // TODO: Improve these error types
            }
        };
        let storage = match window.session_storage() {
            Ok(Some(storage)) => storage,
            _ => {
                return Err(FingerprintGetError::Unknown); // TODO: Improve these error types
            }
        };
        let fingerprint: Fingerprint = match storage.get_item("oauth_flow_fingerprint") {
            Ok(Some(fingerprint)) => match serde_json::from_str(&fingerprint) {
                Ok(fingerprint) => fingerprint,
                _ => {
                    return Err(FingerprintGetError::Unknown);
                } // TODO: Improve these error types
            },
            _ => {
                return Err(FingerprintGetError::Unknown); // TODO: Improve these error types
            }
        };
        Ok(fingerprint)
    }
    fn set(&self, fingerprint: Fingerprint) -> Result<(), FingerprintSetError> {
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                return Err(FingerprintSetError::Unknown); // TODO: Improve these error types
            }
        };
        let storage = match window.session_storage() {
            Ok(Some(storage)) => storage,
            _ => {
                return Err(FingerprintSetError::Unknown); // TODO: Improve these error types
            }
        };
        let fingerprint_json = match serde_json::to_string(&fingerprint) {
            Ok(json) => json,
            _ => {
                return Err(FingerprintSetError::Unknown); // TODO: Improve these error types
            }
        };
        match storage.set_item("oauth_flow_fingerprint", &fingerprint_json) {
            Ok(_) => return Ok(()),
            _ => {
                return Err(FingerprintSetError::Unknown); // TODO: Improve these error types
            }
        };
    }
}

pub struct WebAuthorizationFlowDispatcher {}

impl WebAuthorizationFlowDispatcher {
    pub fn new() -> WebAuthorizationFlowDispatcher {
        WebAuthorizationFlowDispatcher {}
    }
}
impl AuthorizationFlowDispatcher for WebAuthorizationFlowDispatcher {
    fn dispatch(&self, authorization_url: Url) -> Result<(), AuthorizationFlowDispatchError> {
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                return Err(AuthorizationFlowDispatchError::Unknown); // TODO: Improve these error types
            }
        };
        let w = match window.open_with_url_and_target_and_features(
            &authorization_url.to_string(),
            "_self",
            "",
        ) {
            Ok(Some(w)) => w,
            Ok(None) => {
                return Err(AuthorizationFlowDispatchError::Unknown); // TODO: Improve these error types
            }
            Err(e) => {
                return Err(AuthorizationFlowDispatchError::Unknown); // TODO: Improve these error types
            }
        };
        Ok(())
    }
}
