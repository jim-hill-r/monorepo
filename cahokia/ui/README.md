# ui

Shared UI components for the Cahokia genealogy application. This crate holds the Dioxus components and styles that are reused across the `web`, `desktop` and `mobile` platform crates, so each platform renders the same look and feel.

## Components

| Component | Description | Used by |
|-----------|-------------|---------|
| `Hero` | The landing banner with the Cahokia title and tagline | web, desktop, mobile |
| `Navbar` | A navigation bar wrapper that renders its children | web, desktop, mobile |
| `Sidebar` | A collapsible sidebar driven by an `is_open: Signal<bool>` | web |

## Structure

```
ui/
├─ assets/
│  ├─ header.svg   # Header image
│  ├─ styling/     # Per-component CSS (hero.css, navbar.css, sidebar.css)
├─ src/
│  ├─ lib.rs       # Re-exports the public components
│  ├─ hero.rs
│  ├─ navbar.rs
│  ├─ sidebar.rs
├─ Cargo.toml
```

## Usage

The crate is available to the other workspace members as `ui`:

```rust
use ui::{Hero, Navbar};
```
