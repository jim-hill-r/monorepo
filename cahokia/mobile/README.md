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

The navbar links both pages and has the Login button (see "Sign-in"
below), and the appearance picker from the
[design system](../../design_system/README.md) sits at the foot of every
page.

## Sign-in

Login opens the phone's browser at Auth0's sign-in page, and Auth0 sends
the browser back to `com.jimhillr.cahokia://callback`. The OS opens the
app with that URL, `main` passes it to `auth::INBOX`, and the waiting
sign-in finishes it (`src/auth.rs`, native sign-in phase 5 in
[the plan](../../auth_sdk/docs/plans/2026-11-native-sign-in.md)). While it
waits, the navbar says to finish in the browser and offers Cancel; it gives
up after five minutes. The session stays in memory until Sign out or the
app closes.

## Not done yet

- **Sign-in on a phone.** A build needs the native Auth0 client ID
  (`CAHOKIA_NATIVE_CLIENT_ID`, from native sign-in phase 3) or Login says
  sign-in isn't set up, and the OS doesn't open the app with the callback
  until the scheme is registered (phase 5b for iOS, 5c for Android).

- **Store builds.** Nothing builds or signs an iOS or Android package yet.
