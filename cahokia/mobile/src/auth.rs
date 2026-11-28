//! Mobile sign-in (RFC 8252 section 7.1): the navbar's Login button opens
//! Auth0 in the system browser, Auth0 redirects to the app's own custom URL
//! scheme, [`REDIRECT_URI`], and the OS opens the app with that URL, which
//! finishes the sign-in. The session stays in memory until the user signs
//! out or the app is closed.
//!
//! iOS hands the opened URL to the app as tao's `Event::Opened`, registered
//! in `ios/Info.plist`. Android doesn't yet: tao doesn't forward an
//! activity's intent, so Login there says sign-in isn't available
//! (`issues/native-sign-in-phase-5b-android-callback.md`).
//!
//! Plan: `auth_sdk/docs/plans/2026-11-native-sign-in.md` (phase 5).

use auth_sdk::native::{NativeAuthClient, PendingLogin, SignedIn};
use auth_sdk::provider::{AuthError, ProviderConfig, User};
use dioxus::prelude::*;

/// The Auth0 tenant cahokia signs in with, the same one as the web app. The
/// trailing slash matters: discovery checks the issuer exactly.
pub const AUTH0_ISSUER: &str = "https://dev-jdadpn4pckxevrv5.us.auth0.com/";

/// The public client ID of cahokia's native Auth0 application, shared with
/// the desktop app. Native sign-in phase 3
/// (`infra/issues/native-sign-in-phase-3-auth0-native-app.md`) creates the
/// application; until then a build can pass one in with
/// `CAHOKIA_NATIVE_CLIENT_ID`, and without one Login says sign-in isn't set
/// up. It is public, like the web app's, so it is not a secret.
pub const NATIVE_CLIENT_ID: Option<&str> = option_env!("CAHOKIA_NATIVE_CLIENT_ID");

/// The app's custom URL scheme, the reverse of a domain we own so no other
/// app is likely to claim it. `ios/Info.plist` registers it, and phase 3
/// lists [`REDIRECT_URI`] as an Allowed Callback URL.
pub const URL_SCHEME: &str = "com.jimhillr.cahokia";

/// Where Auth0 sends the browser after sign-in.
pub const REDIRECT_URI: &str = "com.jimhillr.cahokia://callback";

/// What Login says on a platform that can't take the callback back yet.
pub const ANDROID_NOT_YET: &str = "Sign-in isn't available on Android yet.";

/// What the navbar shows about sign-in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthView {
    /// Not signed in: a Login button.
    SignedOut,
    /// The browser is open and the app is waiting to be opened with the
    /// callback. A Cancel button gives up, since the app can't tell when
    /// the user closes the browser without signing in.
    SigningIn,
    /// Signed in: the user's name and a Sign out button.
    SignedIn { name: String },
    /// The last sign-in failed: a short message and Login again.
    Failed { message: String },
}

