//! The jimhillr account button: the same sign-in words in every product
//! ([plan](../../../standards/docs/plans/2026-11-jimhillr-account.md)).
//!
//! ```ignore
//! use design_system_dioxus::{AccountButton, AccountState};
//!
//! rsx! {
//!     AccountButton {
//!         state: AccountState::SignedOut,
//!         on_sign_in: move |()| { let _ = provider.login(); },
//!         on_sign_out: move |()| { let _ = provider.logout(); },
//!     }
//! }
//! ```
//!
//! It takes plain values and callbacks, so the design system doesn't depend
//! on `auth_sdk` or any one sign-in provider.

use dioxus::prelude::*;

/// The signed-out button.
pub const SIGN_IN_LABEL: &str = "Sign in with jimhillr";
/// The line under the signed-out button, and its tooltip.
pub const ACCOUNT_HINT: &str = "One jimhillr account works in every jimhillr app.";
/// The signed-in button.
pub const SIGN_OUT_LABEL: &str = "Sign out";
/// The tooltip on the signed-in name.
pub const SIGNED_IN_TITLE: &str = "Signed in with your jimhillr account";

/// Where the person's sign-in stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountState {
    /// The app is still finding out (discovery, or finishing a redirect).
    Loading,
    /// Not signed in.
    SignedOut,
    /// Signed in, with the name to show (a name, username or email).
    SignedIn(String),
    /// Sign-in failed. A short message for the person, never technical
    /// detail (`auth_sdk`'s `AuthError::user_message`).
    Error(String),
}

impl AccountState {
    /// Signed in as the first of `name`, `username` and `email` that is set,
    /// or "Signed in" when the provider gave none.
    #[must_use]
    pub fn signed_in(name: Option<&str>, username: Option<&str>, email: Option<&str>) -> Self {
        let shown = [name, username, email]
            .into_iter()
            .flatten()
            .map(str::trim)
            .find(|s| !s.is_empty())
            .unwrap_or("Signed in");
        AccountState::SignedIn(shown.to_string())
    }
}

/// The account control for an app's header: "Sign in with jimhillr" and its
/// hint when signed out, the person's name and "Sign out" when signed in.
#[component]
pub fn AccountButton(
    state: AccountState,
    on_sign_in: EventHandler<()>,
    on_sign_out: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "ds-account",
            match state {
                AccountState::Loading => rsx! {
                    button { class: "ds-account-sign-in", disabled: true, aria_busy: "true", title: ACCOUNT_HINT, {SIGN_IN_LABEL} }
                },
                AccountState::SignedOut => rsx! {
                    button { class: "ds-account-sign-in", title: ACCOUNT_HINT, onclick: move |_| on_sign_in.call(()), {SIGN_IN_LABEL} }
                    span { class: "ds-account-hint", {ACCOUNT_HINT} }
                },
                AccountState::SignedIn(name) => rsx! {
                    span { class: "ds-account-name", title: SIGNED_IN_TITLE, "{name}" }
                    button { class: "ds-account-sign-out", onclick: move |_| on_sign_out.call(()), {SIGN_OUT_LABEL} }
                },
                AccountState::Error(message) => rsx! {
                    span { class: "ds-account-error error", role: "status", "{message}" }
                    button { class: "ds-account-sign-in", title: ACCOUNT_HINT, onclick: move |_| on_sign_in.call(()), {SIGN_IN_LABEL} }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_name_jimhillr() {
        assert!(SIGN_IN_LABEL.contains("jimhillr"));
        assert!(ACCOUNT_HINT.contains("every jimhillr app"));
        assert!(SIGNED_IN_TITLE.contains("jimhillr account"));
    }

    #[test]
    fn signed_in_shows_the_first_name_given() {
        assert_eq!(
            AccountState::signed_in(Some("Jim"), Some("jim"), Some("j@example.com")),
            AccountState::SignedIn("Jim".into())
        );
        assert_eq!(
            AccountState::signed_in(Some("  "), None, Some("j@example.com")),
            AccountState::SignedIn("j@example.com".into())
        );
        assert_eq!(
            AccountState::signed_in(None, None, None),
            AccountState::SignedIn("Signed in".into())
        );
    }
}
