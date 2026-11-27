# Contributing to mobile

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands").
`mobile` is a member of the cahokia Cargo workspace, so its tasks run from
`cahokia/`:

```bash
cd cahokia
mise install
```

On Linux, building for the host (which the tests do) also needs GTK and
WebKit system packages:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev
```

To run it on a device or simulator you also need the Android SDK and NDK
or Xcode, as the Dioxus mobile guide describes.

### Build

`mobile` is not in the workspace's `default-members`, so name it:

```bash
cargo build -p mobile
```

To run the app with hot reload, use the Dioxus CLI from the crate folder:

```bash
cd mobile
dx serve --platform android   # or: --platform ios
```

### Test

To run its tests (route parsing and server-side renders of each view):

```bash
cargo test -p mobile
```

`mise run ci` in `cahokia/` checks this crate too (clippy, build and
tests), and PR CI installs the GTK and WebKit packages for it. Where
`webkit2gtk-4.1` isn't installed, `mise run ci` checks only `core` and `ui`
and says so.

### Where code goes

- Put pages and components every platform shows in `ui`, and keep only the
  mobile entry point, routes and mobile-only views here.
- Keep a route's path the same as the web app's, and test it in
  `src/main.rs`.
