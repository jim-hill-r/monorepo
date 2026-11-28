//! Desktop sign-in through a loopback redirect (RFC 8252 section 7.3).
//!
//! [`sign_in`] binds `127.0.0.1` on the first free port of a fixed list,
//! opens the authorize URL in the system browser, waits for the browser to
//! be sent to `http://127.0.0.1:<port>/callback`, and finishes the sign-in
//! with [`NativeAuthClient::complete_login`].
//!
//! The ports are a fixed list because Auth0 doesn't accept a wildcard port
//! in Allowed Callback URLs, so every port in [`LoopbackConfig::ports`] must
//! be registered on the provider's native application.
//!
//! ```no_run
//! # async fn example() -> Result<(), auth_sdk::provider::AuthError> {
//! use auth_sdk::native::loopback::{LoopbackConfig, sign_in};
//! use auth_sdk::provider::ProviderConfig;
//!
//! let signed_in = sign_in(
//!     ProviderConfig {
//!         client_id: "public-client-id".to_string(),
//!         auth_url: String::new(),
//!         token_url: String::new(),
//!         // Replaced with the loopback address of the port that was free.
//!         redirect_url: String::new(),
//!         issuer_url: Some("https://example.us.auth0.com/".to_string()),
//!     },
//!     &LoopbackConfig::default(),
//! )
//! .await?;
//! println!("signed in as {}", signed_in.user.sub);
//! # Ok(())
//! # }
//! ```

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};

use super::{NativeAuthClient, SignedIn};
use crate::provider::{AuthError, ProviderConfig};

/// The ports the listener tries, in order. Each must be an Allowed Callback
/// URL (`http://127.0.0.1:<port>/callback`) of the native application.
pub const DEFAULT_PORTS: [u16; 4] = [53682, 53683, 53684, 53685];

/// The path the provider redirects the browser to.
pub const CALLBACK_PATH: &str = "/callback";

/// How long [`sign_in`] waits for the browser by default.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_mins(5);

/// How long one browser connection may take to send its request line, so a
/// connection that never sends (a browser's speculative preconnect) can't
/// hold anything up.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// The most of a request the listener reads; the request line is all it
/// needs.
const MAX_REQUEST_BYTES: usize = 16 * 1024;

const CLOSE_PAGE: &str = "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<title>Signed in</title></head><body style=\"font-family: system-ui, sans-serif; \
text-align: center; margin-top: 4rem\"><h1>Sign-in received</h1>\
<p>You can close this tab and go back to the app.</p></body></html>";

/// Where the loopback listener listens and for how long.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopbackConfig {
    /// Ports to try in order; the first free one is used. Port `0` lets the
    /// OS choose, which only works with a provider that accepts any port.
    pub ports: Vec<u16>,
    /// How long to wait for the callback after opening the browser.
    pub timeout: Duration,
}