/// A sign-in that is waiting for its callback.
pub struct InFlight {
    client: NativeAuthClient,
    pending: PendingLogin,
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

/// The provider settings for `client_id`, redirecting to [`REDIRECT_URI`].
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

/// Start a sign-in: discover the provider, then open its authorize URL with
/// `open`. The app passes [`open_system_browser`], tests a fake browser.
///
/// # Errors
///
/// Any error of [`provider_config`], [`NativeAuthClient::new`] or `open`.
pub async fn begin(
    client_id: Option<&str>,
    issuer: &str,
    open: impl FnOnce(&str) -> Result<(), AuthError>,
) -> Result<InFlight, AuthError> {
    let client = NativeAuthClient::new(provider_config(client_id, issuer)?).await?;
    let request = client.begin_login();
    open(&request.authorize_url)?;
    Ok(InFlight {
        client,
        pending: request.pending,
    })
}

/// Finish a sign-in from the URL the OS opened the app with.
///
/// # Errors
///
/// Any error of [`NativeAuthClient::complete_login`].
pub async fn finish(in_flight: InFlight, callback_url: &str) -> Result<SignedIn, AuthError> {
    in_flight
        .client
        .complete_login(in_flight.pending, callback_url)
        .await
}

/// Whether a URL the app was opened with is meant for sign-in. Other URLs
/// (a future share link, say) are left alone; a sign-in URL with the wrong
/// path or `state` still fails in [`finish`].
#[must_use]
pub fn is_sign_in_callback(url: &str) -> bool {
    url.split_once("://")
        .is_some_and(|(scheme, _)| scheme.eq_ignore_ascii_case(URL_SCHEME))
}

/// Open the authorize URL in the system browser (Safari), never a web view.
///
/// # Errors
///
/// [`AuthError::BrowserApiError`] when the browser can't be opened.
pub fn open_system_browser(url: &str) -> Result<(), AuthError> {
    webbrowser::open(url).map_err(|e| AuthError::BrowserApiError(e.to_string()))
}

/// Why Login can't work on `os` (a `std::env::consts::OS` value), if it
/// can't.
#[must_use]
pub fn unsupported_reason(os: &str) -> Option<&'static str> {
    (os == "android").then_some(ANDROID_NOT_YET)
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

/// What Login says when the app is opened with a sign-in URL it wasn't
/// waiting for, such as after the OS closed the app while the browser was
/// open.
pub const NOT_WAITING: &str = "Sign-in was interrupted. Try again.";

/// The Login, Signing in and Sign out controls, wired to a real sign-in.
/// It needs the mobile window, so tests render [`AuthControls`] instead.
#[component]
pub fn MobileAuth() -> Element {
    let mut view = use_signal(|| AuthView::SignedOut);
    let mut session = use_signal(Session::default);
    let mut in_flight = use_signal(|| None::<InFlight>);

    dioxus::mobile::use_wry_event_handler(move |event, _| {
        let dioxus::mobile::tao::event::Event::Opened { urls } = event else {
            return;
        };
        for url in urls.iter().map(ToString::to_string) {
            if !is_sign_in_callback(&url) {
                continue;
            }
            let Some(waiting) = in_flight.write().take() else {
                view.set(AuthView::Failed {
                    message: NOT_WAITING.to_string(),
                });
                continue;
            };
            spawn(async move {
                let result = finish(waiting, &url).await;
                if let Err(error) = &result {
                    tracing::warn!("mobile sign-in failed: {error}");
                }
                view.set(view_after(&result));
                session.set(Session(result.ok()));
            });
        }
    });

    let login = move |()| {
        if let Some(reason) = unsupported_reason(std::env::consts::OS) {
            view.set(AuthView::Failed {
                message: reason.to_string(),
            });
            return;
        }
        if view() == AuthView::SigningIn {
            return;
        }
        view.set(AuthView::SigningIn);
        spawn(async move {
            match begin(NATIVE_CLIENT_ID, AUTH0_ISSUER, open_system_browser).await {
                Ok(waiting) => in_flight.set(Some(waiting)),
                Err(error) => {
                    tracing::warn!("mobile sign-in failed to start: {error}");
                    view.set(view_after(&Err(error)));
                }
            }
        });
    };
    let cancel = move |()| {
        in_flight.set(None);
        view.set(AuthView::SignedOut);
    };
    let sign_out = move |()| {
        session.set(Session(None));
        view.set(AuthView::SignedOut);
    };

    rsx! {
        AuthControls {
            view: view(),
            on_login: login,
            on_cancel: cancel,
            on_sign_out: sign_out,
        }
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
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Event handlers need a running component, so render through one.
    #[component]
    fn Harness(view: AuthView) -> Element {
        rsx! {
            AuthControls {
                view,
                on_login: |()| {},
                on_cancel: |()| {},
                on_sign_out: |()| {},
            }
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
    fn mobile_signed_out_shows_only_login() {
        let html = render(AuthView::SignedOut);
        assert!(html.contains(">Login</button>"), "{html}");
        assert!(!html.contains("Sign out"), "{html}");
        assert!(!html.contains("Cancel"), "{html}");
    }

    #[test]
    fn mobile_signing_in_points_at_the_browser_and_can_cancel() {
        let html = render(AuthView::SigningIn);
        assert!(
            html.contains("Finish signing in in your browser."),
            "{html}"
        );
        assert!(html.contains(">Cancel</button>"), "{html}");
        assert!(!html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn mobile_signed_in_shows_the_name_and_sign_out() {
        let html = render(AuthView::SignedIn {
            name: "Ada Hill".to_string(),
        });
        assert!(html.contains("Signed in as Ada Hill"), "{html}");
        assert!(html.contains(">Sign out</button>"), "{html}");
        assert!(!html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn mobile_failure_shows_the_message_and_login_again() {
        let html = render(AuthView::Failed {
            message: NOT_WAITING.to_string(),
        });
        assert!(html.contains("role=\"alert\""), "{html}");
        assert!(html.contains(NOT_WAITING), "{html}");
        assert!(html.contains(">Login</button>"), "{html}");
    }

    #[test]
    fn mobile_name_falls_back_to_username_then_email_never_the_sub() {
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
    fn mobile_error_messages_never_leak_details() {
        let cases = [
            (AuthError::ConfigError("detail-123".into()), "set up"),
            (
                AuthError::AuthorizationDenied("access_denied".into()),
                "cancelled",
            ),
            (
                AuthError::BrowserApiError("detail-123".into()),
                "open your browser",
            ),
            (
                AuthError::HttpClientError("detail-123".into()),
                "Check your connection",
            ),
            (
                AuthError::CallbackMismatch("detail-123".into()),
                "Sign-in failed",
            ),
        ];
        for (error, expected) in cases {
            let message = error_message(&error);
            assert!(message.contains(expected), "{error:?}: {message}");
            assert!(!message.contains("detail-123"), "leaks detail: {message}");
        }
    }

    #[test]
    fn mobile_redirect_is_the_custom_scheme() {
        assert_eq!(REDIRECT_URI, format!("{URL_SCHEME}://callback"));
        let config = provider_config(Some("abc"), AUTH0_ISSUER).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(config.redirect_url, REDIRECT_URI);
        assert_eq!(config.client_id, "abc");
        assert_eq!(config.issuer_url.as_deref(), Some(AUTH0_ISSUER));
        for id in [None, Some("")] {
            assert!(matches!(
                provider_config(id, AUTH0_ISSUER),
                Err(AuthError::ConfigError(_))
            ));
        }
    }

    #[test]
    fn mobile_info_plist_registers_the_scheme() {
        let plist = include_str!("../ios/Info.plist");
        assert!(plist.contains("<key>CFBundleURLTypes</key>"), "{plist}");
        assert!(plist.contains(&format!("<string>{URL_SCHEME}</string>")));
        // dx uses this file as it is, so it must name the binary.
        assert!(plist.contains("<string>mobile</string>"));
    }

    #[test]
    fn only_urls_with_the_apps_scheme_are_sign_in_callbacks() {
        assert!(is_sign_in_callback(
            "com.jimhillr.cahokia://callback?code=c&state=s"
        ));
        assert!(is_sign_in_callback("COM.JIMHILLR.CAHOKIA://callback"));
        for other in [
            "https://jimhillr-cahokia.pages.dev/callback?code=c",
            "com.jimhillr.cahokiax://callback",
            "file:///tmp/tree.ged",
            "not a url",
        ] {
            assert!(!is_sign_in_callback(other), "{other}");
        }
    }

    #[test]
    fn android_says_sign_in_is_not_available_yet() {
        assert_eq!(unsupported_reason("android"), Some(ANDROID_NOT_YET));
        assert_eq!(unsupported_reason("ios"), None);
    }

    #[tokio::test]
    async fn mobile_without_a_client_id_the_browser_never_opens() {
        let result = begin(None, AUTH0_ISSUER, |_| {
            panic!("the browser must not open without a client ID")
        })
        .await;
        assert!(matches!(result, Err(AuthError::ConfigError(_))));
    }

    #[tokio::test]
    async fn mobile_sign_in_round_trips_through_the_custom_scheme() {
        let issuer = fake_provider(r#"{"access_token":"at","token_type":"Bearer"}"#).await;
        let mut authorize_url = String::new();
        let waiting = begin(Some("native-client"), &issuer, |url| {
            authorize_url = url.to_string();
            Ok(())
        })
        .await
        .unwrap_or_else(|e| panic!("{e}"));

        let params = query(&authorize_url);
        assert_eq!(params["client_id"], "native-client");
        assert_eq!(params["redirect_uri"], REDIRECT_URI);
        assert_eq!(params["code_challenge_method"], "S256");

        // What the OS opens the app with once Auth0 redirects.
        let callback = format!("{REDIRECT_URI}?code=c&state={}", params["state"]);
        assert!(is_sign_in_callback(&callback));
        let result = finish(waiting, &callback).await;
        // The fake provider signs no ID token, so this far means the state
        // matched and the code was exchanged for the custom-scheme redirect.
        assert!(
            matches!(result, Err(AuthError::IdTokenValidationFailed(_))),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn mobile_callback_with_the_wrong_state_is_refused() {
        let issuer = fake_provider("{}").await;
        let waiting = begin(Some("native-client"), &issuer, |_| Ok(()))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let result = finish(waiting, &format!("{REDIRECT_URI}?code=c&state=forged")).await;
        assert!(
            matches!(result, Err(AuthError::CsrfValidationFailed)),
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
    async fn mobile_cancel_in_the_browser_says_cancelled() {
        let issuer = fake_provider("{}").await;
        let waiting = begin(Some("native-client"), &issuer, |_| Ok(()))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let result = finish(waiting, &format!("{REDIRECT_URI}?error=access_denied")).await;
        assert_eq!(
            view_after(&result),
            AuthView::Failed {
                message: "Sign-in was cancelled.".to_string()
            }
        );
    }

    /// The query parameters of a URL, percent-decoded.
    fn query(url: &str) -> HashMap<String, String> {
        url.split_once('?')
            .map_or("", |(_, q)| q)
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .map(|(k, v)| (k.to_string(), decode(v)))
            .collect()
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
