//! Desktop sign-in: the navbar's Login button opens the system browser
//! through `auth_sdk::native::loopback`, and the session stays in memory
//! until the user signs out or closes the app.
//!
//! Plan: `auth_sdk/docs/plans/2026-11-native-sign-in.md` (phase 4).

use std::time::Duration;

use auth_sdk::native::SignedIn;
use auth_sdk::native::loopback::{self, LoopbackConfig};
use auth_sdk::provider::{AuthError, ProviderConfig, User};
use dioxus::prelude::*;

/// The Auth0 tenant cahokia signs in with, the same one as the web app. The
/// trailing slash matters: discovery checks the issuer exactly.
pub const AUTH0_ISSUER: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/";

/// The public client ID of cahokia's native Auth0 application. Native
/// sign-in phase 3 (`infra/issues/native-sign-in-phase-3-auth0-native-app.md`)
/// creates the application; until then a build can pass one in with
/// `CAHOKIA_NATIVE_CLIENT_ID`, and without one Login says sign-in isn't set
/// up. It is public, like the web app's, so it is not a secret.
pub const NATIVE_CLIENT_ID: Option<&str> = option_env!("CAHOKIA_NATIVE_CLIENT_ID");

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

/// The signed-in session, kept in memory only. Staying signed in across
/// restarts is native sign-in phase 6.
/// Nothing reads the access token yet: cahokia has no API to call until a
/// family's tree is stored per user.
#[derive(Debug, Default)]
pub struct Session(
    #[expect(
        dead_code,
        reason = "the access token is for API calls that don't exist yet"
    )]
    Option<SignedIn>,
);

/// The provider settings for `client_id`, or a config error when this build
/// has no client ID. The redirect URL is filled in by the loopback listener.
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
        redirect_url: String::new(),
        issuer_url: Some(issuer.to_string()),
    })
}