impl Default for LoopbackConfig {
    fn default() -> Self {
        Self {
            ports: DEFAULT_PORTS.to_vec(),
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

/// A listener on `127.0.0.1` waiting for one callback.
#[derive(Debug)]
pub struct Loopback {
    listener: TcpListener,
    port: u16,
}

impl Loopback {
    /// Bind `127.0.0.1` (never all interfaces) on the first port of `ports`
    /// that is free.
    ///
    /// # Errors
    ///
    /// [`AuthError::ConfigError`] for an empty list, and
    /// [`AuthError::LoopbackError`] when no port in it can be bound.
    pub async fn bind(ports: &[u16]) -> Result<Self, AuthError> {
        if ports.is_empty() {
            return Err(AuthError::ConfigError(
                "no loopback ports configured".to_string(),
            ));
        }
        let mut last_error = String::new();
        for &port in ports {
            match TcpListener::bind(("127.0.0.1", port)).await {
                Ok(listener) => {
                    let port = listener
                        .local_addr()
                        .map_err(|e| AuthError::LoopbackError(e.to_string()))?
                        .port();
                    return Ok(Self { listener, port });
                }
                Err(e) => {
                    tracing::debug!("loopback port {port} is taken: {e}");
                    last_error = format!("port {port}: {e}");
                }
            }
        }
        Err(AuthError::LoopbackError(format!(
            "no free port in {ports:?} ({last_error})"
        )))
    }

    /// The port this listener is on.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The redirect URI to sign in with: `http://127.0.0.1:<port>/callback`.
    #[must_use]
    pub fn redirect_url(&self) -> String {
        format!("http://127.0.0.1:{}{CALLBACK_PATH}", self.port)
    }

    /// Wait for one `GET /callback` and return its full URL.
    ///
    /// That request is answered with a short "you can close this tab" page;
    /// any other request gets a 404 and the listener keeps waiting. The
    /// listener closes when this returns.
    ///
    /// # Errors
    ///
    /// [`AuthError::SignInTimedOut`] when no callback arrives within
    /// `wait`, and [`AuthError::LoopbackError`] when accepting a connection
    /// fails.
    pub async fn wait_for_callback(self, wait: Duration) -> Result<String, AuthError> {
        let deadline = Instant::now() + wait;
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                accepted = self.listener.accept() => {
                    let (stream, _) =
                        accepted.map_err(|e| AuthError::LoopbackError(e.to_string()))?;
                    connections.spawn(answer(stream));
                }
                Some(answered) = connections.join_next(), if !connections.is_empty() => {
                    if let Ok(Some(target)) = answered {
                        return Ok(format!("http://127.0.0.1:{}{target}", self.port));
                    }
                }
                () = sleep_until(deadline) => {
                    return Err(AuthError::SignInTimedOut(format!(
                        "no callback within {} seconds",
                        wait.as_secs()
                    )));
                }
            }
        }
    }
}

/// Sign in on desktop: open the system browser and receive the callback on
/// a loopback port.
///
/// `config.redirect_url` is replaced with the loopback address of the port
/// that was free.
///
/// # Errors
///
/// Any error of [`Loopback::bind`], [`NativeAuthClient::new`],
/// [`Loopback::wait_for_callback`] or [`NativeAuthClient::complete_login`],
/// and [`AuthError::BrowserApiError`] when the system browser can't be
/// opened.
pub async fn sign_in(
    config: ProviderConfig,
    loopback: &LoopbackConfig,
) -> Result<SignedIn, AuthError> {
    sign_in_with(config, loopback, open_system_browser).await
}

/// [`sign_in`] with another way to open the authorize URL, for tests or an
/// app that wants to show the URL as well.
///
/// # Errors
///
/// As [`sign_in`], plus whatever `open` returns.
pub async fn sign_in_with(
    mut config: ProviderConfig,
    loopback: &LoopbackConfig,
    open: impl FnOnce(&str) -> Result<(), AuthError>,
) -> Result<SignedIn, AuthError> {
    let listener = Loopback::bind(&loopback.ports).await?;
    config.redirect_url = listener.redirect_url();
    let client = NativeAuthClient::new(config).await?;
    let request = client.begin_login();
    open(&request.authorize_url)?;
    let callback_url = listener.wait_for_callback(loopback.timeout).await?;
    client.complete_login(request.pending, &callback_url).await
}

/// Open `url` in the system browser (never an embedded web view).
///
/// # Errors
///
/// [`AuthError::BrowserApiError`] when no browser can be opened.
pub fn open_system_browser(url: &str) -> Result<(), AuthError> {
    webbrowser::open(url).map_err(|e| AuthError::BrowserApiError(e.to_string()))
}

/// Read one request and answer it. Returns the request target when it was
/// a `GET` of [`CALLBACK_PATH`].
async fn answer(mut stream: TcpStream) -> Option<String> {
    let request_line = timeout(REQUEST_TIMEOUT, read_request_line(&mut stream))
        .await
        .ok()??;
    let target = callback_target(&request_line);
    let response = if target.is_some() {
        response("200 OK", CLOSE_PAGE)
    } else {
        response("404 Not Found", "Not found")
    };
    // The callback is already received; a browser that hung up early
    // doesn't change that.
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;
    target
}

/// Read until the end of the headers (or [`MAX_REQUEST_BYTES`]) and return
/// the request line.
async fn read_request_line(stream: &mut TcpStream) -> Option<String> {
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0; 1024];
    while !buf.windows(4).any(|w| w == b"\r\n\r\n") && buf.len() < MAX_REQUEST_BYTES {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let text = String::from_utf8_lossy(&buf);
    text.lines().next().map(str::to_string)
}

/// The target of a `GET /callback...` request line, or `None` for anything
/// else.
fn callback_target(request_line: &str) -> Option<String> {
    let mut parts = request_line.split_whitespace();
    let (method, target) = (parts.next()?, parts.next()?);
    let path = target.split_once('?').map_or(target, |(path, _)| path);
    (method == "GET" && path == CALLBACK_PATH).then(|| target.to_string())
}

fn response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\n\
         content-type: text/html; charset=utf-8\r\n\
         content-length: {}\r\n\
         cache-control: no-store\r\n\
         referrer-policy: no-referrer\r\n\
         content-security-policy: default-src 'none'; style-src 'unsafe-inline'\r\n\
         connection: close\r\n\r\n{body}",
        body.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native::tests::{config, fake_provider};

