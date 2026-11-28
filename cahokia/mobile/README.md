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

## Not done yet

- **Store builds.** Nothing builds or signs an iOS or Android package yet.
