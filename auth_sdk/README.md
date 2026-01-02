# Auth SDK

Generic library for producing auth flows in applications.

Currently targetting auth0 API's, but is designed to be as generic as possible in order to make porting to other oauth providers reasonable.

## Platform Support

This library is designed to support multiple platforms:
- **Web (WebAssembly)**: Full OAuth2 implementation with browser-based authentication flow
- **Desktop/Mobile**: Platform-specific implementations (coming soon)

The web-specific dependencies (like `web-sys`) are only included when compiling for WebAssembly targets.

## Configuration

The `ProviderConfig` struct supports two configuration modes:

### OAuth2 Mode (Current)
Explicitly specify authentication endpoints:
```rust
ProviderConfig {
    client_id: "your_client_id".to_string(),
    auth_url: "https://example.com/authorize".to_string(),
    token_url: "https://example.com/token".to_string(),
    redirect_url: "https://example.com/callback".to_string(),
    issuer_url: None,
}
```

### OIDC Discovery Mode (Recommended for OIDC providers)
Provide an issuer URL for automatic endpoint discovery:
```rust
ProviderConfig {
    client_id: "your_client_id".to_string(),
    auth_url: "https://example.com/authorize".to_string(), // Used as fallback
    token_url: "https://example.com/token".to_string(), // Used as fallback
    redirect_url: "https://example.com/callback".to_string(),
    issuer_url: Some("https://example.com".to_string()), // Enable OIDC discovery
}
```

When `issuer_url` is provided, future implementations can use OIDC discovery to automatically fetch endpoints from `{issuer_url}/.well-known/openid-configuration`.

## Documentation

- [OAuth2 vs OpenID Connect Research](docs/oauth2_vs_openidconnect_research.md) - Comprehensive comparison and recommendation for migrating to the `openidconnect` crate
