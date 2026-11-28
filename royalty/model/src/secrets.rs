//! RTOML-007: a `royalty.toml` is public and ships inside packages, so no
//! value may look like a secret (ROYL-008). This is a tripwire for mistakes,
//! not a full secret scanner: gitleaks still runs on every commit.

/// Prefixes of well-known token and key formats.
const PREFIXES: &[&str] = &[
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "sk_live_",
    "sk_test_",
    "rk_live_",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "AKIA",
    "ASIA",
    "AIza",
    "sk-ant-",
    "npm_",
];

/// Fields whose values are public by design and may be long and random
/// looking: a payment address.
const PUBLIC_RANDOM_FIELDS: &[&str] = &["address"];

/// Why `value` (the value of field `key`) looks like a secret, or `None`.
pub fn looks_secret(key: &str, value: &str) -> Option<&'static str> {
    if value.contains("PRIVATE KEY") {
        return Some("a private key block");
    }
    for token in value.split(|c: char| !is_token_char(c)) {
        if PREFIXES
            .iter()
            .any(|p| token.starts_with(p) && token.len() >= p.len() + 12)
        {
            return Some("a token or key prefix");
        }
        if !PUBLIC_RANDOM_FIELDS.contains(&key) && looks_random(token) {
            return Some("a long random string");
        }
    }
    None
}

fn is_token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+')
}

/// At least 32 characters with upper case, lower case and digits mixed,
/// like an API key; words and slugs don't qualify.
fn looks_random(token: &str) -> bool {
    token.len() >= 32
        && token.chars().any(|c| c.is_ascii_uppercase())
        && token.chars().any(|c| c.is_ascii_lowercase())
        && token.chars().filter(|c| c.is_ascii_digit()).count() >= 4
}

#[cfg(test)]
mod tests {
    use super::looks_secret;

    // Test tokens are split so secret scanners don't flag this file.
    #[test]
    fn royl_008_tokens_and_keys_are_caught() {
        assert!(
            looks_secret(
                "account",
                &format!("{}{}", "ghp_", "abcdefghijklmnopqrstuvwxyz0123456789")
            )
            .is_some()
        );
        assert!(
            looks_secret(
                "url",
                &format!(
                    "https://pay.example/?key={}{}",
                    "sk_live_", "51HxYzAbCdEfGhIjKl"
                )
            )
            .is_some()
        );
        assert!(looks_secret("contact", "-----BEGIN OPENSSH PRIVATE KEY-----").is_some());
        assert!(looks_secret("name", "Zx9Qm2Lk8Vb4Nc7Hd1Jf5Rt3Wy6Pa0Se").is_some());
    }

    #[test]
    fn royl_008_public_values_pass() {
        for (key, value) in [
            ("account", "sam-example"),
            ("url", "https://geocalc.example/pay"),
            ("contact", "billing@webframe.example"),
            (
                "description",
                "Generic library for producing auth flows in applications.",
            ),
            ("address", "0x52908400098527886E0F7030069857D2E4169EE7"),
            ("repository", "https://github.com/formkit-example/formkit"),
        ] {
            assert_eq!(looks_secret(key, value), None, "{key} = {value}");
        }
    }
}