    /// Send `request` to the listener and return the response.
    async fn send(port: u16, request: &str) -> String {
        let mut stream = TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        stream
            .write_all(request.as_bytes())
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        response
    }

    fn get(path: &str) -> String {
        format!("GET {path} HTTP/1.1\r\nhost: 127.0.0.1\r\n\r\n")
    }

    async fn bind_any() -> Loopback {
        Loopback::bind(&[0])
            .await
            .unwrap_or_else(|e| panic!("bind failed: {e}"))
    }

    #[tokio::test]
    async fn binds_loopback_only_with_a_callback_redirect() {
        let loopback = bind_any().await;
        let addr = loopback
            .listener
            .local_addr()
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(addr.ip().is_loopback());
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert_eq!(
            loopback.redirect_url(),
            format!("http://127.0.0.1:{}/callback", loopback.port())
        );
    }

    #[tokio::test]
    async fn falls_back_to_the_next_port_when_the_first_is_taken() {
        let taken = bind_any().await;
        let loopback = Loopback::bind(&[taken.port(), 0])
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        assert_ne!(loopback.port(), taken.port());
    }

    #[tokio::test]
    async fn every_port_taken_is_an_error() {
        let first = bind_any().await;
        let second = bind_any().await;
        let result = Loopback::bind(&[first.port(), second.port()]).await;
        assert!(
            matches!(result, Err(AuthError::LoopbackError(ref m)) if m.contains("no free port")),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn an_empty_port_list_is_a_config_error() {
        let result = Loopback::bind(&[]).await;
        assert!(matches!(result, Err(AuthError::ConfigError(_))));
    }

    #[tokio::test]
    async fn returns_the_callback_url_and_says_to_close_the_tab() {
        let loopback = bind_any().await;
        let port = loopback.port();
        let waiting = tokio::spawn(loopback.wait_for_callback(Duration::from_secs(5)));
        let response = send(port, &get("/callback?code=c&state=s")).await;
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert!(response.contains("You can close this tab"));
        assert!(response.contains("cache-control: no-store"));
        let url = waiting
            .await
            .unwrap_or_else(|e| panic!("{e}"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(
            url,
            format!("http://127.0.0.1:{port}/callback?code=c&state=s")
        );
    }

    #[tokio::test]
    async fn the_wrong_path_gets_404_and_it_keeps_waiting() {
        let loopback = bind_any().await;
        let port = loopback.port();
        let waiting = tokio::spawn(loopback.wait_for_callback(Duration::from_secs(5)));
        for request in [
            get("/favicon.ico"),
            get("/callbackx?code=c"),
            "POST /callback?code=c HTTP/1.1\r\n\r\n".to_string(),
        ] {
            let response = send(port, &request).await;
            assert!(
                response.starts_with("HTTP/1.1 404"),
                "{request}: {response}"
            );
        }
        assert!(!waiting.is_finished());
        send(port, &get("/callback?code=c&state=s")).await;
        let url = waiting
            .await
            .unwrap_or_else(|e| panic!("{e}"))
            .unwrap_or_else(|e| panic!("{e}"));
        assert!(url.ends_with("/callback?code=c&state=s"));
    }

    #[tokio::test]
    async fn a_silent_connection_does_not_block_the_callback() {
        let loopback = bind_any().await;
        let port = loopback.port();
        let waiting = tokio::spawn(loopback.wait_for_callback(Duration::from_secs(5)));
        // Like a browser's speculative preconnect: opens, never sends.
        let _silent = TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        send(port, &get("/callback?code=c&state=s")).await;
        let result = timeout(Duration::from_secs(2), waiting).await;
        assert!(matches!(result, Ok(Ok(Ok(_)))), "{result:?}");
    }

    #[tokio::test]
    async fn no_callback_in_time_is_a_timeout() {
        let loopback = bind_any().await;
        let result = loopback.wait_for_callback(Duration::from_millis(50)).await;
        assert!(matches!(result, Err(AuthError::SignInTimedOut(_))));
    }

    #[tokio::test]
    async fn the_port_closes_after_the_callback() {
        let loopback = bind_any().await;
        let port = loopback.port();
        let waiting = tokio::spawn(loopback.wait_for_callback(Duration::from_secs(5)));
        send(port, &get("/callback?code=c&state=s")).await;
        let _ = waiting.await;
        assert!(TcpStream::connect(("127.0.0.1", port)).await.is_err());
    }

    #[test]
    fn only_a_get_of_the_callback_path_is_the_callback() {
        assert_eq!(
            callback_target("GET /callback?code=c HTTP/1.1").as_deref(),
            Some("/callback?code=c")
        );
        assert_eq!(
            callback_target("GET /callback HTTP/1.1").as_deref(),
            Some("/callback")
        );
        assert_eq!(callback_target("GET /callback/x HTTP/1.1"), None);
        assert_eq!(callback_target("HEAD /callback HTTP/1.1"), None);
        assert_eq!(callback_target("GET"), None);
        assert_eq!(callback_target(""), None);
    }

    /// The whole desktop flow against the fake provider, with a "browser"
    /// that follows the authorize URL's redirect straight back.
    async fn sign_in_against(
        token_status: u16,
        token_body: &'static str,
        state: Option<&'static str>,
    ) -> Result<SignedIn, AuthError> {
        let issuer = fake_provider(token_status, token_body).await;
        let loopback = LoopbackConfig {
            ports: vec![0],
            timeout: Duration::from_secs(5),
        };
        sign_in_with(config(issuer), &loopback, |authorize_url| {
            let url =
                openidconnect::url::Url::parse(authorize_url).map_err(|_| AuthError::ParseError)?;
            let param = |name: &str| {
                url.query_pairs()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| v.into_owned())
                    .unwrap_or_default()
            };
            let redirect = openidconnect::url::Url::parse(&param("redirect_uri"))
                .map_err(|_| AuthError::ParseError)?;
            let port = redirect.port().unwrap_or_default();
            let state = state.map_or_else(|| param("state"), str::to_string);
            tokio::spawn(async move {
                send(port, &get(&format!("/callback?code=c&state={state}"))).await;
            });
            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn sign_in_reaches_the_token_exchange_with_the_callback() {
        let result = sign_in_against(
            200,
            r#"{"access_token":"at","token_type":"Bearer","expires_in":60}"#,
            None,
        )
        .await;
        // The fake provider issues no ID token, so getting this far means the
        // callback came back with the right state and the code was exchanged.
        assert!(
            matches!(result, Err(AuthError::IdTokenValidationFailed(ref m)) if m == "no ID token in the token response"),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn sign_in_rejects_a_forged_state() {
        let result = sign_in_against(500, "{}", Some("forged")).await;
        assert!(
            matches!(result, Err(AuthError::CsrfValidationFailed)),
            "{result:?}"
        );
    }

    #[tokio::test]
    async fn a_browser_that_cannot_open_fails_the_sign_in() {
        let issuer = fake_provider(200, "{}").await;
        let loopback = LoopbackConfig {
            ports: vec![0],
            timeout: Duration::from_secs(5),
        };
        let result = sign_in_with(config(issuer), &loopback, |_| {
            Err(AuthError::BrowserApiError("no browser".to_string()))
        })
        .await;
        assert!(matches!(result, Err(AuthError::BrowserApiError(_))));
    }
}
