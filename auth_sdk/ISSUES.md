# Priority Issues

TODO (agent-generated): Add openidconnect dependency to Cargo.toml while keeping oauth2 temporarily for incremental migration
TODO (agent-generated): Update ProviderConfig to support issuer URL for OIDC discovery alongside existing endpoint URLs
TODO (agent-generated): Add nonce generation and storage to AppState structure
TODO (agent-generated): Implement discovery document fetching and caching in WebAuthProvider
TODO (agent-generated): Create wrapper types for openidconnect CoreClient to maintain backward compatibility
TODO (agent-generated): Update login flow to use openidconnect with nonce validation
TODO (agent-generated): Update token exchange to validate ID tokens using openidconnect
TODO (agent-generated): Extract user information from ID token claims instead of separate API calls
TODO (agent-generated): Update tests to cover ID token validation and nonce handling
TODO (agent-generated): Remove oauth2 dependency once migration is complete (it becomes transitive through openidconnect)
TODO (agent-generated): Update documentation to reflect OIDC usage patterns

# Backlog
