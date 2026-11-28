//! Mobile sign-in: the navbar's Login button opens the system browser at
//! Auth0's authorize URL, and Auth0 sends the browser back to the app's
//! custom URL scheme, [`REDIRECT_URI`]. The OS opens the app with that URL,
//! the app's event handler hands it to [`INBOX`], and the waiting sign-in
//! finishes with it. The session stays in memory until the user signs out
//! or closes the app.
//!
//! Plan: `auth_sdk/docs/plans/2026-11-native-sign-in.md` (phase 5).

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use auth_sdk::native::{NativeAuthClient, SignedIn};
use auth_sdk::provider::{AuthError, ProviderConfig, User};
use dioxus::prelude::*;
use tokio::sync::oneshot;

/// The Auth0 tenant cahokia signs in with, the same one as the web app. The
/// trailing slash matters: discovery checks the issuer exactly.
pub const AUTH0_ISSUER: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/";

/// The public client ID of cahokia's native Auth0 application. Native
/// sign-in phase 3 (`infra/issues/native-sign-in-phase-3-auth0-native-app.md`)
/// creates the application; until then a build can pass one in with
/// `CAHOKIA_NATIVE_CLIENT_ID`, and without one Login says sign-in isn't set
/// up. It is public, like the web app's, so it is not a secret.
pub const NATIVE_CLIENT_ID: Option<&str> = option_env!("CAHOKIA_NATIVE_CLIENT_ID");

/// The app's redirect URI: a private-use URI scheme named after a domain we
/// own, reversed (RFC 8252 section 7.1). The native Auth0 application lists
/// it as an Allowed Callback URL, and the app registers the scheme with the
/// OS (an iOS URL type, an Android intent filter).
pub const REDIRECT_URI: &str = "com.jimhillr.cahokia://callback";

/// How long the app waits for the browser to come back.
pub const SIGN_IN_TIMEOUT: Duration = Duration::from_mins(5);

/// Whether `url` is a sign-in callback: [`REDIRECT_URI`] itself, with a
/// query or a fragment and nothing else. Any other URL the app is opened
/// with is not for sign-in.
#[must_use]
pub fn is_callback(url: &str) -> bool {
    url.strip_prefix(REDIRECT_URI)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['?', '#']))
}

/// Hands the callback URL the OS opens the app with to the sign-in waiting
/// for it. At most one sign-in waits at a time.
#[derive(Debug, Default)]
pub struct CallbackInbox {
    waiting: Mutex<Option<oneshot::Sender<String>>>,
}

/// The app's one inbox, fed by the event handler in `main`.
pub static INBOX: CallbackInbox = CallbackInbox::new();

impl CallbackInbox {
    /// An inbox with no sign-in waiting.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            waiting: Mutex::new(None),
        }
    }

    /// Wait for the next callback. A sign-in that was already waiting stops
    /// waiting, as if cancelled.
    pub fn expect(&self) -> oneshot::Receiver<String> {
        let (sender, receiver) = oneshot::channel();
        *self.slot() = Some(sender);
        receiver
    }

    /// Pass `url` to the waiting sign-in. Returns whether it was taken: a
    /// URL that isn't a callback, or a callback with no sign-in waiting
    /// (such as one that arrives after a timeout), is dropped.
    pub fn deliver(&self, url: &str) -> bool {
        if !is_callback(url) {
            return false;
        }
        self.slot()
            .take()
            .is_some_and(|sender| sender.send(url.to_string()).is_ok())
    }

    /// Stop the waiting sign-in, if any.
    pub fn cancel(&self) {
        self.slot().take();
    }

    fn slot(&self) -> std::sync::MutexGuard<'_, Option<oneshot::Sender<String>>> {
        // A panic while holding the lock leaves only an `Option` behind,
        // which is still valid.
        self.waiting.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The provider settings for `client_id`, or a config error when this build
/// has no client ID.
///
/// # Errors
///
/// [`AuthError::ConfigError`] when `client_id` is `None` or empty.
pub fn provider_config(client_id: Option<&str>, issuer: &str) -> Result<ProviderConfig, AuthError> {
    let client_id = client_id
        .filter(|id| !id.is_empty())
        .ok_or_else(|| AuthError::ConfigError("no native client ID in this build".to_string()))?;
    Ok(ProviderConfig {
        client_id: client_id.to_string(),
        auth_url: String::new(),
        token_url: String::new(),
        redirect_url: REDIRECT_URI.to_string(),
        issuer_url: Some(issuer.to_string()),
    })
}

