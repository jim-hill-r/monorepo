# mobile

The Cahokia genealogy app for iOS and Android. It is named for the platform
it targets: each Cahokia platform crate (`web`, `desktop`, `mobile`) is
named after where it runs. It renders Dioxus in the phone's webview and
shows the same pages as the web app, drawn by the shared `ui` crate, so a
change to a page in `ui` reaches every platform.

## Pages

| Route | What it shows |
| --- | --- |
| `/` | Home page (`ui::Hero`). |
| `/family-tree` | `ui::FamilyTreePage`, the same page and path as on the web. Pan and pinch zoom work with touch. |

The navbar links both pages, and the appearance picker from the
[design system](../../design_system/README.md) sits at the foot of every
page.

## Sign-in

**Login** in the navbar opens Auth0 in the phone's browser (never a web
view, RFC 8252). Auth0 redirects to the app's own URL scheme,
`com.jimhillr.cahokia://callback`, and the phone opens the app with it,
which finishes the sign-in with PKCE through
[`auth_sdk::native`](../../auth_sdk/README.md). While it waits the navbar
says to finish in the browser and offers **Cancel**; afterwards it shows
**Signed in as** the user's name and **Sign out**, or a short message and
Login again. The session is kept in memory only. The code is in
`src/auth.rs`.

- **iOS:** `ios/Info.plist` registers the scheme, and `Dioxus.toml` points
  dx at it. iOS hands the URL to the app as tao's `Event::Opened`.
- **Android:** not yet. tao doesn't pass the opened URL on, so Login says
  sign-in isn't available
  ([issue](docs/issues/native-sign-in-phase-5b-android-callback.md)).
- **Client ID:** the native Auth0 application doesn't exist yet
  ([phase 3](../../infra/docs/issues/native-sign-in-phase-3-auth0-native-app.md)).
  Until it does, build with `CAHOKIA_NATIVE_CLIENT_ID=<id>`; without one
  Login says sign-in isn't set up in this build.

## Not done yet

- **Store builds.** Nothing builds or signs an iOS or Android package yet.
- **Staying signed in** across restarts (native sign-in phase 6).
