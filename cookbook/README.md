# Cookbook

A Dioxus workspace for building web applications with Rust.

## Development

This workspace contains member crates for different platforms and features:

```
cookbook/
├─ web/
│  ├─ ... # Web specific UI/logic
```

## Platform crates

Each platform crate contains the entry point for the platform, and any assets, components and dependencies that are specific to that platform.

### Serving Your App

Navigate to the platform crate of your choice:
```bash
cd web
```

and serve:

```bash
dx serve
```

Or use the Cast CLI from the workspace root:

```bash
cast dev
```

## Building

To build the entire workspace:

```bash
cargo build
```

To check the workspace without building:

```bash
cargo check
```

## Running Tests

To run all tests in the workspace:

```bash
cargo test
```