/// Sign in through the system browser and the custom URL scheme.
///
/// `open` opens the authorize URL; the app passes [`open_system_browser`],
/// tests pass a fake browser. The inbox starts waiting before the browser
/// opens, so a fast callback isn't missed.
///
/// # Errors
///
/// Any error of [`provider_config`], [`NativeAuthClient::new`], `open` or
/// [`NativeAuthClient::complete_login`];
/// [`AuthError::SignInTimedOut`] when no callback arrives within `wait`, and
/// [`AuthError::AuthorizationDenied`] when the sign-in is cancelled.
pub async fn sign_in(
    client_id: Option<&str>,
    issuer: &str,
    inbox: &CallbackInbox,
    wait: Duration,
    open: impl FnOnce(&str) -> Result<(), AuthError>,
) -> Result<SignedIn, AuthError> {
    let config = provider_config(client_id, issuer)?;
    let client = NativeAuthClient::new(config).await?;
    let request = client.begin_login();
    let callback = inbox.expect();
    if let Err(error) = open(&request.authorize_url) {
        inbox.cancel();
        return Err(error);
    }
    let callback_url = match tokio::time::timeout(wait, callback).await {
        Ok(Ok(url)) => url,
        Ok(Err(_)) => {
            return Err(AuthError::AuthorizationDenied(
                "sign-in was cancelled in the app".to_string(),
            ));
        }
        Err(_) => {
            inbox.cancel();
            return Err(AuthError::SignInTimedOut(format!(
                "no callback to {REDIRECT_URI} within {}s",
                wait.as_secs()
            )));
        }
    };
    client.complete_login(request.pending, &callback_url).await
}

/// Open `url` in the system browser (never an embedded web view, which
/// RFC 8252 forbids because the app could read the user's password).
///
/// # Errors
///
/// [`AuthError::BrowserApiError`] when no browser can be opened.
pub fn open_system_browser(url: &str) -> Result<(), AuthError> {
    webbrowser::open(url).map_err(|e| AuthError::BrowserApiError(e.to_string()))
}

/// What the navbar shows about sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthView {
    /// Not signed in: a Login button.
    SignedOut,
    /// The browser is open and the app is waiting for the callback.
    SigningIn,
    /// Signed in: the user's name and a Sign out button.
    SignedIn { name: String },
    /// The last sign-in failed: a short message and Login again.
    Failed { message: String },
}

/// The name to show for a signed-in user: their name, else their username,
/// else their email. The `sub` (such as `auth0|64f...`) means nothing to a
/// person, so it is never shown.
#[must_use]
pub fn display_name(user: &User) -> String {
    [&user.name, &user.preferred_username, &user.email]
        .into_iter()
        .flatten()
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
        .unwrap_or("you")
        .to_string()
}

/// A short message for a failed sign-in, in words a user can act on. The
/// full error goes to the log.
#[must_use]
pub fn error_message(error: &AuthError) -> String {
    match error {
        AuthError::ConfigError(_) => "Sign-in isn't set up in this build yet.",
        AuthError::SignInTimedOut(_) => "Sign-in timed out. Try again.",
        AuthError::AuthorizationDenied(_) => "Sign-in was cancelled.",
        AuthError::BrowserApiError(_) => "Couldn't open your browser to sign in.",
        AuthError::OidcDiscoveryFailed(_) | AuthError::HttpClientError(_) => {
            "Couldn't reach the sign-in service. Check your connection and try again."
        }
        _ => "Sign-in failed. Try again.",
    }
    .to_string()
}

/// The view after a sign-in finishes.
#[must_use]
pub fn view_after(result: &Result<SignedIn, AuthError>) -> AuthView {
    match result {
        Ok(signed_in) => AuthView::SignedIn {
            name: display_name(&signed_in.user),
        },
        Err(error) => AuthView::Failed {
            message: error_message(error),
        },
    }
}

