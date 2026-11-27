//! `<project>/docs/threat.toml`: what a project exposes, to whom, with which
//! data, secrets and sign-in, and its main threats. The format is in
//! `standards/docs/security.md`.

use serde::{Deserialize, Deserializer};

use crate::Date;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatFile {
    /// The last time someone checked this file against the code.
    #[serde(deserialize_with = "date")]
    pub reviewed: Date,
    /// One or two sentences: what the project exposes and holds.
    pub summary: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub surface: Vec<Surface>,
    #[serde(default)]
    pub secret: Vec<Secret>,
    #[serde(default)]
    pub auth: Option<Auth>,
    #[serde(default)]
    pub threat: Vec<Threat>,
}

/// Something someone outside the repo can reach, or would if it shipped.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Surface {
    pub name: String,
    pub kind: Kind,
    #[serde(default)]
    pub url: Option<String>,
    pub access: Access,
    pub data: Data,
    /// What Pages preview URLs expose. Required for `kind = "pages"`.
    #[serde(default)]
    pub previews: Option<Access>,
    /// The repo-relative `_headers` file or Worker source that sets the
    /// security headers. Required for a public web surface.
    #[serde(default)]
    pub headers: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Pages,
    Worker,
    Api,
    Desktop,
    Mobile,
    Cli,
    Library,
}

impl Kind {
    /// Served over HTTP, so it needs security headers when public.
    pub fn is_web(self) -> bool {
        matches!(self, Kind::Pages | Kind::Worker | Kind::Api)
    }

