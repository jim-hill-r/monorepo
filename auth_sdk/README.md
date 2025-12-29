# Auth SDK

Generic library for producing auth flows in applications.

Currently targetting auth0 API's, but is designed to be as generic as possible in order to make porting to other oauth providers reasonable.

## Platform Support

This library is designed to support multiple platforms:
- **Web (WebAssembly)**: Full OAuth2 implementation with browser-based authentication flow
- **Desktop/Mobile**: Platform-specific implementations (coming soon)

The web-specific dependencies (like `web-sys`) are only included when compiling for WebAssembly targets.

## Documentation

- [OAuth2 vs OpenID Connect Research](docs/oauth2_vs_openidconnect_research.md) - Comprehensive comparison and recommendation for migrating to the `openidconnect` crate
