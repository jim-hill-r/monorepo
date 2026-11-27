# Authentication (authn)

Authentication proves who someone is. Authorization ([authz](authz.md))
decides what they may do once you know.

- **Factors:** something you know (password), have (phone, security key) or
  are (biometric). Multi-factor combines two kinds.
- **Passkeys (WebAuthn)** replace passwords with a key pair kept on the
  device. They resist phishing because the browser binds each key to the
  site.
- **Federated sign-in:** let an identity provider authenticate the user.
  OpenID Connect adds an ID token (a signed JWT saying who signed in) on top
  of OAuth 2.0. Apps here sign in with Auth0 through
  [`auth_sdk`](../../../../auth_sdk/README.md).
- **Public clients** (single-page and mobile apps) can't keep a secret, so
  they use the authorization code flow with PKCE, never the implicit flow.
- **Sessions:** after sign-in, keep a short-lived session or token; put web
  tokens in `HttpOnly`, `Secure`, `SameSite` cookies rather than local
  storage, and verify every token's signature, issuer, audience and expiry.

## Sources

- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [NIST SP 800-63B, Digital Identity Guidelines: Authentication](https://pages.nist.gov/800-63-3/sp800-63b.html)
- [OAuth 2.0 Security Best Current Practice (RFC 9700)](https://datatracker.ietf.org/doc/html/rfc9700)
- [passkeys.dev](https://passkeys.dev/)