/// Sign in through the system browser and a loopback port.
///
/// `open` opens the authorize URL; the app passes
/// [`loopback::open_system_browser`], tests pass a fake browser.
///
/// # Errors
///
/// Any error of [`provider_config`] or [`loopback::sign_in_with`].
pub async fn sign_in(
    client_id: Option<&str>,
    issuer: &str,
    loopback: &LoopbackConfig,
    open: impl FnOnce(&str) -> Result<(), AuthError>,
) -> Result<SignedIn, AuthError> {
    let config = provider_config(client_id, issuer)?;
    loopback::sign_in_with(config, loopback, open).await
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
        AuthError::LoopbackError(_) => {
            "Couldn't start sign-in: another app is using the sign-in ports."
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

/// How long the app waits for the browser to come back.
const SIGN_IN_TIMEOUT: Duration = loopback::DEFAULT_TIMEOUT;

/// The Login, Signing in and Sign out controls, wired to a real sign-in.
#[component]
pub fn DesktopAuth() -> Element {
    let mut view = use_signal(|| AuthView::SignedOut);
    let mut session = use_signal(Session::default);

    let login = move |()| {
        if view() == AuthView::SigningIn {
            return;
        }
        view.set(AuthView::SigningIn);
        spawn(async move {
            let config = LoopbackConfig {
                timeout: SIGN_IN_TIMEOUT,
                ..LoopbackConfig::default()
            };
            let result = sign_in(
                NATIVE_CLIENT_ID,
                AUTH0_ISSUER,
                &config,
                loopback::open_system_browser,
            )
            .await;
            if let Err(error) = &result {
                tracing::warn!("desktop sign-in failed: {error}");
            }
            view.set(view_after(&result));
            session.set(Session(result.ok()));
        });
    };
    let sign_out = move |()| {
        session.set(Session(None));
        view.set(AuthView::SignedOut);
    };

    rsx! {
        AuthControls { view: view(), on_login: login, on_sign_out: sign_out }
    }
}

/// The sign-in controls for one [`AuthView`].
#[component]
pub fn AuthControls(
    view: AuthView,
    on_login: EventHandler<()>,
    on_sign_out: EventHandler<()>,
) -> Element {
    match view {
        AuthView::SignedOut => rsx! {
            button { class: "login", onclick: move |_| on_login.call(()), "Login" }
        },
        AuthView::SigningIn => rsx! {
            button { class: "login", disabled: true, "Signing in…" }
            span { class: "auth-status", role: "status",
                "Finish signing in in your browser."
            }
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    /// Event handlers need a running component, so render through one.
    #[component]
    fn Harness(view: AuthView) -> Element {
        rsx! {
            AuthControls { view, on_login: |()| {}, on_sign_out: |()| {} }
        }
    }

    fn render(view: AuthView) -> String {
        dioxus_ssr::render_element(rsx! { Harness { view } })
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
    fn the_app_starts_signed_out() {
        let html = dioxus_ssr::render_element(rsx! { DesktopAuth {} });
        assert!(html.contains(">Login</button>"), "{html}");
        assert!(!html.contains("Signed in as"), "{html}");
    }

    #[test]
    fn signed_out_shows_only_login() {
        let html = render(AuthView::SignedOut);
        assert!(html.contains(">Login</button>"), "{html}");
        assert!(!html.contains("Sign out"), "{html}");
        assert!(!html.contains("disabled"), "{html}");
    }

    #[test]
    fn signing_in_disables_login_and_points_at_the_browser() {
        let html = render(AuthView::SigningIn);
        assert!(html.contains("disabled"), "{html}");
        assert!(html.contains("Signing in…"), "{html}");
        assert!(
            html.contains("Finish signing in in your browser."),
            "{html}"
        );
    }

    #[test]
    fn signed_in_shows_the_name_and_sign_out() {
        let html = render(AuthView::SignedIn {
            name: "Ada Hill".to_string(),
        });
        assert!(html.contains("Signed in as Ada Hill"), "{html}");
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

    #[test]
    fn the_name_falls_back_to_username_then_email_never_the_sub() {
        let mut u = user();
        assert_eq!(display_name(&u), "you");
        u.email = Some("ada@example.com".to_string());
        assert_eq!(display_name(&u), "ada@example.com");
        u.preferred_username = Some("ada".to_string());
        assert_eq!(display_name(&u), "ada");
        u.name = Some("  ".to_string());
        assert_eq!(display_name(&u), "ada");
        u.name = Some("Ada Hill".to_string());
        assert_eq!(display_name(&u), "Ada Hill");
    }

    #[test]
    fn errors_become_short_messages() {
        let cases = [
            (AuthError::ConfigError("detail-123".into()), "isn't set up"),
            (AuthError::SignInTimedOut("detail-123".into()), "timed out"),
            (
                AuthError::AuthorizationDenied("access_denied".into()),
                "cancelled",
            ),
            (
                AuthError::BrowserApiError("detail-123".into()),
                "open your browser",
            ),
            (
                AuthError::OidcDiscoveryFailed("detail-123".into()),
                "Check your connection",
            ),
            (
                AuthError::LoopbackError("detail-123".into()),
                "sign-in ports",
            ),
            (AuthError::CsrfValidationFailed, "Sign-in failed"),
        ];
        for (error, expected) in cases {
            let message = error_message(&error);
            assert!(message.contains(expected), "{error:?}: {message}");
            assert!(!message.contains("detail-123"), "leaks detail: {message}");
        }
    }

    #[test]
    fn no_client_id_is_a_config_error() {
        for id in [None, Some("")] {
            assert!(matches!(
                provider_config(id, AUTH0_ISSUER),
                Err(AuthError::ConfigError(_))
            ));
        }
        let config = provider_config(Some("abc"), AUTH0_ISSUER).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(config.client_id, "abc");
        assert_eq!(config.issuer_url.as_deref(), Some(AUTH0_ISSUER));
    }

    #[test]
    fn the_issuer_is_the_web_apps_tenant_with_its_trailing_slash() {
        assert!(AUTH0_ISSUER.starts_with("https://"));
        assert!(AUTH0_ISSUER.ends_with(".auth0.com/"));
    }

    #[tokio::test]
    async fn without_a_client_id_the_browser_never_opens() {
        let result = sign_in(None, AUTH0_ISSUER, &fast_loopback(), |_| {
            panic!("the browser must not open without a client ID")
        })
        .await;
        assert_eq!(
            view_after(&result),
            AuthView::Failed {
                message: "Sign-in isn't set up in this build yet.".to_string()
            }
        );
    }

    #[tokio::test]
    async fn sign_in_reaches_the_fake_providers_token_exchange() {
        let issuer = fake_provider(r#"{"access_token":"at","token_type":"Bearer"}"#).await;
        let result = sign_in(
            Some("native-client"),
            &issuer,
            &fast_loopback(),
            follow_redirect,
        )
        .await;
        // The fake provider signs no ID token, so this far means the browser
        // was opened with the right client, the callback came back on the
        // loopback port with the right state, and the code was exchanged.
        assert!(
            matches!(result, Err(AuthError::IdTokenValidationFailed(_))),
            "{result:?}"
        );
        assert_eq!(
            view_after(&result),
            AuthView::Failed {
                message: "Sign-in failed. Try again.".to_string()
            }
        );
    }

    #[tokio::test]
    async fn a_browser_that_cannot_open_says_so() {
        let issuer = fake_provider("{}").await;
        let result = sign_in(Some("native-client"), &issuer, &fast_loopback(), |_| {
            Err(AuthError::BrowserApiError("no browser".to_string()))
        })
        .await;
        assert_eq!(
            view_after(&result),
            AuthView::Failed {
                message: "Couldn't open your browser to sign in.".to_string()
            }
        );
    }

    #[tokio::test]
    async fn an_unreachable_provider_says_check_your_connection() {
        // Bind and drop a port, so nothing is listening there.
        let port = TcpListener::bind("127.0.0.1:0")
            .await
            .and_then(|l| l.local_addr())
            .unwrap_or_else(|e| panic!("{e}"))
            .port();
        let issuer = format!("http://127.0.0.1:{port}/");
        let result = sign_in(Some("native-client"), &issuer, &fast_loopback(), |_| Ok(())).await;
        assert!(
            matches!(&view_after(&result), AuthView::Failed { message } if message.contains("Check your connection")),
            "{result:?}"
        );
    }

    fn fast_loopback() -> LoopbackConfig {
        LoopbackConfig {
            ports: vec![0],
            timeout: Duration::from_secs(5),
        }
    }

    /// A fake browser: read the authorize URL, check it names the client,
    /// and send the callback straight to the loopback port.
    fn follow_redirect(authorize_url: &str) -> Result<(), AuthError> {
        let params: HashMap<String, String> = authorize_url
            .split_once('?')
            .map_or("", |(_, q)| q)
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.to_string(), decode(v)))
            .collect();
        assert_eq!(params["client_id"], "native-client");
        let redirect = params["redirect_uri"].clone();
        assert!(redirect.starts_with("http://127.0.0.1:"), "{redirect}");
        let port: u16 = redirect
            .trim_start_matches("http://127.0.0.1:")
            .trim_end_matches("/callback")
            .parse()
            .map_err(|_| AuthError::ParseError)?;
        let state = params["state"].clone();
        tokio::spawn(async move {
            let mut stream = TcpStream::connect(("127.0.0.1", port))
                .await
                .unwrap_or_else(|e| panic!("{e}"));
            let request =
                format!("GET /callback?code=c&state={state} HTTP/1.1\r\nhost: 127.0.0.1\r\n\r\n");
            let _ = stream.write_all(request.as_bytes()).await;
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response).await;
        });
        Ok(())
    }

    /// Percent-decode a query value (enough for URLs and `state`).
    fn decode(value: &str) -> String {
        let bytes = value.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'%' if i + 2 < bytes.len() => {
                    let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                    out.push(u8::from_str_radix(hex, 16).unwrap_or(b'?'));
                    i += 3;
                }
                b'+' => {
                    out.push(b' ');
                    i += 1;
                }
                b => {
                    out.push(b);
                    i += 1;
                }
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    /// A fake `OpenID` provider on `127.0.0.1`: discovery, an empty key set and
    /// a token endpoint that answers `token_body`.
    async fn fake_provider(token_body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let base = format!(
            "http://{}",
            listener.local_addr().unwrap_or_else(|e| panic!("{e}"))
        );
        let issuer = format!("{base}/");
        let discovery = format!(
            r#"{{"issuer":"{issuer}","authorization_endpoint":"{base}/authorize","token_endpoint":"{base}/oauth/token","jwks_uri":"{base}/jwks.json","response_types_supported":["code"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"]}}"#
        );
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0; 16 * 1024];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let body = match path {
                    "/.well-known/openid-configuration" => discovery.clone(),
                    "/jwks.json" => r#"{"keys":[]}"#.to_string(),
                    "/oauth/token" => token_body.to_string(),
                    _ => "{}".to_string(),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });
        issuer
    }
}
