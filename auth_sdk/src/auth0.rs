//! Auth0 endpoint URLs built from one tenant domain.
//!
//! Auth0 publishes its OIDC issuer with a trailing slash, and `openidconnect`
//! compares the configured issuer to it as an exact string, so an issuer
//! without the slash fails discovery on every page load. Apps used to keep
//! the issuer, authorize and token URLs as three hand-written constants,
//! which drifted (cahokia shipped without the slash). Build them here from
//! the tenant domain instead, so they can't disagree.

use crate::provider::{AuthError, ProviderConfig};

/// The Auth0 URLs a sign-in flow needs, all from one tenant domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auth0Urls {
    /// The OIDC issuer, `https://<domain>/`, with the trailing slash Auth0
    /// publishes.
    pub issuer: String,
    /// The authorize endpoint, `https://<domain>/authorize`.
    pub authorize: String,
    /// The token endpoint, `https://<domain>/oauth/token`.
    pub token: String,
}

impl Auth0Urls {
    /// Builds the URLs from an Auth0 tenant domain.
    ///
    /// `domain` is the tenant's host, such as `your-tenant.us.auth0.com`. An
    /// `https://` prefix and trailing slashes are accepted and dropped, so
    /// the domain can be pasted from the Auth0 dashboard or an issuer URL.
    /// Anything else (an empty domain, another scheme, a path, a query,
    /// whitespace inside it) is a `ConfigError`.
    pub fn from_domain(domain: &str) -> Result<Self, AuthError> {
        let host = tenant_host(domain)?;
        Ok(Auth0Urls {
            issuer: format!("https://{host}/"),
            authorize: format!("https://{host}/authorize"),
            token: format!("https://{host}/oauth/token"),
        })
    }

    /// The tenant's origin, `https://<domain>`, with no trailing slash: the
    /// form a Content Security Policy `connect-src` source takes.
    pub fn origin(&self) -> &str {
        self.issuer.trim_end_matches('/')
    }
}

impl ProviderConfig {
    /// A provider config for an Auth0 tenant, with OIDC discovery on.
    ///
    /// The issuer, authorize and token URLs all come from
    /// [`Auth0Urls::from_domain`], so the issuer always has its trailing
    /// slash.
    pub fn auth0(
        domain: &str,
        client_id: impl Into<String>,
        redirect_url: impl Into<String>,
    ) -> Result<Self, AuthError> {
        let urls = Auth0Urls::from_domain(domain)?;
        Ok(ProviderConfig {
            client_id: client_id.into(),
            auth_url: urls.authorize,
            token_url: urls.token,
            redirect_url: redirect_url.into(),
            issuer_url: Some(urls.issuer),
        })
    }
}

