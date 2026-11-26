# desktop

The desktop platform crate for the Cahokia genealogy application. It is the Dioxus desktop entry point: it defines the desktop routes and layout, and renders views using shared components from the `ui` crate.

## Structure

```
desktop/
├─ assets/      # Desktop-specific assets (e.g. main.css)
├─ src/
│  ├─ main.rs   # Entry point; defines routes and the desktop navbar layout
│  ├─ views/    # Views rendered for each route
├─ Cargo.toml   # Desktop-specific dependencies
```

## Running

```bash
cd cahokia/desktop
dx serve
```

Desktop authentication is not yet implemented; it is waiting on `auth_sdk` support for desktop platforms.