    /// Runs on the user's device, so it can't keep a client secret.
    pub fn is_client(self) -> bool {
        matches!(self, Kind::Pages | Kind::Desktop | Kind::Mobile)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Access {
    Public,
    Auth0,
    CloudflareAccess,
    /// A shared secret a machine client sends (basic auth or a bearer
    /// token), listed as a `[[secret]]`. For APIs only people never use.
    Token,
    /// Not deployed or not reachable from outside.
    None,
}

impl Access {
    pub fn as_str(self) -> &'static str {
        match self {
            Access::Public => "public",
            Access::Auth0 => "auth0",
            Access::CloudflareAccess => "cloudflare-access",
            Access::Token => "token",
            Access::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Data {
    Public,
    Personal,
    /// Jim-only: dashboards, money, repo state.
    Private,
    Secret,
}

impl Data {
    pub fn as_str(self) -> &'static str {
        match self {
            Data::Public => "public",
            Data::Personal => "personal",
            Data::Private => "private",
            Data::Secret => "secret",
        }
    }
}

/// A secret the project uses, by name. Never its value.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Secret {
    pub name: String,
    #[serde(rename = "where")]
    pub location: Where,
    #[serde(default)]
    pub used_by: Vec<String>,
    /// What it can do: the narrowest scope that works.
    #[serde(default)]
    pub scope: Option<String>,
    /// Its `id` in `standards/settings.toml`.
    #[serde(default)]
    pub setting: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Where {
    Github,
    Worker,
    Auth0,
    Local,
}

/// How people sign in.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auth {
    pub provider: String,
    pub flow: Flow,
    #[serde(default)]
    pub audience: Option<String>,
    #[serde(default)]
    pub callbacks: Vec<String>,
    /// Its `id` in `standards/settings.toml`.
    #[serde(default)]
    pub setting: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Flow {
    AuthorizationCodePkce,
    /// Authorization code with a client secret: servers only.
    AuthorizationCode,
    /// Machine to machine: servers only.
    ClientCredentials,
    Implicit,
    Password,
}

impl Flow {
    pub fn as_str(self) -> &'static str {
        match self {
            Flow::AuthorizationCodePkce => "authorization-code-pkce",
            Flow::AuthorizationCode => "authorization-code",
            Flow::ClientCredentials => "client-credentials",
            Flow::Implicit => "implicit",
            Flow::Password => "password",
        }
    }
}

/// One of the main threats, STRIDE-tagged.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Threat {
    pub id: String,
    pub stride: Stride,
    pub what: String,
    #[serde(default)]
    pub mitigation: Option<String>,
    /// The surface it is about, by `name`. Without one it covers every
    /// surface in the file.
    #[serde(default)]
    pub surface: Option<String>,
    /// An eval in the project's `evals.toml` that proves the mitigation.
    #[serde(default)]
    pub eval: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stride {
    Spoofing,
    Tampering,
    Repudiation,
    InformationDisclosure,
    DenialOfService,
    ElevationOfPrivilege,
}

fn date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Date, D::Error> {
    let s = String::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

/// Parses a `docs/threat.toml`. The error is one line, for a THR-002 finding.
pub fn parse_threat_file(text: &str) -> Result<ThreatFile, String> {
    toml::from_str(text).map_err(|e| e.message().trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const EXAMPLE: &str = r#"
reviewed = "2026-11-26"
summary = "Public recipe site; Auth0 sign-in only to edit."

[[surface]]
name = "site"
kind = "pages"
url = "https://jimhillr-cookbook.pages.dev"
access = "public"
data = "public"
previews = "public"
headers = "cookbook/web/public/_headers"

[[surface]]
name = "edit API"
kind = "worker"
url = "https://jimhillr-cookbook.pages.dev/api/*"
access = "auth0"
data = "personal"

[[secret]]
name = "CLOUDFLARE_API_TOKEN"
where = "github"
used_by = ["push-main.yml deploy step"]
scope = "Pages edit on one account"
setting = "cloudflare-api-token-scopes"

[auth]
provider = "Auth0"
flow = "authorization-code-pkce"
audience = "https://cookbook/api"
callbacks = ["https://jimhillr-cookbook.pages.dev/callback", "http://localhost:8080/callback"]
setting = "auth0-cookbook-app"

[[threat]]
id = "COOK-T1"
stride = "spoofing"
what = "Someone edits recipes without signing in."
mitigation = "The Worker verifies the Auth0 JWT on every write."
surface = "edit API"
eval = "COOK-E12"
"#;

    #[test]
    fn test_parses_the_example_from_the_plan() {
        let file = parse_threat_file(EXAMPLE).expect("parses");
        assert_eq!(file.reviewed.to_string(), "2026-11-26");
        assert_eq!(file.surface[0].kind, Kind::Pages);
        assert_eq!(file.surface[0].previews, Some(Access::Public));
        assert_eq!(file.surface[1].access, Access::Auth0);
        assert_eq!(file.secret[0].location, Where::Github);
        let auth = file.auth.as_ref().expect("auth");
        assert_eq!(auth.flow, Flow::AuthorizationCodePkce);
        assert_eq!(auth.callbacks.len(), 2);
        assert_eq!(file.threat[0].stride, Stride::Spoofing);
    }

    #[test]
    fn test_summary_only_file_parses() {
        let file = parse_threat_file("reviewed = \"2026-11-26\"\nsummary = \"A library.\"\n")
            .expect("parses");
        assert!(file.surface.is_empty() && file.auth.is_none());
    }

    #[test]
    fn test_rejects_unknown_fields_and_values() {
        let base = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n";
        for (extra, want) in [
            ("colour = \"red\"\n", "unknown field"),
            (
                "[[surface]]\nname = \"x\"\nkind = \"ftp\"\naccess = \"public\"\ndata = \"public\"\n",
                "unknown variant",
            ),
            (
                "[[surface]]\nname = \"x\"\nkind = \"cli\"\naccess = \"vpn\"\ndata = \"public\"\n",
                "unknown variant",
            ),
            (
                "[[secret]]\nname = \"X\"\nwhere = \"vault\"\n",
                "unknown variant",
            ),
            (
                "[[threat]]\nid = \"T1\"\nstride = \"phishing\"\nwhat = \"x\"\n",
                "unknown variant",
            ),
            (
                "[auth]\nprovider = \"Auth0\"\nflow = \"magic\"\n",
                "unknown variant",
            ),
        ] {
            let err = parse_threat_file(&format!("{base}{extra}")).expect_err(extra);
            assert!(err.contains(want), "{extra}: {err}");
        }
    }

    #[test]
    fn test_rejects_bad_dates() {
        let err =
            parse_threat_file("reviewed = \"26 Nov 2026\"\nsummary = \"x\"\n").expect_err("bad");
        assert!(err.contains("YYYY-MM-DD"), "{err}");
    }
}
