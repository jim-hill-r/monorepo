# OAuth2 vs OpenID Connect Crate Research

**Date:** 2025-12-29  
**Author:** GitHub Copilot Agent  
**Related Issue:** Research openidconnect crate to see if it is a better choice compared with oauth2 crate

## Executive Summary

After thorough research, **the `openidconnect` crate is recommended** for the auth_sdk project as it provides a higher-level abstraction specifically designed for OpenID Connect (OIDC) flows, which is what Auth0 and most modern authentication providers use. The `openidconnect` crate builds on top of `oauth2`, adding OIDC-specific features while maintaining the same underlying security and implementation quality.

## Background

The auth_sdk currently uses the `oauth2` crate (v5.0.0) to implement authentication flows targeting Auth0 APIs. The codebase uses:
- Basic OAuth2 authorization code flow with PKCE
- CSRF token validation
- Web-based authentication for WebAssembly targets
- Token exchange functionality

## Crate Comparison

### OAuth2 Crate (v5.0.0)

**Repository:** https://github.com/ramosbugs/oauth2-rs  
**Documentation:** https://docs.rs/oauth2/5.0.0  
**License:** MIT OR Apache-2.0  
**Rust Version:** 1.65+

**Features:**
- Pure OAuth2 implementation (RFC 6749)
- Extensible and strongly-typed
- Support for multiple HTTP clients (reqwest, curl, ureq)
- PKCE support (RFC 7636)
- Custom token types and endpoints
- Well-maintained, active development

**Pros:**
- Lower-level, more flexible for custom OAuth2 flows
- Smaller dependency footprint
- Direct control over all OAuth2 parameters

**Cons:**
- Does not include OpenID Connect specific features
- Requires manual implementation of OIDC features (ID tokens, userinfo endpoint, etc.)
- No built-in ID token validation
- No built-in discovery document support

### OpenID Connect Crate (v4.0.1)

**Repository:** https://github.com/ramosbugs/openidconnect-rs  
**Documentation:** https://docs.rs/openidconnect/4.0.1  
**License:** MIT  
**Rust Version:** 1.65+

**Features:**
- Full OpenID Connect implementation (builds on oauth2 crate)
- ID token validation and parsing
- Discovery document support (auto-configuration from .well-known/openid-configuration)
- Userinfo endpoint integration
- Support for all standard OIDC claims
- Nonce validation for security
- Same HTTP client flexibility (inherits from oauth2)

**Pros:**
- Higher-level abstraction specifically for OIDC
- Built-in ID token JWT validation
- Automatic endpoint discovery
- Better alignment with Auth0 and modern identity providers
- More secure by default (includes nonce validation, ID token signature verification)
- Built on top of oauth2, so same underlying quality and maintenance
- Same maintainer as oauth2 (ramosbugs)

**Cons:**
- Slightly larger dependency (includes oauth2 + OIDC features)
- May include features not needed if only using basic OAuth2

## Security Considerations

### Current Implementation (OAuth2)
The current implementation includes:
- ✅ CSRF protection (state parameter)
- ✅ PKCE for authorization code flow
- ✅ No redirect following (SSRF protection)
- ❌ No ID token validation
- ❌ No nonce validation
- ❌ Manual user info retrieval

### With OpenID Connect
Moving to openidconnect would add:
- ✅ Automatic ID token JWT signature verification
- ✅ Built-in nonce validation (prevents replay attacks)
- ✅ Claims validation (iss, aud, exp, etc.)
- ✅ Standardized userinfo endpoint integration
- ✅ Type-safe access to standard OIDC claims

## Auth0 Compatibility

Auth0 is an **OpenID Connect provider** that supports OAuth2 as a subset. Key points:
- Auth0 returns ID tokens (JWT) in addition to access tokens
- Auth0 exposes a discovery document at `https://{domain}/.well-known/openid-configuration`
- Auth0 recommends using OIDC for authentication use cases
- The current implementation ignores the ID token returned by Auth0

## Migration Considerations

### Breaking Changes
- API surface will change (different client types, different methods)
- Need to update all authentication code
- Type signatures for tokens and providers will change

### Benefits
- Simplified code (discovery document removes manual endpoint configuration)
- Better security (automatic ID token validation)
- More idiomatic for OIDC providers like Auth0
- Access to user claims directly from ID token without extra API call

### Code Example Comparison

**Current (oauth2):**
```rust
use oauth2::{
    AuthUrl, ClientId, TokenUrl, RedirectUrl, BasicClient
};

let client = BasicClient::new(ClientId::new(config.client_id))
    .set_auth_uri(AuthUrl::new(config.auth_url)?)
    .set_token_uri(TokenUrl::new(config.token_url)?)
    .set_redirect_uri(RedirectUrl::new(config.redirect_url)?);
```

**With openidconnect:**
```rust
use openidconnect::{
    IssuerUrl, ClientId, RedirectUrl,
    core::{CoreClient, CoreProviderMetadata}
};

// Automatic discovery
let provider_metadata = CoreProviderMetadata::discover_async(
    IssuerUrl::new(config.issuer_url)?,
    async_http_client
).await?;

let client = CoreClient::from_provider_metadata(
    provider_metadata,
    ClientId::new(config.client_id),
    None // No client secret for public clients
).set_redirect_uri(RedirectUrl::new(config.redirect_url)?);

// ID token is automatically validated
let token_response = client
    .exchange_code(code)
    .set_pkce_verifier(verifier)
    .request_async(async_http_client)
    .await?;

let id_token = token_response.id_token().ok_or(...)?;
let claims = id_token.claims(&verifier, &nonce)?; // Validated!
let user_email = claims.email();
```

## Recommendation

**Switch to the `openidconnect` crate** for the following reasons:

1. **Better Alignment:** Auth0 is an OIDC provider; using an OIDC client is the right abstraction level
2. **Enhanced Security:** Automatic ID token validation, nonce validation, and claims verification
3. **Simplified Configuration:** Discovery document support reduces configuration boilerplate
4. **User Information:** Direct access to user claims from ID token without additional API calls
5. **Same Quality:** Built and maintained by the same author as oauth2, same underlying HTTP client support
6. **Future-Proof:** Most modern identity providers are moving to OIDC as the standard

## Implementation Path

1. Add `openidconnect` dependency to Cargo.toml
2. Update `ProviderConfig` to use issuer URL instead of individual endpoint URLs
3. Refactor `WebAuthProvider::new()` to use discovery and CoreClient
4. Update login flow to include nonce generation and validation
5. Update token exchange to validate ID tokens
6. Extract user information from ID token claims
7. Update tests to cover ID token validation
8. Remove oauth2 dependency (it's now transitive through openidconnect)
9. Update documentation

## Conclusion

The `openidconnect` crate is the better choice for auth_sdk as it:
- Provides the appropriate abstraction level for OIDC providers like Auth0
- Improves security posture with built-in ID token validation
- Simplifies code with discovery and standard OIDC features
- Maintains the same quality and maintenance as oauth2

The migration effort is justified by the security improvements and better alignment with modern authentication standards.

## References

- [OAuth 2.0 RFC 6749](https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [Auth0 OIDC Documentation](https://auth0.com/docs/authenticate/protocols/openid-connect-protocol)
- [oauth2-rs GitHub](https://github.com/ramosbugs/oauth2-rs)
- [openidconnect-rs GitHub](https://github.com/ramosbugs/openidconnect-rs)
