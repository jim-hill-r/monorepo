# Cookbook

A Dioxus workspace for building web applications with Rust.

## Development

This workspace contains member crates for different platforms and features:

```
cookbook/
├─ cloudflare/
│  ├─ ... # Cloudflare Pages deployment configuration
├─ core/
│  ├─ ... # Core business logic and data models (library)
├─ web/
│  ├─ ... # Web specific UI/logic
```

## Core Library

The `cookbook-core` library contains shared business logic and data models:
- **Recipe struct**: Data model for recipe information including title, ingredients, instructions, timing, and tags
- Designed to be consumed by web, cloudflare, and future platform crates

## Deployment

The cookbook web application can be deployed to Cloudflare Pages using the `cloudflare` deployment project in this workspace. See the [cloudflare README](./cloudflare/README.md) for deployment instructions.

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
cast run
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