/// The bare host from a tenant domain, or a `ConfigError` saying what is
/// wrong with it.
fn tenant_host(domain: &str) -> Result<&str, AuthError> {
    let invalid =
        |why: &str| AuthError::ConfigError(format!("invalid Auth0 domain {domain:?}: {why}"));
    let trimmed = domain.trim();
    let without_scheme = match trimmed.split_once("://") {
        Some(("https", rest)) => rest,
        Some(_) => return Err(invalid("Auth0 is served over https only")),
        None => trimmed,
    };
    let host = without_scheme.trim_end_matches('/');
    if host.is_empty() {
        return Err(invalid("the domain is empty"));
    }
    if let Some(bad) = host
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '.' || *c == '-'))
    {
        return Err(invalid(&format!(
            "{bad:?} is not allowed; give only the tenant host, such as your-tenant.us.auth0.com"
        )));
    }
    if host.starts_with('.') || host.ends_with('.') || host.contains("..") {
        return Err(invalid("the host has an empty label"));
    }
    Ok(host)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TENANT: &str = "dev-tenant.us.auth0.com";

    fn expected() -> Auth0Urls {
        Auth0Urls {
            issuer: "https://dev-tenant.us.auth0.com/".into(),
            authorize: "https://dev-tenant.us.auth0.com/authorize".into(),
            token: "https://dev-tenant.us.auth0.com/oauth/token".into(),
        }
    }

    #[test]
    fn builds_urls_from_a_bare_domain() {
        assert_eq!(Auth0Urls::from_domain(TENANT).ok(), Some(expected()));
    }

    #[test]
    fn issuer_has_exactly_one_trailing_slash() {
        let urls = Auth0Urls::from_domain(TENANT).ok();
        let issuer = urls.map(|u| u.issuer).unwrap_or_default();
        assert!(issuer.ends_with(".com/"), "{issuer}");
        assert!(!issuer.ends_with("//"), "{issuer}");
    }

    #[test]
    fn accepts_https_prefix_and_trailing_slashes() {
        for domain in [
            "https://dev-tenant.us.auth0.com",
            "https://dev-tenant.us.auth0.com/",
            "dev-tenant.us.auth0.com/",
            "dev-tenant.us.auth0.com//",
            "  dev-tenant.us.auth0.com  ",
        ] {
            assert_eq!(
                Auth0Urls::from_domain(domain).ok(),
                Some(expected()),
                "{domain:?}"
            );
        }
    }

    #[test]
    fn endpoint_urls_have_no_double_slash() {
        let urls = Auth0Urls::from_domain("https://dev-tenant.us.auth0.com/").ok();
        let urls = urls.unwrap_or_else(expected);
        for url in [&urls.issuer, &urls.authorize, &urls.token] {
            let path = url.trim_start_matches("https://");
            assert!(!path.contains("//"), "{url}");
        }
    }

    #[test]
    fn origin_has_no_trailing_slash() {
        assert_eq!(expected().origin(), "https://dev-tenant.us.auth0.com");
    }

    #[test]
    fn rejects_bad_domains() {
        for domain in [
            "",
            "   ",
            "https://",
            "/",
            "http://dev-tenant.us.auth0.com",
            "ftp://dev-tenant.us.auth0.com",
            "dev-tenant.us.auth0.com/authorize",
            "dev-tenant.us.auth0.com?x=1",
            "dev-tenant.us.auth0.com:443",
            "user@dev-tenant.us.auth0.com",
            "dev tenant.us.auth0.com",
            ".us.auth0.com",
            "dev-tenant..auth0.com",
        ] {
            assert!(
                matches!(
                    Auth0Urls::from_domain(domain),
                    Err(AuthError::ConfigError(_))
                ),
                "{domain:?} should be rejected"
            );
        }
    }

    #[test]
    fn error_names_the_domain() {
        let message = Auth0Urls::from_domain("http://x.auth0.com")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(message.contains("http://x.auth0.com"), "{message}");
        assert!(message.contains("https"), "{message}");
    }

    #[test]
    fn provider_config_uses_the_built_urls() {
        let config = ProviderConfig::auth0(TENANT, "client", "https://app.example/");
        let config = config.ok();
        let urls = expected();
        assert_eq!(
            config.as_ref().map(|c| c.issuer_url.clone()),
            Some(Some(urls.issuer))
        );
        assert_eq!(
            config.as_ref().map(|c| c.auth_url.as_str()),
            Some(urls.authorize.as_str())
        );
        assert_eq!(
            config.as_ref().map(|c| c.token_url.as_str()),
            Some(urls.token.as_str())
        );
        assert_eq!(
            config.as_ref().map(|c| c.client_id.as_str()),
            Some("client")
        );
        assert_eq!(
            config.as_ref().map(|c| c.redirect_url.as_str()),
            Some("https://app.example/")
        );
    }

    #[test]
    fn provider_config_rejects_a_bad_domain() {
        assert!(matches!(
            ProviderConfig::auth0("http://x", "client", "https://app.example/"),
            Err(AuthError::ConfigError(_))
        ));
    }
}
