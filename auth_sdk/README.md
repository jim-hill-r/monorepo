# Auth SDK

Generic library for producing auth flows in applications.

Currently targetting auth0 API's, but is designed to be as generic as possible in order to make porting to other oauth providers reasonable.

## Platform Support

This library is designed to support multiple platforms:
- **Web (WebAssembly)**: Full OpenID Connect implementation with browser-based authentication flow
- **Desktop/Mobile**: Platform-specific implementations (coming soon)

The web-specific dependencies (like `web-sys`) are only included when compiling for WebAssembly targets.

## Security Features

The auth SDK implements multiple layers of security for OAuth2/OIDC flows:

- **PKCE (Proof Key for Code Exchange)**: Protects against authorization code interception attacks
- **CSRF Protection**: Uses state parameter validation to prevent cross-site request forgery
- **Nonce Validation**: Validates ID token nonce to prevent replay attacks (OIDC)
- **ID Token Verification**: Automatically validates ID token signatures and claims (OIDC)
- **SSRF Protection**: Disables HTTP redirects on non-wasm32 targets to prevent server-side request forgery

## Configuration

The `ProviderConfig` struct supports two configuration modes:

### OAuth2 Mode
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

### OIDC Discovery Mode (Recommended)
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

When `issuer_url` is provided, the WebAuthProvider will automatically fetch the OIDC discovery document from `{issuer_url}/.well-known/openid-configuration` and use the discovered endpoints. The provider metadata is cached for the lifetime of the WebAuthProvider instance.

## User Information

The SDK automatically extracts user information from the ID token claims returned during authentication. The `user()` method on the `AuthProvider` trait returns a `User` struct containing standard OIDC claims:

- `sub` (subject) - Unique user identifier (required)
- `name` - Full name (optional)
- `email` - Email address (optional)
- `email_verified` - Email verification status (optional)
- `picture` - Profile picture URL (optional)
- `preferred_username` - Preferred username (optional)

This approach eliminates the need for separate API calls to fetch user information, as recommended by OIDC best practices. The user information is extracted directly from the validated ID token claims during the authentication flow.

## Documentation

- [OAuth2 vs OpenID Connect Research](docs/oauth2_vs_openidconnect_research.md) - Comprehensive comparison and recommendation for migrating to the `openidconnect` crate
