//! The parts of `standards/radar/radar.json` the cost rules need.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Blip {
    pub name: String,
    pub ring: String,
    pub kind: String,
    /// US dollars a month.
    pub budget: f64,
}

impl Blip {
    pub fn is_vendor(&self) -> bool {
        self.kind == "vendor"
    }

    /// Adopt and Trial vendors may be used; Assess and Hold need Jim first.
    pub fn is_approved(&self) -> bool {
        matches!(self.ring.as_str(), "Adopt" | "Trial")
    }

    /// The monthly budget in whole cents.
    pub fn budget_cents(&self) -> u64 {
        (self.budget * 100.0).round().max(0.0) as u64
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Radar {
    pub blips: Vec<Blip>,
}

impl Radar {
    pub fn vendor(&self, name: &str) -> Option<&Blip> {
        self.blips.iter().find(|b| b.is_vendor() && b.name == name)
    }
}

pub fn parse_radar(text: &str) -> Result<Radar, String> {
    let blips: Vec<Blip> = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(Radar { blips })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_repo_radar_parses() {
        let radar = parse_radar(include_str!("../../../standards/radar/radar.json"))
            .expect("radar.json parses");
        let pages = radar.vendor("Cloudflare Pages").expect("Pages is a vendor");
        assert!(pages.is_approved());
        assert_eq!(
            radar.vendor("Claude Code").map(Blip::budget_cents),
            Some(2500)
        );
        assert!(radar.vendor("Rust").is_none(), "Rust is a technology");
    }
}
