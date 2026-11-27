//! Build the design system's preview site: one panel per theme, scheme and
//! mode, each showing every token. Run `cargo run -p design_system_preview --
//! <out dir>` (the `release` task does, into `dist/`).

use design_system_model::spec::COLOR_TOKENS;
use design_system_model::{Combo, DesignSystem, Resolved, find_app_themes};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const PAGE_CSS: &str = include_str!("preview.css");

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The attributes that pin one panel to one combination, whatever the
/// visitor's system settings.
fn attrs(theme: &str, combo: Combo) -> String {
    format!(
        "data-theme=\"{}\" data-scheme=\"{}\" data-a11y=\"{}\"",
        esc(theme),
        combo.scheme.name(),
        if combo.accessible { "on" } else { "off" }
    )
}

fn swatch(name: &str, r: &Resolved) -> String {
    let hex = r.color(name).map(|c| c.to_string()).unwrap_or_default();
    format!(
        "<li class=\"swatch\"><span class=\"chip\" style=\"background: var(--ds-color-{name})\"></span><code>{name}</code> <span class=\"hex\">{hex}</span></li>"
    )
}

fn panel(label: &str, r: &Resolved) -> String {
    let mut s = format!(
        "<section class=\"panel\" {} aria-label=\"{} {}\">\n",
        attrs(&r.theme, r.combo),
        esc(label),
        r.combo.label().to_lowercase()
    );
    s.push_str(&format!(
        "<h3>{} · {}</h3>\n<p>Body text in <code>--ds-font-body</code>. <span class=\"muted\">Muted text for captions and help.</span> <a href=\"#top\">A link</a>.</p>\n",
        esc(label),
        r.combo.label()
    ));
    s.push_str("<p class=\"row\"><button class=\"accent\">Accent button</button> <button class=\"plain\">Plain button</button> <input aria-label=\"Sample input\" value=\"Input\"> <button class=\"focus-demo\">Focused</button></p>\n");
    s.push_str("<p class=\"row\">");
    for st in ["success", "warning", "danger", "info"] {
        s.push_str(&format!(
            "<span class=\"badge\" style=\"background: var(--ds-color-{st}); color: var(--ds-color-on-{st})\">{st}</span> <span style=\"color: var(--ds-color-{st})\">{st} text</span> "
        ));
    }
    s.push_str("</p>\n<div class=\"cards\"><div class=\"card\">surface</div><div class=\"card raised\">surface-raised</div></div>\n");
    s.push_str("<div class=\"data\" role=\"img\" aria-label=\"The eight data colours\">");
    for i in 1..=8 {
        s.push_str(&format!(
            "<span style=\"background: var(--ds-color-data-{i}); height: {}%\">{i}</span>",
            30 + i * 8
        ));
    }
    s.push_str("</div>\n<p class=\"scale\">");
    for step in ["xs", "sm", "base", "lg", "xl", "2xl", "3xl"] {
        s.push_str(&format!(
            "<span style=\"font-size: var(--ds-text-{step})\">{step}</span> "
        ));
    }
    s.push_str("</p>\n<details><summary>Every colour token</summary><ul class=\"swatches\">");
    for k in COLOR_TOKENS {
        s.push_str(&swatch(k, r));
    }
    s.push_str("</ul></details>\n</section>\n");
    s
}

fn page(ds: &DesignSystem, themes: &[(String, String, Vec<Resolved>)], app_css: &str) -> String {
    let mut body = String::new();
    let names: Vec<&str> = themes.iter().map(|t| t.0.as_str()).collect();
    for (name, label, all) in themes {
        let description = ds
            .theme(name)
            .and_then(|t| t.description.clone())
            .map(|d| format!("<p class=\"lede\">{}</p>\n", esc(&d)))
            .unwrap_or_default();
        body.push_str(&format!(
            "<h2 id=\"{}\">{}</h2>\n{description}<div class=\"grid\">\n",
            esc(name),
            esc(label)
        ));
        for r in all {
            body.push_str(&panel(label, r));
        }
        body.push_str("</div>\n");
    }
    let nav: Vec<String> = themes
        .iter()
        .map(|(n, l, _)| format!("<a href=\"#{}\">{}</a>", esc(n), esc(l)))
        .collect();
    format!(
        "<!doctype html>\n<html lang=\"en\" data-ds-default=\"professional\" data-ds-themes=\"{}\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>Design system</title>\n<script src=\"theme.js\"></script>\n<link rel=\"stylesheet\" href=\"fonts.css\">\n<link rel=\"stylesheet\" href=\"tokens.css\">\n<style>{app_css}</style>\n<link rel=\"stylesheet\" href=\"base.css\">\n<style>{PAGE_CSS}</style>\n</head>\n<body id=\"top\">\n<header>\n<h1>Design system</h1>\n<p>Every theme in light, dark and accessible mode. Each panel is pinned to its combination; the page itself follows your system settings. Contrast results: <a href=\"https://github.com/jimhillr/monorepo/blob/main/design_system/docs/contrast.md\">contrast.md</a>.</p>\n<nav>{}</nav>\n</header>\n<main>\n{body}</main>\n</body>\n</html>\n",
        names.join(" "),
        nav.join(" · ")
    )
}

