//! `bear_down`: downloads the nflverse data, runs the simulation and writes
//! `rooting.json` for the web app. Only a stub for now; the data loading and
//! simulation arrive with the later items in `ISSUES.md`.

fn main() {
    println!("{}", summary());
}

/// What this build of the CLI does, printed on every run.
fn summary() -> String {
    format!(
        "bear_down {}: rooting guide for {} fans. Simulation not implemented yet.",
        env!("CARGO_PKG_VERSION"),
        bear_down_core::BEARS
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_names_the_version_and_the_bears() {
        let text = summary();
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
        assert!(text.contains("CHI"));
    }
}
