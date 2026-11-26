//! `docs/contrast.md`: every checked pair, readable without running anything.

use design_system_model::Report;

pub struct ThemeReport<'a> {
    pub title: String,
    pub reports: Vec<&'a Report>,
}

fn cell(r: &Report, fg: &str, bg: &str) -> String {
    match r.pairs.iter().find(|p| p.fg == fg && p.bg == bg) {
        Some(p) if p.passes() => format!("{:.2} · Lc {:.0}", p.ratio, p.apca.abs()),
        Some(p) => format!("**{:.2} < {}** · Lc {:.0}", p.ratio, p.min, p.apca.abs()),
        None => "n/a".into(),
    }
}

fn section(out: &mut String, t: &ThemeReport<'_>) {
    out.push_str(&format!("## {}\n\n", t.title));
    let Some(first) = t.reports.first() else {
        return;
    };
    out.push_str("| Pair |");
    for r in &t.reports {
        out.push_str(&format!(" {} |", r.combo.label()));
    }
    out.push_str("\n| --- |");
    for _ in &t.reports {
        out.push_str(" --- |");
    }
    out.push('\n');
    for p in &first.pairs {
        out.push_str(&format!("| `{}` on `{}` |", p.fg, p.bg));
        for r in &t.reports {
            out.push_str(&format!(" {} |", cell(r, &p.fg, &p.bg)));
        }
        out.push('\n');
    }
    out.push_str("| Data palette, closest pair |");
    for r in &t.reports {
        let cells: Vec<String> = r
            .distances
            .iter()
            .map(|d| {
                let mark = if d.passes() { "" } else { " **too close**" };
                format!(
                    "{}: {:.3} ({}, {}){mark}",
                    d.vision.name(),
                    d.distance,
                    d.between.0,
                    d.between.1
                )
            })
            .collect();
        out.push_str(&format!(" {} |", cells.join("; ")));
    }
    out.push_str("\n\n");
}

pub fn contrast_md(themes: &[ThemeReport<'_>], apps: &[ThemeReport<'_>], minimums: &str) -> String {
    let all = themes.iter().chain(apps).flat_map(|t| &t.reports);
    let (mut pairs, mut failing) = (0, 0);
    for r in all.clone() {
        pairs += r.pairs.len();
        failing += r.pairs.iter().filter(|p| !p.passes()).count();
        failing += r.distances.iter().filter(|d| !d.passes()).count();
    }
    let combos = all.count();
    let status = if failing == 0 {
        "Every pair passes.".to_string()
    } else {
        format!("**{failing} check(s) fail**, shown in bold below.")
    };
    let mut out = format!(
        "# Contrast report\n\n<!-- {} -->\n\nWCAG 2.2 contrast ratios for every pair in `tokens/pairs.toml`, in {combos} combinations of theme, scheme and mode ({pairs} pairs). {status} \"Lc\" is the APCA lightness contrast, shown for information only (WCAG 3 is not final).\n\n{minimums}\n",
        crate::css::HEADER
    );
    for t in themes {
        section(&mut out, t);
    }
    for t in apps {
        section(&mut out, t);
    }
    out
}