fn fonts_css(dir: &Path) -> Result<String, String> {
    let mut out = String::new();
    let mut files: Vec<String> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".woff2"))
        .collect();
    files.sort();
    for f in files {
        let family = if f.starts_with("press-start") {
            "Press Start 2P"
        } else {
            "Atkinson Hyperlegible"
        };
        let weight = if f.contains("-700-") { 700 } else { 400 };
        let style = if f.contains("italic") {
            "italic"
        } else {
            "normal"
        };
        out.push_str(&format!(
            "@font-face {{\n  font-family: '{family}';\n  src: url(\"fonts/{f}\") format(\"woff2\");\n  font-weight: {weight};\n  font-style: {style};\n  font-display: swap;\n}}\n"
        ));
    }
    Ok(out)
}

fn build(dir: &Path, out: &Path) -> Result<usize, String> {
    let repo = dir.parent().ok_or("design_system has no parent")?;
    let ds = DesignSystem::load(dir)?;
    let mut themes = Vec::new();
    for t in &ds.themes {
        themes.push((t.name.clone(), t.label.clone(), ds.resolve_all(t)?));
    }
    let mut app_css = String::new();
    for a in find_app_themes(repo)? {
        let css = fs::read_to_string(repo.join(&a.app).join("design/theme.css"))
            .map_err(|e| format!("{}: {e}", a.app))?;
        app_css.push_str(&css);
        let label = format!(
            "{} ({}, extends {})",
            a.theme.label,
            a.app,
            a.theme.extends.clone().unwrap_or_default()
        );
        themes.push((a.theme.name.clone(), label, ds.resolve_all(&a.theme)?));
    }
    let panels = themes.iter().map(|t| t.2.len()).sum();
    let html = page(&ds, &themes, &app_css);

    let fonts = dir.join("dioxus/fonts");
    let write = |name: &str, text: &str| {
        fs::write(out.join(name), text).map_err(|e| format!("{name}: {e}"))
    };
    fs::create_dir_all(out.join("fonts")).map_err(|e| format!("{}: {e}", out.display()))?;
    write("index.html", &html)?;
    write("fonts.css", &fonts_css(&fonts)?)?;
    for (from, to) in [
        ("dist/tokens.css", "tokens.css"),
        ("dist/base.css", "base.css"),
        ("js/theme.js", "theme.js"),
        // Cloudflare Pages reads it from the build output's root.
        ("preview/_headers", "_headers"),
    ] {
        fs::copy(dir.join(from), out.join(to)).map_err(|e| format!("{from}: {e}"))?;
    }
    for e in fs::read_dir(&fonts).map_err(|e| format!("{}: {e}", fonts.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        fs::copy(e.path(), out.join("fonts").join(e.file_name())).map_err(|e| e.to_string())?;
    }
    Ok(panels)
}

fn main() -> ExitCode {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "dist".into()));
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    match build(&dir, &out) {
        Ok(n) => {
            println!(
                "preview: wrote {} ({n} panels)",
                out.join("index.html").display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use design_system_model::Scheme;

    #[test]
    fn every_combination_gets_a_pinned_panel() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let ds = DesignSystem::load(&dir).unwrap();
        let themes: Vec<_> = ds
            .themes
            .iter()
            .map(|t| (t.name.clone(), t.label.clone(), ds.resolve_all(t).unwrap()))
            .collect();
        let html = page(&ds, &themes, "");
        for t in &ds.themes {
            for c in Combo::ALL {
                assert!(html.contains(&attrs(&t.name, c)), "{} {c}", t.name);
            }
        }
        assert_eq!(html.matches("class=\"panel\"").count(), 20);
        assert!(
            attrs(
                "gamey",
                Combo {
                    scheme: Scheme::Dark,
                    accessible: true
                }
            )
            .contains("data-a11y=\"on\"")
        );
    }

    /// The `_headers` CSP allows no inline script, so the page may only load
    /// scripts from files.
    #[test]
    fn page_has_no_inline_script() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let ds = DesignSystem::load(&dir).unwrap();
        let html = page(&ds, &[], "");
        for tag in html.match_indices("<script").map(|(i, _)| &html[i..]) {
            let open = &tag[..tag.find('>').unwrap()];
            assert!(open.contains(" src=\""), "inline script: {open}");
        }
        assert!(!html.contains(" onclick="), "inline event handler");
    }

    #[test]
    fn headers_file_sends_the_security_headers() {
        let headers = include_str!("../_headers");
        for name in [
            "Content-Security-Policy:",
            "frame-ancestors 'none'",
            "X-Content-Type-Options: nosniff",
            "Referrer-Policy:",
            "Strict-Transport-Security:",
        ] {
            assert!(headers.contains(name), "{name}");
        }
    }

    #[test]
    fn escapes_html() {
        assert_eq!(esc("<a \"b\" & c>"), "&lt;a &quot;b&quot; &amp; c&gt;");
    }
}
