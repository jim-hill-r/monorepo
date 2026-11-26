//! `design_system_cli`: render the design system's outputs from its token
//! files, and check them.
//!
//! ```text
//! design_system_cli render [--app <dir>]   write dist/, palette/src/generated.rs,
//!                                          docs/contrast.md and every app theme
//! design_system_cli check [--app <dir>]    contrast, completeness, outputs up to date
//! ```
//!
//! `--app <dir>` (relative to the repo root, e.g. `bear_down`) limits either
//! command to that app's `design/theme.toml`. `--dir <path>` points at the
//! `design_system` directory (default: the one this binary was built from).

mod css;
mod json;
mod report;
mod rust;

use design_system_model::{
    AppTheme, DesignSystem, Report, Resolved, check, find_app_themes, load_app_theme,
    resolve::validate_shared, validate,
};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

struct Args {
    command: String,
    app: Option<String>,
    dir: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let mut app = None;
    let mut dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--app" => app = Some(args.next().ok_or("--app needs a directory")?),
            "--dir" => dir = PathBuf::from(args.next().ok_or("--dir needs a path")?),
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if !matches!(command.as_str(), "render" | "check") {
        return Err(
            "usage: design_system_cli render|check [--app <dir>] [--dir <design_system>]".into(),
        );
    }
    let dir = dir
        .canonicalize()
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(Args { command, app, dir })
}

/// Everything loaded and resolved, with the problems found on the way.
struct State {
    ds: DesignSystem,
    themes: Vec<(String, String, Vec<Resolved>)>,
    apps: Vec<(AppTheme, Vec<Resolved>)>,
    reports: Vec<Report>,
    problems: Vec<String>,
}

fn load(dir: &Path, repo: &Path, only_app: Option<&str>) -> Result<State, String> {
    let ds = DesignSystem::load(dir)?;
    let mut problems: Vec<String> = validate_shared(&ds);
    let mut themes = Vec::new();
    for t in &ds.themes {
        let found = validate(t);
        if found.is_empty() {
            themes.push((t.name.clone(), t.label.clone(), ds.resolve_all(t)?));
        }
        problems.extend(
            found
                .into_iter()
                .map(|p| format!("tokens/themes/{}.toml: {p}", t.name)),
        );
    }
    let found_apps = match only_app {
        Some(a) => vec![load_app_theme(repo, a)?],
        None => find_app_themes(repo)?,
    };
    let mut apps = Vec::new();
    for a in found_apps {
        let mut found = validate(&a.theme);
        if a.theme.extends.is_none() {
            found.push("an app theme must name the standard theme it extends".into());
        }
        if found.is_empty() {
            let resolved = ds.resolve_all(&a.theme)?;
            apps.push((a, resolved));
        } else {
            problems.extend(
                found
                    .into_iter()
                    .map(|p| format!("{}/{}: {p}", a.app, design_system_model::APP_THEME_FILE)),
            );
        }
    }
    let mut reports = Vec::new();
    for r in themes
        .iter()
        .flat_map(|t| &t.2)
        .chain(apps.iter().flat_map(|a| &a.1))
    {
        let rep = check(&ds, r)?;
        problems.extend(rep.failures());
        reports.push(rep);
    }
    Ok(State {
        ds,
        themes,
        apps,
        reports,
        problems,
    })
}

fn minimums(ds: &DesignSystem) -> String {
    let mut out = String::from("| Rule | Standard | Accessible |\n| --- | --- | --- |\n");
    for p in &ds.pairs.pair {
        out.push_str(&format!(
            "| {}: `{}` on `{}` | {}:1 | {}:1 |\n",
            p.why,
            p.fg.join("`, `"),
            p.bg.join("`, `"),
            p.min,
            p.accessible
        ));
    }
    let d = &ds.pairs.data;
    out.push_str(&format!(
        "| Data palette on `{}` | {}:1, OKLab distance {} | {}:1, distance {} for normal vision and each simulated colour blindness |\n",
        d.on.join("`, `"),
        d.min,
        d.min_distance,
        d.accessible,
        d.accessible_distance
    ));
    out
}

