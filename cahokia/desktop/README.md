# desktop

The Cahokia genealogy app for Windows, macOS and Linux. It is named for the
platform it targets: each Cahokia platform crate (`web`, `desktop`, `mobile`)
is named after where it runs. It renders Dioxus in a native webview and
shows the same pages as the web app, drawn by the shared `ui` crate, so a
change to a page in `ui` reaches every platform.

## Pages

| Route | What it shows |
| --- | --- |
| `/` | Home page (`ui::Hero`). |
| `/family-tree` | `ui::FamilyTreePage`, the same page and path as on the web. |

The navbar links both pages, and the appearance picker from the
[design system](../../design_system/README.md) sits at the foot of every
page.

## Sign-in

The navbar's Login button (`src/auth.rs`) signs in through
`auth_sdk::native::loopback`: it opens the system browser at Auth0 and
receives the callback on `http://127.0.0.1:<port>/callback`, for the first
free port of `DEFAULT_PORTS`. While it waits the button reads "Signing in…";
afterwards the navbar shows "Signed in as" the user's name and a Sign out
button, or a short message when sign-in failed. The session is kept in
memory only, so closing the app signs out (staying signed in is native
sign-in phase 6).

It needs the public client ID of cahokia's native Auth0 application, which
native sign-in phase 3 creates in `infra`. Until then, build with
`CAHOKIA_NATIVE_CLIENT_ID=<client id>`; without it Login says sign-in isn't
set up. Plan: [native sign-in](../../auth_sdk/docs/plans/2026-11-native-sign-in.md).

## Not done yet

- **Real sign-in end to end.** It is tested against a fake provider; it
  can't be checked against Auth0 until phase 3's application exists.
