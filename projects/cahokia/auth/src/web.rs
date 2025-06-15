use oauth2::url::Url;

use crate::authorization_flow::{
    AuthorizationFlowDispatchError, AuthorizationFlowDispatcher, Fingerprint, FingerprintGetError,
    FingerprintSetError, FingerprintStore,
};

pub struct WebSessionStorageFingerprintStore {}

impl WebSessionStorageFingerprintStore {
    pub fn new() -> WebSessionStorageFingerprintStore {
        WebSessionStorageFingerprintStore {}
    }
}

impl FingerprintStore for WebSessionStorageFingerprintStore {
    fn get(&self) -> Result<Fingerprint, FingerprintGetError> {
        // TODO:
        todo!();
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