/// Every generated file and its contents. With `only_app`, just that app's.
fn outputs(state: &State, dir: &Path, repo: &Path, only_app: bool) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    for (a, resolved) in &state.apps {
        let t = &a.theme;
        let design = repo.join(&a.app).join("design");
        let extends = t.extends.clone().unwrap_or_default();
        out.push((
            design.join("theme.css"),
            css::app_css(&a.app, &t.name, &extends, resolved),
        ));
        if t.rust == Some(true) {
            out.push((
                design.join("theme.rs"),
                rust::app_rs(&a.app, &t.name, &t.label, resolved),
            ));
        }
    }
    if only_app {
        return out;
    }
    let names: Vec<(String, Vec<Resolved>)> = state
        .themes
        .iter()
        .map(|(n, _, r)| (n.clone(), r.clone()))
        .collect();
    out.push((dir.join("dist/tokens.css"), css::tokens_css(&names)));
    out.push((
        dir.join("dist/tokens.json"),
        json::tokens_json(&state.themes),
    ));
    out.push((
        dir.join("palette/src/generated.rs"),
        rust::palette_rs(&state.themes),
    ));

    let reports_for = |theme: &str| -> Vec<&Report> {
        state.reports.iter().filter(|r| r.theme == theme).collect()
    };
    let themes: Vec<report::ThemeReport<'_>> = state
        .themes
        .iter()
        .map(|(n, label, _)| report::ThemeReport {
            title: label.clone(),
            reports: reports_for(n),
        })
        .collect();
    let apps: Vec<report::ThemeReport<'_>> = state
        .apps
        .iter()
        .map(|(a, _)| report::ThemeReport {
            title: format!(
                "{} (`{}`, the {} app theme, extends {})",
                a.theme.label,
                a.theme.name,
                a.app,
                a.theme.extends.clone().unwrap_or_default()
            ),
            reports: reports_for(&a.theme.name),
        })
        .collect();
    out.push((
        dir.join("docs/contrast.md"),
        report::contrast_md(&themes, &apps, &minimums(&state.ds)),
    ));
    out
}

fn base_css(dir: &Path, repo: &Path) -> Result<(PathBuf, String), String> {
    let read = |p: PathBuf| fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()));
    let reset = read(repo.join("standards/docs/reset.css"))?;
    let elements = read(dir.join("css/elements.css"))?;
    Ok((
        dir.join("dist/base.css"),
        format!(
            "/* {}\n * Sources: standards/docs/reset.css, then design_system/css/elements.css.\n * Load after dist/tokens.css. */\n\n{reset}\n{elements}",
            css::HEADER
        ),
    ))
}

fn rel(path: &Path, repo: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn run() -> Result<bool, String> {
    let args = parse_args()?;
    let dir = args.dir;
    let repo = dir
        .parent()
        .ok_or("the design_system directory has no parent")?
        .to_path_buf();
    let state = load(&dir, &repo, args.app.as_deref())?;
    let mut files = outputs(&state, &dir, &repo, args.app.is_some());
    if args.app.is_none() {
        files.push(base_css(&dir, &repo)?);
    }

    let combos = state.reports.len();
    let pairs: usize = state.reports.iter().map(|r| r.pairs.len()).sum();
    match args.command.as_str() {
        "render" => {
            for (path, text) in &files {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
                }
                fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
                println!("wrote {}", rel(path, &repo));
            }
            for p in &state.problems {
                eprintln!("warning: {p}");
            }
            Ok(true)
        }
        _ => {
            let mut problems = state.problems;
            for (path, text) in &files {
                match fs::read_to_string(path) {
                    Ok(on_disk) if on_disk == *text => {}
                    Ok(_) => problems.push(format!(
                        "{} is out of date; run `mise run render` in design_system",
                        rel(path, &repo)
                    )),
                    Err(_) => problems.push(format!(
                        "{} is missing; run `mise run render` in design_system",
                        rel(path, &repo)
                    )),
                }
            }
            for p in &problems {
                eprintln!("error: {p}");
            }
            if problems.is_empty() {
                println!(
                    "check: ok ({} standard themes, {} app themes, {combos} combinations, {pairs} pairs)",
                    state.themes.len(),
                    state.apps.len()
                );
            } else {
                eprintln!("check: {} problem(s)", problems.len());
            }
            Ok(problems.is_empty())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
