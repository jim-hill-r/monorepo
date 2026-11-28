# Auth SDK

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/auth_sdk.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

## auth_sdk

Generic library for producing auth flows in applications.

Currently targetting auth0 API's, but is designed to be as generic as possible in order to make porting to other oauth providers reasonable.

## Platform Support

What works today:
- **Web (WebAssembly)**: `web::WebAuthProvider`, the full browser redirect flow.
- **Desktop and mobile (native)**: `native::NativeAuthClient`, the platform-free
  half of RFC 8252 sign-in. `begin_login()` gives the authorize URL to open in
  the system browser, and `complete_login()` finishes from the callback URL
  (a loopback `http://127.0.0.1:<port>/callback` or a custom URL scheme).
- **Desktop**: `native::loopback::sign_in` does the whole sign-in. It binds
  `127.0.0.1` on the first free port of `LoopbackConfig::ports` (default
  53682 to 53685, each an Allowed Callback URL on the Auth0 native app,
  since Auth0 has no wildcard ports), opens the system browser, waits for
  one `GET /callback` (five minutes by default) and finishes the sign-in.
  Mobile apps receive the callback through a custom URL scheme themselves
  for now ([plan](docs/plans/2026-11-native-sign-in.md)).

Both share `flow`, which holds every security check (PKCE, `state`, a required
ID token and its nonce), so it is tested natively with `cargo test`. The
web-specific dependencies (like `web-sys`) are only included when compiling
for WebAssembly targets.

## Security Features

The auth SDK implements multiple layers of security for OAuth2/OIDC flows:

- **PKCE (Proof Key for Code Exchange)**: Protects against authorization code interception attacks
- **CSRF Protection**: Uses state parameter validation to prevent cross-site request forgery
- **Nonce Validation**: Validates ID token nonce to prevent replay attacks (OIDC). A token response without an ID token is rejected, so this check can't be skipped
- **Callback Check**: A native callback must be to the configured redirect URI (scheme, host, port and path), and an `error` response becomes `AuthError::AuthorizationDenied`
- **ID Token Verification**: Automatically validates ID token signatures and claims (OIDC)
- **SSRF Protection**: Disables HTTP redirects on non-wasm32 targets to prevent server-side request forgery

## Configuration

The `ProviderConfig` struct supports two configuration modes:

### Auth0 (Recommended for Auth0 tenants)
Build the config from the tenant domain. `ProviderConfig::auth0` derives the
issuer (with the trailing slash Auth0 publishes), authorize and token URLs, so
they can't drift apart:
```rust
use auth_sdk::provider::ProviderConfig;

let config = ProviderConfig::auth0(
    "your-tenant.us.auth0.com", // an https:// prefix or trailing slash is fine too
    "your_client_id",
    "https://example.com/",
)?;
```
`auth_sdk::auth0::Auth0Urls::from_domain` gives the URLs on their own, and
`Auth0Urls::origin()` the `https://<domain>` form a CSP `connect-src` needs.
A domain with another scheme, a path, a port or a query is a `ConfigError`.

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
    issuer_url: Some("https://example.com/".to_string()), // Enable OIDC discovery
}
```

When `issuer_url` is provided, the WebAuthProvider will automatically fetch the OIDC discovery document from `{issuer_url}/.well-known/openid-configuration` and use the discovered endpoints. The provider metadata is cached for the lifetime of the WebAuthProvider instance.

**Important**: The issuer URL must match exactly (including trailing slashes) with the `issuer` field returned in the OIDC discovery document. For Auth0 domains, the issuer URL includes a trailing slash (e.g., `https://your-domain.auth0.com/`).

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

- [Native sign-in plan](docs/plans/2026-11-native-sign-in.md) - desktop and mobile sign-in, in phases
- [OAuth2 vs OpenID Connect Research](docs/oauth2_vs_openidconnect_research.md) - Comprehensive comparison and recommendation for migrating to the `openidconnect` crate
