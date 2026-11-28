# Plan: native sign-in in auth_sdk

Status: phases 1, 2 and 5 done, 2026-11-28. Asked for so cahokia's desktop and mobile
Login buttons (the `TODO`s in `cahokia/desktop/src/main.rs` and
`cahokia/mobile/src/main.rs`) can sign in with the same Auth0 tenant as the
web app.

## What native sign-in needs

A native app is a public client: it can't keep a client secret, and it
can't be redirected to like a web page. [RFC 8252](https://www.rfc-editor.org/rfc/rfc8252)
(OAuth 2.0 for Native Apps) is the standard answer, and Auth0 follows it:

1. The app opens the authorize URL in the **system browser**, never an
   embedded web view (the app could read the password, and the user's
   existing Auth0 session isn't there).
2. The provider redirects to a URI **the app owns**:
   - desktop: a **loopback** listener, `http://127.0.0.1:<port>/callback`
     (RFC 8252 section 7.3; `127.0.0.1`, not `localhost`, section 8.3);
   - mobile: a **custom URL scheme**, such as
     `com.jimhillr.cahokia://callback` (section 7.1), which Android and iOS
     route to the app.
3. **PKCE** binds the code to the app that started the sign-in, because
   another app could register the same scheme or port (section 8.1).
   auth_sdk already sends PKCE, a `state` and a nonce on the web.

So most of the work is shared with the browser flow. Only "open a URL" and
"get the callback URL back" differ per platform.

## Design

- **`flow` (phase 1):** the authorization code flow without any platform
  calls: discovery, building the authorize URL with fresh PKCE, `state` and
  nonce (`PendingLogin`), parsing a callback URL, checking `state`,
  exchanging the code, and validating the ID token. `web` now uses it too,
  so every security check exists once and is tested natively. The web
  module's tests were wasm32-only and never ran; this moves the logic where
  `cargo test` runs it.
- **`native::NativeAuthClient` (phase 1):** `new(config)` discovers the
  provider, `begin_login()` returns the authorize URL and a `PendingLogin`,
  and `complete_login(pending, callback_url)` finishes. It doesn't open a
  browser or listen on a port, so the same API serves desktop and mobile.
  It doesn't implement the `AuthProvider` trait: that trait's sync
  `login()` with no return value is shaped for a browser redirect, and
  bending it would hide the two steps a native app has to drive.
- **Stricter than the web flow was:** the callback must be to the
  configured redirect URI (scheme, host, port and path), an `error`
  response becomes `AuthError::AuthorizationDenied`, and a token response
  without an ID token is an error (this was the `require-id-token` FIX,
  now done for web and native alike).
- **Loopback ports (phase 2):** Auth0 doesn't accept a wildcard port in
  Allowed Callback URLs (see the Auth0 community threads
  [Are port wildcards supported?](https://community.auth0.com/t/are-port-wildcards-supported-pkce-for-desktop-apps/34132)
  and [Allow wildcard port as per RFC 8252](https://community.auth0.com/t/allow-wildcard-port-in-redirect-uri-as-per-rfc-8252/98409)).
  So the listener tries a short, fixed list of ports, all registered on the
  Auth0 application, and uses the first free one: `DEFAULT_PORTS`, 53682 to
  53685 (phase 2). It binds `127.0.0.1` only, answers one `GET /callback`
  with a "you can close this tab" page and a 404 to anything else, gives up
  after a timeout (five minutes by default), and closes the port as soon as
  the callback arrives. Each connection gets its own task with a short read
  timeout, so a browser's speculative preconnect that never sends can't
  block the real callback. Phase 3 checks whether
  Auth0 has since relaxed this; if it has, the list shrinks to port 0.
- **Tokens stay in memory** until phase 6. Staying signed in across
  restarts needs a refresh token in the OS keychain, which is its own
  decision (scopes, storage, revocation).

## Phases

Each phase is an issue file with the `Native sign-in` epic.

| Phase | What | Where | Issue |
| --- | --- | --- | --- |
| 1 | Shared `flow`, `NativeAuthClient`, tests with a fake provider | auth_sdk | done in this PR |
| 2 | Loopback listener and system browser for desktop | auth_sdk | done: `native::loopback` |
| 3 | A native Auth0 application for cahokia, through IaC | infra | [native-sign-in-phase-3-auth0-native-app.md](../../../infra/issues/native-sign-in-phase-3-auth0-native-app.md) |
| 4 | cahokia desktop Login button signs in | cahokia/desktop | [native-sign-in-phase-4-desktop-login.md](../../../cahokia/desktop/issues/native-sign-in-phase-4-desktop-login.md) |
| 5 | cahokia mobile Login waits for the custom URL scheme callback | cahokia/mobile | done: `mobile/src/auth.rs` (see "Mobile spike" below) |
| 5b | Register the scheme on iOS (Info.plist URL type) | cahokia/mobile | [native-sign-in-phase-5b-ios-url-scheme.md](../../../cahokia/mobile/issues/native-sign-in-phase-5b-ios-url-scheme.md) |
| 5c | Android intent filter and a JNI hand-off of the URL | cahokia/mobile | [native-sign-in-phase-5c-android-intent.md](../../../cahokia/mobile/issues/native-sign-in-phase-5c-android-intent.md) |
| 6 | Stay signed in: refresh token in the OS keychain | auth_sdk | [native-sign-in-phase-6-stay-signed-in.md](../../issues/native-sign-in-phase-6-stay-signed-in.md) |

Phase 3 is a credential change (a new Auth0 application and its grants), so
it applies only after Jim approves its run in the `infra-credentials`
environment ([decision 0010](../../../standards/docs/decisions/0010-credential-second-approval.md)).
The application's client ID is public, like the web app's, so no secret is
made or stored. Phase 4 can be built and tested against the fake provider
before phase 3 lands, but can't be checked end to end until it does.

## Mobile spike (phase 5, 2026-11-28)

How Dioxus 0.7.3 mobile hands the app a URL it was opened with, read from
the tao 0.34.5, dioxus-desktop 0.7.3 and dioxus-cli 0.7.3 sources:

- **The event:** tao raises `Event::Opened { urls }` when the OS opens the
  app with a URL. On iOS it comes from `application:openURL:options:` (and
  universal links); on macOS from `application:openURLs:`. On **Android
  tao raises nothing**: its Android event loop never reads the activity's
  intent.
- **Reaching it:** Dioxus mobile is dioxus-desktop underneath
  (`dioxus::mobile` re-exports it), so
  `Config::with_custom_event_handler` sees every tao event. cahokia mobile
  forwards `Opened` URLs to `auth::INBOX`, which passes a callback to the
  one sign-in waiting for it, and drops any other URL.
- **Registering the scheme:** dx generates Info.plist and
  AndroidManifest.xml from templates with no URL scheme setting, but
  `[application]` in `Dioxus.toml` takes `ios_info_plist`,
  `android_manifest` and `android_main_activity` files that **replace**
  the generated ones (not merge, whatever the doc comment says). iOS needs
  only the plist (phase 5b). Android needs the manifest's intent filter and
  a `MainActivity.kt` that hands the intent's URL to Rust over JNI (phase
  5c), which needs `#[unsafe(no_mangle)]` and so an exception to the
  workspace's `unsafe_code = "forbid"`.

So phase 5 shipped the platform-free half, tested on the host against the
fake provider: the inbox, the sign-in that waits on it (with a timeout and
a Cancel button, since a user who closes the browser sends no callback),
and the Login controls. Login says sign-in isn't set up until phase 3's
client ID is built in, and on a phone it can't finish until 5b or 5c
registers the scheme. The redirect URI is
`com.jimhillr.cahokia://callback`; phase 3 must list exactly that.

## Cost

$0. Auth0's free plan covers native applications and the same monthly
active users; one more application doesn't change the tier. No new vendor.
Phase 2 adds the `webbrowser` crate (open source) to open the system
browser. Claude Code effort: phases 2, 4 and 6 are each about one thread;
phases 3 and 5 are smaller in code but wait on Jim (approval) and on a
spike into Dioxus mobile deep links.

## Risks

- **Another local app grabs the port first** and receives the callback.
  PKCE means it can't exchange the code, and `state` must match, so the
  worst case is a failed sign-in (RFC 8252 section 8.3). The listener binds
  `127.0.0.1` only, never all interfaces.
- **Custom scheme hijacking on mobile**: same PKCE protection. Claimed
  `https` app links are stronger and a later option.
- **Embedded web views**: the API only returns a URL; the docs say to open
  it in the system browser.
