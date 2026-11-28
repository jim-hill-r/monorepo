//! Team names for the nflverse abbreviations in `rooting.json`, and which
//! team's guide the page shows (decision 0010).

/// Every team a fan can pick, as nflverse abbreviations, in the order of
/// their full names.
pub const ALL: [&str; 32] = [
    "ARI", "ATL", "BAL", "BUF", "CAR", "CHI", "CIN", "CLE", "DAL", "DEN", "DET", "GB", "HOU",
    "IND", "JAX", "KC", "LV", "LAC", "LA", "MIA", "MIN", "NE", "NO", "NYG", "NYJ", "PHI", "PIT",
    "SF", "SEA", "TB", "TEN", "WAS",
];

/// Where this browser remembers the team it picked.
pub const STORAGE_KEY: &str = "bear_down.team";

/// The team a value names, as one of [`ALL`]: an abbreviation in any case,
/// such as `det` from `?team=det` or a remembered choice. `None` for
/// anything else.
pub fn known(value: &str) -> Option<&'static str> {
    let value = value.trim();
    ALL.into_iter()
        .find(|team| team.eq_ignore_ascii_case(value))
}

/// The team named by `?team=` in a page's query string (`location.search`).
pub fn from_query(search: &str) -> Option<&'static str> {
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|pair| pair.strip_prefix("team="))
        .and_then(known)
}

/// The team's guide on this site: `/rooting.json` for the Bears, whose
/// guide it has always been, and `/teams/<team>.json` for every other team.
pub fn guide_path(team: &str) -> String {
    if team == bear_down_core::BEARS {
        "/rooting.json".to_string()
    } else {
        format!("/teams/{team}.json")
    }
}

/// The query string that links to `team`'s guide: none for the Bears, the
/// page's default.
pub fn query(team: &str) -> String {
    if team == bear_down_core::BEARS {
        String::new()
    } else {
        format!("?team={team}")
    }
}

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
    fn every_team_can_be_picked_in_name_order() {
        let names: Vec<&str> = ALL.iter().map(|team| name(team)).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        sorted.dedup();
        assert_eq!(sorted.len(), 32);
        assert!(ALL.iter().all(|team| name(team) != *team));
    }

    #[test]
    fn the_team_comes_from_the_query_and_has_its_own_file() {
        assert_eq!(from_query("?team=det"), Some("DET"));
        assert_eq!(from_query("?arm=b&team=KC"), Some("KC"));
        assert_eq!(from_query("?team=XYZ"), None);
        assert_eq!(from_query("?team=../rooting"), None);
        assert_eq!(from_query(""), None);
        assert_eq!(known(" gb "), Some("GB"));
        assert_eq!(guide_path("CHI"), "/rooting.json");
        assert_eq!(guide_path("DET"), "/teams/DET.json");
        assert_eq!(query("CHI"), "");
        assert_eq!(query("SF"), "?team=SF");
    }

    #[test]
    fn falls_back_to_the_abbreviation() {
        assert_eq!(name("XYZ"), "XYZ");
        assert_eq!(nickname("XYZ"), "XYZ");
    }
}
