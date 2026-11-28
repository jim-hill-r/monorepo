//! Project tiers (decision 0019) and the product/platform balance that
//! `next-task` uses to keep at least half of the work on products (Jim,
//! 2026-11-28: "at least 50% of all of my claude credits are being used on
//! product work, not to let the platform work overwhelm it").
//!
//! A project's tier is read from `<project>/docs/project.toml` (`tier =
//! "..."`, decision 0022) when that file exists. Until those files are
//! added (product strategy phase 1), the tier comes from the table in
//! decision 0019, copied in `FALLBACK`; phase 1 deletes the copy.

use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};

/// The two sides Jim balances. `software-development-platform` and
/// `technical-platform` both count as platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    Product,
    Platform,
}

/// Decision 0019's product tier. Every other top-level folder, the root
/// files and the dot folders are platform.
const FALLBACK_PRODUCTS: &[&str] = &[
    "bear_down",
    "blueeel",
    "bogan",
    "cahokia",
    "controlled_chaos",
    "cookbook",
    "homer",
    "luggage",
    "my_lang",
    "profiles",
    "psy_ents",
    "sparkhill",
    "starcraft",
];

#[derive(Deserialize)]
struct ProjectToml {
    tier: Option<String>,
}

/// Which side a top-level folder is on. `None` (root files, and the old
/// root `issues/` folder in history) is platform.
pub fn side(root: &Path, project: Option<&str>) -> Side {
    let Some(project) = project else {
        return Side::Platform;
    };
    let file = root.join(project).join("docs/project.toml");
    if let Some(tier) = fs::read_to_string(file)
        .ok()
        .and_then(|t| toml::from_str::<ProjectToml>(&t).ok())
        .and_then(|p| p.tier)
    {
        return if tier == "product" {
            Side::Product
        } else {
            Side::Platform
        };
    }
    if FALLBACK_PRODUCTS.contains(&project) {
        Side::Product
    } else {
        Side::Platform
    }
}

/// Which side one change (a commit's file list) is on: the side with more
/// files, platform on a tie. Files at the repo root or in dot folders are
/// platform.
pub fn side_of_files(root: &Path, files: &[String]) -> Side {
    let mut product = 0usize;
    let mut platform = 0usize;
    for f in files {
        let project = f.split_once('/').map(|(top, _)| top);
        let project = project.filter(|p| !p.starts_with('.') && *p != "issues");
        match side(root, project) {
            Side::Product => product += 1,
            Side::Platform => platform += 1,
        }
    }
    if product > platform {
        Side::Product
    } else {
        Side::Platform
    }
}

/// The recent split of work, from the threads working now and the PRs
/// merged lately.
#[derive(Debug, Default, Clone, Serialize)]
pub struct Balance {
    pub working_product: usize,
    pub working_platform: usize,
    pub merged_product: usize,
    pub merged_platform: usize,
    /// How many days of `main` history the merged counts cover.
    pub days: u32,
}

/// Jim's floor: at least this share of work on products.
pub const PRODUCT_FLOOR: f64 = 0.5;

fn share(product: usize, platform: usize) -> Option<f64> {
    let total = product + platform;
    #[allow(clippy::cast_precision_loss)]
    (total > 0).then(|| product as f64 / total as f64)
}

impl Balance {
    /// The product share: the mean of the working threads' share and the
    /// merged PRs' share, each counted only when it has any. Threads are
    /// where credits go now; merged PRs show where they went this week.
    pub fn product_share(&self) -> Option<f64> {
        let parts: Vec<f64> = [
            share(self.working_product, self.working_platform),
            share(self.merged_product, self.merged_platform),
        ]
        .into_iter()
        .flatten()
        .collect();
        #[allow(clippy::cast_precision_loss)]
        (!parts.is_empty()).then(|| parts.iter().sum::<f64>() / parts.len() as f64)
    }

    /// True when the next pick should be a product issue if one qualifies.
    pub fn wants_product(&self) -> bool {
        self.product_share().is_some_and(|s| s < PRODUCT_FLOOR)
    }

    /// Count a new pick as a working thread.
    pub fn add_working(&mut self, side: Side) {
        match side {
            Side::Product => self.working_product += 1,
            Side::Platform => self.working_platform += 1,
        }
    }

