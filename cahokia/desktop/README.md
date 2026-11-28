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

## Not done yet

- **Sign-in.** The Login button does nothing until `auth_sdk` supports
  desktop platforms (a `TODO` in `src/main.rs`).