/// The signed-in session, kept in memory only. Staying signed in across
/// restarts is native sign-in phase 6.
#[derive(Debug, Default)]
pub struct Session(
    #[expect(
        dead_code,
        reason = "the access token is for API calls that don't exist yet"
    )]
    Option<SignedIn>,
);

/// The Login, Signing in and Sign out controls, wired to a real sign-in.
#[component]
pub fn MobileAuth() -> Element {
    let mut view = use_signal(|| AuthView::SignedOut);
    let mut session = use_signal(Session::default);

    let login = move |()| {
        if view() == AuthView::SigningIn {
            return;
        }
        view.set(AuthView::SigningIn);
        spawn(async move {
            let result = sign_in(
                NATIVE_CLIENT_ID,
                AUTH0_ISSUER,
                &INBOX,
                SIGN_IN_TIMEOUT,
                open_system_browser,
            )
            .await;
            if let Err(error) = &result {
                tracing::warn!("mobile sign-in failed: {error}");
            }
            view.set(view_after(&result));
            session.set(Session(result.ok()));
        });
    };
    let cancel = move |()| INBOX.cancel();
    let sign_out = move |()| {
        session.set(Session(None));
        view.set(AuthView::SignedOut);
    };

    rsx! {
        AuthControls { view: view(), on_login: login, on_cancel: cancel, on_sign_out: sign_out }
    }
}