    /// One line for the report.
    pub fn describe(&self) -> String {
        let pct = self
            .product_share()
            .map_or_else(|| "unknown".to_string(), |s| format!("{:.0}%", s * 100.0));
        format!(
            "product share {pct} (working threads: {} product, {} platform; PRs merged in the last {} days: {} product, {} platform; floor {:.0}%)",
            self.working_product,
            self.working_platform,
            self.days,
            self.merged_product,
            self.merged_platform,
            PRODUCT_FLOOR * 100.0
        )
    }
}

/// Count working threads by side from their projects (`repo` or an
/// `epic:` name means repo-wide work in standards, so platform).
pub fn count_working(root: &Path, working: &[String], balance: &mut Balance) {
    for w in working {
        let w = w.trim().trim_end_matches('/');
        if w.is_empty() {
            continue;
        }
        let project = w.split('/').next().filter(|p| {
            !(p.starts_with("epic:") || *p == "repo" || *p == "issues" || p.starts_with('.'))
        });
        balance.add_working(side(root, project));
    }
}

/// Count the commits on `HEAD` in the last `days` days by side. Squash
/// merges make one commit per PR. A missing git or a shallow clone gives
/// smaller counts, never an error.
pub fn count_merged(root: &Path, days: u32, balance: &mut Balance) {
    balance.days = days;
    let Ok(out) = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "log",
            &format!("--since={days}.days"),
            "--no-merges",
            "--name-only",
            "--format=%x00",
        ])
        .output()
    else {
        return;
    };
    if !out.status.success() {
        return;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for commit in text.split('\0') {
        let files: Vec<String> = commit
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
        if files.is_empty() {
            continue;
        }
        match side_of_files(root, &files) {
            Side::Product => balance.merged_product += 1,
            Side::Platform => balance.merged_platform += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers_from_decision_0019_and_project_toml() -> anyhow::Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        assert_eq!(side(root, Some("cookbook")), Side::Product);
        assert_eq!(side(root, Some("pane")), Side::Platform);
        assert_eq!(side(root, Some("auth_sdk")), Side::Platform);
        assert_eq!(side(root, None), Side::Platform);
        fs::create_dir_all(root.join("pane/docs"))?;
        fs::write(root.join("pane/docs/project.toml"), "tier = \"product\"\n")?;
        assert_eq!(side(root, Some("pane")), Side::Product);
        fs::create_dir_all(root.join("cookbook/docs"))?;
        fs::write(
            root.join("cookbook/docs/project.toml"),
            "tier = \"technical-platform\"\nstatus = \"active\"\n",
        )?;
        assert_eq!(side(root, Some("cookbook")), Side::Platform);
        Ok(())
    }

    #[test]
    fn a_change_takes_the_side_with_more_files() {
        let root = Path::new("/nonexistent");
        let files = |v: &[&str]| v.iter().map(|s| (*s).to_string()).collect::<Vec<_>>();
        assert_eq!(
            side_of_files(
                root,
                &files(&["cookbook/src/a.rs", "cookbook/README.md", "AGENTS.md"])
            ),
            Side::Product
        );
        assert_eq!(
            side_of_files(root, &files(&["cookbook/src/a.rs", "issues/x.md"])),
            Side::Platform
        );
        assert_eq!(
            side_of_files(root, &files(&["cookbook/docs/issues/x.md"])),
            Side::Product
        );
        assert_eq!(
            side_of_files(root, &files(&[".github/workflows/x.yml"])),
            Side::Platform
        );
    }

    #[test]
    fn share_is_the_mean_of_threads_and_prs() {
        let mut b = Balance::default();
        assert_eq!(b.product_share(), None);
        assert!(!b.wants_product());
        count_working(
            Path::new("/nonexistent"),
            &[
                "cookbook".into(),
                "pane".into(),
                "epic:Cost".into(),
                "repo".into(),
            ],
            &mut b,
        );
        assert_eq!((b.working_product, b.working_platform), (1, 3));
        assert_eq!(b.product_share(), Some(0.25));
        b.merged_product = 6;
        b.merged_platform = 4;
        assert_eq!(b.product_share(), Some(0.425));
        assert!(b.wants_product());
        b.add_working(Side::Product);
        b.add_working(Side::Product);
        assert_eq!(b.product_share(), Some(0.55));
        assert!(!b.wants_product());
        assert!(b.describe().contains("product share 55%"));
    }
}
