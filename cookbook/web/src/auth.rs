//! Sign-in with the jimhillr account in the header. Nothing on the site
//! needs it yet. The button and its words come from the design system's
//! `AccountButton`, the same in every jimhillr product.

use design_system_dioxus::{AccountButton, AccountState};
use dioxus::prelude::*;

#[cfg(target_arch = "wasm32")]
use auth_sdk::provider::{AuthError, AuthProvider};
#[cfg(target_arch = "wasm32")]
use auth_sdk::web::WebAuthProvider;

#[cfg(target_arch = "wasm32")]
pub const AUTH0_DOMAIN: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/";
#[cfg(target_arch = "wasm32")]
pub const CLIENT_ID: &str = "savzmZnyHcvewGkQX8aaInwPFonC9k2x";
#[cfg(target_arch = "wasm32")]
pub const AUTH_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize";
#[cfg(target_arch = "wasm32")]
pub const TOKEN_URL: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/oauth/token";

/// The account state to show for the provider's state.
#[cfg(target_arch = "wasm32")]
fn account_state(auth: Option<&Result<WebAuthProvider, AuthError>>) -> AccountState {
    match auth {
        None => AccountState::Loading,
        Some(Err(err)) => AccountState::Error(err.user_message().into()),
        Some(Ok(provider)) => match provider.user() {
            Some(user) if provider.is_authenticated() => AccountState::signed_in(
                user.name.as_deref(),
                user.preferred_username.as_deref(),
                user.email.as_deref(),
            ),
            _ => AccountState::SignedOut,
        },
    }
}

#[cfg(target_arch = "wasm32")]
pub fn render_auth_section() -> Element {
    let auth = use_context::<Resource<Result<WebAuthProvider, AuthError>>>();
    let auth_state = auth.read();
    let state = account_state(auth_state.as_ref());
    let provider = auth_state.as_ref().and_then(|r| r.as_ref().ok()).cloned();
    let sign_out = provider.clone();

    rsx! {
        AccountButton {
            state,
            on_sign_in: move |()| {
                // The provider redirects; an error leaves the button as it is.
                if let Some(p) = &provider {
                    let _ = p.login();
                }
            },
            on_sign_out: move |()| {
                if let Some(p) = &sign_out {
                    let _ = p.logout();
                }
            },
        }
    }
}

/// Native builds (tests, prerendering) have no sign-in: show the
/// signed-out button so the header looks the same.
#[cfg(not(target_arch = "wasm32"))]
pub fn render_auth_section() -> Element {
    rsx! {
        AccountButton {
            state: AccountState::SignedOut,
            on_sign_in: |()| {},
            on_sign_out: |()| {},
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_auth0_domain_has_trailing_slash() {
        // Auth0's OIDC discovery returns the issuer with a trailing slash,
        // and the openidconnect crate validates an exact match.
        assert!(
            AUTH0_DOMAIN.ends_with("/"),
            "AUTH0_DOMAIN must include trailing slash for OIDC discovery"
        );
    }

    #[test]
    fn errors_show_the_shared_message() {
        let err = AuthError::OidcDiscoveryFailed("https://example.com/.well-known".into());
        assert_eq!(
            account_state(Some(&Err(err))),
            AccountState::Error("Sign-in is unavailable right now".into())
        );
        assert_eq!(account_state(None), AccountState::Loading);
    }
}
