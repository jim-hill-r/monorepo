//! The design system's model: read the token files, resolve each theme in
//! each combination of scheme and accessible mode, and check contrast.
//!
//! See `design_system/README.md` and the plan in
//! `design_system/docs/plans/2026-11-design-system.md`.

pub mod check;
pub mod color;
pub mod resolve;
pub mod spec;

pub use check::{Report, check};
pub use color::{Rgba, Vision};
pub use resolve::{Combo, Resolved, resolve, validate};
pub use spec::{Scheme, ThemeFile};

use spec::{Base, Modes, Pairs};
use std::fs;
use std::path::{Path, PathBuf};

/// The standard themes, in the order every output lists them.
pub const STANDARD_THEMES: [&str; 5] = ["earthy", "airy", "professional", "colorful", "gamey"];

/// Where an app keeps its custom theme, relative to the app's directory.
pub const APP_THEME_FILE: &str = "design/theme.toml";

/// All of `design_system/tokens/`.
#[derive(Clone, Debug)]
pub struct DesignSystem {
    pub base: Base,
    pub modes: Modes,
    pub pairs: Pairs,
    /// The standard themes, in `STANDARD_THEMES` order.
    pub themes: Vec<ThemeFile>,
}

/// An app's custom theme.
#[derive(Clone, Debug)]
pub struct AppTheme {
    /// The app's directory relative to the repo root, with `/` separators.
    pub app: String,
    pub theme: ThemeFile,
}

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

impl DesignSystem {
    /// Read `<dir>/tokens/`, where `dir` is the `design_system` directory.
    pub fn load(dir: &Path) -> Result<Self, String> {
        let tokens = dir.join("tokens");
        let mut themes = Vec::new();
        for name in STANDARD_THEMES {
            let path = tokens.join("themes").join(format!("{name}.toml"));
            let theme: ThemeFile = read_toml(&path)?;
            if theme.name != name {
                return Err(format!("{}: name must be {name:?}", path.display()));
            }
            themes.push(theme);
        }
        Ok(Self {
            base: read_toml(&tokens.join("base.toml"))?,
            modes: read_toml(&tokens.join("modes.toml"))?,
            pairs: read_toml(&tokens.join("pairs.toml"))?,
            themes,
        })
    }

    pub fn theme(&self, name: &str) -> Option<&ThemeFile> {
        self.themes.iter().find(|t| t.name == name)
    }

    /// The standard theme a custom theme extends, if it exists.
    pub fn parent_of(&self, theme: &ThemeFile) -> Option<&ThemeFile> {
        theme.extends.as_deref().and_then(|n| self.theme(n))
    }

    /// Every combination of a theme that it has (custom themes may be
    /// light only).
    pub fn combos(theme: &ThemeFile) -> Vec<Combo> {
        let schemes = theme.schemes();
        Combo::ALL
            .into_iter()
            .filter(|c| schemes.contains(&c.scheme))
            .collect()
    }

    /// Resolve every combination of `theme`.
    pub fn resolve_all(&self, theme: &ThemeFile) -> Result<Vec<Resolved>, String> {
        let parent = match &theme.extends {
            Some(n) => Some(
                self.theme(n)
                    .ok_or_else(|| format!("{}: extends unknown theme {n:?}", theme.name))?,
            ),
            None => None,
        };
        Self::combos(theme)
            .into_iter()
            .map(|c| resolve(self, theme, parent, c))
            .collect()
    }
}

/// Load one app's custom theme. `app` is relative to `repo`.
pub fn load_app_theme(repo: &Path, app: &str) -> Result<AppTheme, String> {
    let theme = read_toml(&repo.join(app).join(APP_THEME_FILE))?;
    Ok(AppTheme {
        app: app.to_string(),
        theme,
    })
}

/// Find every `<app>/design/theme.toml` under `repo`, skipping hidden
/// directories, `target` and `node_modules`. Sorted by app path.
pub fn find_app_themes(repo: &Path) -> Result<Vec<AppTheme>, String> {
    fn walk(repo: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut dirs: Vec<PathBuf> = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if !is_dir || name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            dirs.push(entry.path());
        }
        if dir.join(APP_THEME_FILE).is_file() {
            let rel = dir.strip_prefix(repo).unwrap_or(dir);
            let rel: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().to_string())
                .collect();
            out.push(rel.join("/"));
        }
        for d in dirs {
            walk(repo, &d, out)?;
        }
        Ok(())
    }
    let mut apps = Vec::new();
    walk(repo, repo, &mut apps)?;
    apps.sort();
    apps.iter().map(|a| load_app_theme(repo, a)).collect()
}