/// The sign-in controls for one [`AuthView`].
#[component]
pub fn AuthControls(
    view: AuthView,
    on_login: EventHandler<()>,
    on_cancel: EventHandler<()>,
    on_sign_out: EventHandler<()>,
) -> Element {
    match view {
        AuthView::SignedOut => rsx! {
            button { class: "login", onclick: move |_| on_login.call(()), "Login" }
        },
        AuthView::SigningIn => rsx! {
            span { class: "auth-status", role: "status",
                "Finish signing in in your browser."
            }
            button { class: "cancel", onclick: move |_| on_cancel.call(()), "Cancel" }
        },
        AuthView::SignedIn { name } => rsx! {
            span { class: "auth-user", "Signed in as {name}" }
            button { class: "sign-out", onclick: move |_| on_sign_out.call(()), "Sign out" }
        },
        AuthView::Failed { message } => rsx! {
            button { class: "login", onclick: move |_| on_login.call(()), "Login" }
            span { class: "auth-error", role: "alert", "{message}" }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// A fake OIDC provider on 127.0.0.1: discovery, an empty key set, and
    /// a token endpoint that refuses every code. Returns its issuer URL.
    async fn fake_provider() -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let base = format!(
            "http://{}",
            listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
        );
        let issuer = format!("{base}/");
        let discovery = serde_json::json!({
            "issuer": issuer,
            "authorization_endpoint": format!("{base}/authorize"),
            "token_endpoint": format!("{base}/oauth/token"),
            "jwks_uri": format!("{base}/.well-known/jwks.json"),
            "response_types_supported": ["code"],
            "subject_types_supported": ["public"],
            "id_token_signing_alg_values_supported": ["RS256"],
        })
        .to_string();
        let routes: HashMap<&'static str, (u16, String)> = HashMap::from([
            ("/.well-known/openid-configuration", (200, discovery)),
            (
                "/.well-known/jwks.json",
                (200, r#"{"keys":[]}"#.to_string()),
            ),
            (
                "/oauth/token",
                (403, r#"{"error":"invalid_grant"}"#.to_string()),
            ),
        ]);
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0; 16 * 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let (status, body) = routes.get(path).cloned().unwrap_or((404, "{}".to_string()));
                let response = format!(
                    "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        issuer
    }

    fn query(url: &str) -> HashMap<String, String> {
        url::Url::parse(url)
            .unwrap_or_else(|e| panic!("{e}"))
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    fn user() -> User {
        User {
            sub: "auth0|abc123".to_string(),
            name: None,
            email: None,
            email_verified: None,
            picture: None,
            preferred_username: None,
        }
    }

    #[test]
    fn only_the_redirect_uri_is_a_callback() {
        assert!(is_callback(REDIRECT_URI));
        assert!(is_callback(
            "com.jimhillr.cahokia://callback?code=abc&state=xyz"
        ));
        assert!(is_callback("com.jimhillr.cahokia://callback#code=abc"));
        assert!(!is_callback("com.jimhillr.cahokia://callbackx?code=abc"));
        assert!(!is_callback("com.jimhillr.cahokia://other?code=abc"));
        assert!(!is_callback("com.evil.cahokia://callback?code=abc"));
        assert!(!is_callback(
            "https://example.com/?u=com.jimhillr.cahokia://callback"
        ));
    }

    #[tokio::test]
    async fn a_callback_reaches_the_waiting_sign_in() {
        let inbox = CallbackInbox::new();
        let receiver = inbox.expect();
        let url = format!("{REDIRECT_URI}?code=c&state=s");
        assert!(inbox.deliver(&url));
        assert_eq!(receiver.await.ok(), Some(url));
    }

    #[test]
    fn other_urls_and_unawaited_callbacks_are_dropped() {
        let inbox = CallbackInbox::new();
        assert!(
            !inbox.deliver(&format!("{REDIRECT_URI}?code=c")),
            "no one waiting"
        );
        let _receiver = inbox.expect();
        assert!(!inbox.deliver("com.jimhillr.cahokia://elsewhere?code=c"));
        assert!(inbox.deliver(&format!("{REDIRECT_URI}?code=c")));
        assert!(
            !inbox.deliver(&format!("{REDIRECT_URI}?code=again")),
            "a second callback has no one waiting"
        );
    }

    #[tokio::test]
    async fn a_new_sign_in_replaces_the_one_waiting() {
        let inbox = CallbackInbox::new();
        let first = inbox.expect();
        let second = inbox.expect();
        assert!(first.await.is_err(), "the first sign-in stops waiting");
        assert!(inbox.deliver(REDIRECT_URI));
        assert!(second.await.is_ok());
    }

    #[test]
    fn a_build_without_a_client_id_is_not_set_up() {
        for id in [None, Some("")] {
            assert!(matches!(
                provider_config(id, AUTH0_ISSUER),
                Err(AuthError::ConfigError(_))
            ));
        }
        let config = provider_config(Some("abc"), AUTH0_ISSUER).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(config.redirect_url, REDIRECT_URI);
        assert_eq!(config.issuer_url.as_deref(), Some(AUTH0_ISSUER));
    }

    #[tokio::test]
    async fn sign_in_opens_the_browser_and_finishes_with_the_callback() {
        let issuer = fake_provider().await;
        let inbox = Arc::new(CallbackInbox::new());
        let feeder = Arc::clone(&inbox);
        let opened = Arc::new(Mutex::new(None));
        let seen = Arc::clone(&opened);
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &inbox,
            SIGN_IN_TIMEOUT,
            |url| {
                *seen.lock().unwrap_or_else(PoisonError::into_inner) = Some(url.to_string());
                let params = query(url);
                let callback = format!("{REDIRECT_URI}?code=the-code&state={}", params["state"]);
                // The OS opens the app with the callback while the browser is
                // still in front.
                tokio::spawn(async move {
                    assert!(feeder.deliver(&callback));
                });
                Ok(())
            },
        )
        .await;
        // The fake token endpoint refuses the code, so reaching it shows the
        // callback was passed on with the right state.
        assert!(
            matches!(result, Err(AuthError::TokenExchangeError(_))),
            "{result:?}"
        );
        let authorize = opened
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .unwrap_or_else(|| panic!("the browser was never opened"));
        let params = query(&authorize);
        assert_eq!(params["redirect_uri"], REDIRECT_URI);
        assert_eq!(params["client_id"], "native-client");
        assert_eq!(params["code_challenge_method"], "S256");
    }

    #[tokio::test]
    async fn a_callback_with_the_wrong_state_is_refused() {
        let issuer = fake_provider().await;
        let inbox = Arc::new(CallbackInbox::new());
        let feeder = Arc::clone(&inbox);
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &inbox,
            SIGN_IN_TIMEOUT,
            |_| {
                tokio::spawn(async move {
                    feeder.deliver(&format!("{REDIRECT_URI}?code=c&state=forged"));
                });
                Ok(())
            },
        )
        .await;
        assert!(
            matches!(result, Err(AuthError::CsrfValidationFailed)),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn no_callback_times_out() {
        let issuer = fake_provider().await;
        let inbox = CallbackInbox::new();
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &inbox,
            Duration::from_millis(50),
            |_| Ok(()),
        )
        .await;
        assert!(
            matches!(result, Err(AuthError::SignInTimedOut(_))),
            "{result:?}"
        );
        assert!(
            !inbox.deliver(REDIRECT_URI),
            "a late callback has no one waiting"
        );
    }

    #[tokio::test]
    async fn cancel_stops_the_sign_in() {
        let issuer = fake_provider().await;
        let inbox = Arc::new(CallbackInbox::new());
        let canceller = Arc::clone(&inbox);
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &inbox,
            SIGN_IN_TIMEOUT,
            |_| {
                tokio::spawn(async move { canceller.cancel() });
                Ok(())
            },
        )
        .await;
        assert!(
            matches!(result, Err(AuthError::AuthorizationDenied(_))),
            "{result:?}"
        );
        assert_eq!(
            error_message(&result.err().unwrap_or(AuthError::Unknown)),
            "Sign-in was cancelled."
        );
    }

    #[tokio::test]
    async fn a_browser_that_will_not_open_stops_waiting() {
        let issuer = fake_provider().await;
        let inbox = CallbackInbox::new();
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &inbox,
            SIGN_IN_TIMEOUT,
            |_| Err(AuthError::BrowserApiError("no browser".to_string())),
        )
        .await;
        assert!(
            matches!(result, Err(AuthError::BrowserApiError(_))),
            "{result:?}"
        );
        assert!(!inbox.deliver(REDIRECT_URI));
    }

    #[tokio::test]
    async fn sign_in_without_a_client_id_never_opens_the_browser() {
        let result = sign_in(
            None,
            AUTH0_ISSUER,
            &CallbackInbox::new(),
            SIGN_IN_TIMEOUT,
            |_| panic!("opened the browser"),
        )
        .await;
        assert_eq!(
            error_message(&result.err().unwrap_or(AuthError::Unknown)),
            "Sign-in isn't set up in this build yet."
        );
    }

    #[test]
    fn the_name_shown_is_never_the_subject() {
        assert_eq!(display_name(&user()), "you");
        let mut u = user();
        u.email = Some("ada@example.com".to_string());
        assert_eq!(display_name(&u), "ada@example.com");
        u.preferred_username = Some("ada".to_string());
        assert_eq!(display_name(&u), "ada");
        u.name = Some("  ".to_string());
        assert_eq!(display_name(&u), "ada");
        u.name = Some("Ada Lovelace".to_string());
        assert_eq!(display_name(&u), "Ada Lovelace");
    }

    /// Event handlers need a running component, so render through one.
    #[component]
    fn Harness(view: AuthView) -> Element {
        rsx! {
            AuthControls { view, on_login: |()| {}, on_cancel: |()| {}, on_sign_out: |()| {} }
        }
    }

    fn render(view: AuthView) -> String {
        dioxus_ssr::render_element(rsx! { Harness { view } })
    }

    #[test]
    fn the_app_starts_signed_out() {
        let html = dioxus_ssr::render_element(rsx! { MobileAuth {} });
        assert!(html.contains(">Login</button>"), "{html}");
        assert!(!html.contains("Signed in as"), "{html}");
    }

    #[test]
    fn signing_in_says_where_to_finish_and_can_be_cancelled() {
        let html = render(AuthView::SigningIn);
        assert!(
            html.contains("Finish signing in in your browser."),
            "{html}"
        );
        assert!(html.contains(">Cancel</button>"), "{html}");
        assert!(!html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn signed_in_shows_the_name_and_sign_out() {
        let html = render(AuthView::SignedIn {
            name: "Ada".to_string(),
        });
        assert!(html.contains("Signed in as Ada"), "{html}");
        assert!(html.contains(">Sign out</button>"), "{html}");
        assert!(!html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn a_failure_shows_the_message_and_login_again() {
        let html = render(AuthView::Failed {
            message: "Sign-in timed out. Try again.".to_string(),
        });
        assert!(html.contains("role=\"alert\""), "{html}");
        assert!(html.contains("Sign-in timed out. Try again."), "{html}");
        assert!(html.contains(">Login</button>"), "{html}");
    }
}
