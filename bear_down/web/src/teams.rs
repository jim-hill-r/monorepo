//! Team names for the nflverse abbreviations in `rooting.json`.

/// The team's full name, or the abbreviation itself if it is unknown.
pub fn name(abbr: &str) -> &str {
    match abbr {
        "ARI" => "Arizona Cardinals",
        "ATL" => "Atlanta Falcons",
        "BAL" => "Baltimore Ravens",
        "BUF" => "Buffalo Bills",
        "CAR" => "Carolina Panthers",
        "CHI" => "Chicago Bears",
        "CIN" => "Cincinnati Bengals",
        "CLE" => "Cleveland Browns",
        "DAL" => "Dallas Cowboys",
        "DEN" => "Denver Broncos",
        "DET" => "Detroit Lions",
        "GB" => "Green Bay Packers",
        "HOU" => "Houston Texans",
        "IND" => "Indianapolis Colts",
        "JAX" => "Jacksonville Jaguars",
        "KC" => "Kansas City Chiefs",
        "LA" | "LAR" => "Los Angeles Rams",
        "LAC" => "Los Angeles Chargers",
        "LV" => "Las Vegas Raiders",
        "MIA" => "Miami Dolphins",
        "MIN" => "Minnesota Vikings",
        "NE" => "New England Patriots",
        "NO" => "New Orleans Saints",
        "NYG" => "New York Giants",
        "NYJ" => "New York Jets",
        "PHI" => "Philadelphia Eagles",
        "PIT" => "Pittsburgh Steelers",
        "SEA" => "Seattle Seahawks",
        "SF" => "San Francisco 49ers",
        "TB" => "Tampa Bay Buccaneers",
        "TEN" => "Tennessee Titans",
        "WAS" => "Washington Commanders",
        other => other,
    }
}

/// The team's nickname ("Packers"), for compact rows.
pub fn nickname(abbr: &str) -> &str {
    let full = name(abbr);
    full.rsplit(' ').next().unwrap_or(full)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_known_teams() {
        assert_eq!(name("CHI"), "Chicago Bears");
        assert_eq!(name("LA"), "Los Angeles Rams");
        assert_eq!(nickname("GB"), "Packers");
        assert_eq!(nickname("SF"), "49ers");
    }

    #[test]
    fn falls_back_to_the_abbreviation() {
        assert_eq!(name("XYZ"), "XYZ");
        assert_eq!(nickname("XYZ"), "XYZ");
    }
}
