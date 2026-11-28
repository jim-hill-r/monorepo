# Contributing to desktop

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands").
`desktop` is a member of the cahokia Cargo workspace, so its tasks run from
`cahokia/`:

```bash
cd cahokia
mise install
```

On Linux the webview also needs GTK and WebKit system packages:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev
```

### Build

`desktop` is not in the workspace's `default-members`, so name it:

```bash
cargo build -p desktop
```

To run the app with hot reload, use the Dioxus CLI from the crate folder:

```bash
cd desktop
dx serve
```

### Test

To run its tests (route parsing, server-side renders of each view, and
sign-in against a fake OpenID provider on 127.0.0.1):

```bash
cargo test -p desktop
```

`mise run ci` in `cahokia/` checks this crate too (clippy, build and
tests), and PR CI installs the GTK and WebKit packages for it. Where
`webkit2gtk-4.1` isn't installed, `mise run ci` checks only `core` and `ui`
and says so.

### Where code goes

- Put pages and components every platform shows in `ui`, and keep only the
  desktop entry point, routes and desktop-only views here.
- Keep a route's path the same as the web app's, and test it in
  `src/main.rs`.
