//! Requirement IDs (`COOK-001`) and eval IDs (`recipe-mobile-layout`).

/// Splits a requirement ID such as `COOK-001` into its prefix and number.
///
/// The prefix is two to five ASCII uppercase letters and the number has at
/// least three digits.
pub fn parse_requirement_id(id: &str) -> Option<(&str, u32)> {
    let (prefix, number) = id.split_once('-')?;
    let prefix_ok =
        (2..=5).contains(&prefix.len()) && prefix.chars().all(|c| c.is_ascii_uppercase());
    let number_ok = number.len() >= 3 && number.chars().all(|c| c.is_ascii_digit());
    if !prefix_ok || !number_ok {
        return None;
    }
    Some((prefix, number.parse().ok()?))
}

/// The snake-case form of a requirement ID that Rust test names start with:
/// `COOK-001` becomes `cook_001`.
pub fn snake_case_id(id: &str) -> String {
    id.to_ascii_lowercase().replace('-', "_")
}

/// True for lowercase ASCII words joined by single hyphens, such as
/// `recipe-lcp` or `layout2`.
pub fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|word| {
            !word.is_empty()
                && word
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_requirement_ids() {
        assert_eq!(parse_requirement_id("COOK-001"), Some(("COOK", 1)));
        assert_eq!(parse_requirement_id("CHAOS-042"), Some(("CHAOS", 42)));
        assert_eq!(parse_requirement_id("AB-1000"), Some(("AB", 1000)));
    }

    #[test]
    fn rejects_malformed_requirement_ids() {
        for bad in [
            "COOK-01",
            "cook-001",
            "C-001",
            "TOOLONG-001",
            "COOK001",
            "COOK-0a1",
            "COOK-",
            "-001",
            "",
        ] {
            assert_eq!(parse_requirement_id(bad), None, "{bad}");
        }
    }

    #[test]
    fn snake_case_ids() {
        assert_eq!(snake_case_id("COOK-001"), "cook_001");
        assert_eq!(snake_case_id("CHAOS-120"), "chaos_120");
    }

    #[test]
    fn kebab_case() {
        assert!(is_kebab_case("recipe-lcp"));
        assert!(is_kebab_case("layout2"));
        assert!(!is_kebab_case("Recipe-lcp"));
        assert!(!is_kebab_case("recipe--lcp"));
        assert!(!is_kebab_case("-recipe"));
        assert!(!is_kebab_case("recipe_lcp"));
        assert!(!is_kebab_case(""));
    }
}
