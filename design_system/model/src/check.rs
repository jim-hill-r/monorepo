//! The contrast check: every pair in `pairs.toml`, and the data palette's
//! distinctness, for one resolved combination.

use crate::DesignSystem;
use crate::color::{Rgba, Vision, apca_lc, contrast_ratio, distance};
use crate::resolve::Resolved;

/// One foreground checked against one background.
#[derive(Clone, Debug, PartialEq)]
pub struct PairResult {
    pub fg: String,
    pub bg: String,
    pub ratio: f64,
    pub min: f64,
    /// APCA Lc, for information only.
    pub apca: f64,
}

impl PairResult {
    pub fn passes(&self) -> bool {
        // Compare at the two decimals the report shows, so a ratio printed as
        // 4.50 never fails a 4.5 minimum.
        (self.ratio * 100.0).floor() / 100.0 >= self.min
    }
}

/// The smallest distance between two data colours, seen with one vision.
#[derive(Clone, Debug, PartialEq)]
pub struct DistanceResult {
    pub vision: Vision,
    pub distance: f64,
    pub min: f64,
    /// The closest pair.
    pub between: (String, String),
}

impl DistanceResult {
    pub fn passes(&self) -> bool {
        self.distance >= self.min
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub theme: String,
    pub combo: crate::Combo,
    pub pairs: Vec<PairResult>,
    pub distances: Vec<DistanceResult>,
}

impl Report {
    pub fn failures(&self) -> Vec<String> {
        let mut out = Vec::new();
        for p in self.pairs.iter().filter(|p| !p.passes()) {
            out.push(format!(
                "{} {}: {} on {} is {:.2}:1, needs {}:1",
                self.theme, self.combo, p.fg, p.bg, p.ratio, p.min
            ));
        }
        for d in self.distances.iter().filter(|d| !d.passes()) {
            out.push(format!(
                "{} {}: {} and {} are too alike for {} (distance {:.3}, needs {})",
                self.theme,
                self.combo,
                d.between.0,
                d.between.1,
                d.vision.name(),
                d.distance,
                d.min
            ));
        }
        out
    }
}

fn opaque(c: Rgba, under: Rgba) -> Rgba {
    if c.a == 255 { c } else { c.over(under) }
}

/// Check one resolved combination against `pairs.toml`.
pub fn check(ds: &DesignSystem, r: &Resolved) -> Result<Report, String> {
    let get = |name: &str| {
        r.color(name).ok_or_else(|| {
            format!(
                "{} {}: pairs.toml names unknown token {name:?}",
                r.theme, r.combo
            )
        })
    };
    let page = get("bg")?;
    let mut pairs = Vec::new();
    let rules = ds
        .pairs
        .pair
        .iter()
        .map(|p| (&p.fg, &p.bg, p.min, p.accessible));
    let data = &ds.pairs.data;
    let data_rule = std::iter::once((&data.tokens, &data.on, data.min, data.accessible));
    for (fgs, bgs, min, acc) in rules.chain(data_rule) {
        let min = if r.combo.accessible { acc } else { min };
        for fg in fgs {
            for bg in bgs {
                let b = opaque(get(bg)?, page);
                let f = opaque(get(fg)?, b);
                pairs.push(PairResult {
                    fg: fg.clone(),
                    bg: bg.clone(),
                    ratio: contrast_ratio(f, b),
                    min,
                    apca: apca_lc(f, b),
                });
            }
        }
    }

    let visions: &[Vision] = if r.combo.accessible {
        &Vision::ALL
    } else {
        &[Vision::Normal]
    };
    let min = if r.combo.accessible {
        data.accessible_distance
    } else {
        data.min_distance
    };
    let colors: Vec<(String, Rgba)> = data
        .tokens
        .iter()
        .map(|t| Ok((t.clone(), get(t)?)))
        .collect::<Result<_, String>>()?;
    let mut distances = Vec::new();
    for v in visions {
        let mut best: Option<DistanceResult> = None;
        for (i, (a, ca)) in colors.iter().enumerate() {
            for (b, cb) in colors.iter().skip(i + 1) {
                let d = distance(*ca, *cb, *v);
                if best.as_ref().is_none_or(|x| d < x.distance) {
                    best = Some(DistanceResult {
                        vision: *v,
                        distance: d,
                        min,
                        between: (a.clone(), b.clone()),
                    });
                }
            }
        }
        distances.extend(best);
    }

    Ok(Report {
        theme: r.theme.clone(),
        combo: r.combo,
        pairs,
        distances,
    })
}
