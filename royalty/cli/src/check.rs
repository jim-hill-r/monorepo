//! `royalty check`: read one royalty.toml and print its findings.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use royalty_model::{Level, parse};

/// The file a path names: itself, or `royalty.toml` inside a directory.
pub fn royalty_file(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("royalty.toml")
    } else {
        path.to_path_buf()
    }
}

pub fn run(path: &Path, strict: bool) -> Result<ExitCode> {
    let file = royalty_file(path);
    let text =
        fs::read_to_string(&file).with_context(|| format!("can't read {}", file.display()))?;
    let parsed = parse(&text);
    let shown = file.display();
    for finding in &parsed.findings {
        let mut line = format!(
            "{}[{}] {shown}",
            finding.level.as_str(),
            finding.code.as_str()
        );
        if !finding.at.is_empty() {
            line.push_str(&format!(" {}", finding.at));
        }
        line.push_str(&format!(": {}", finding.message));
        if let Some(requirement) = finding.requirement {
            line.push_str(&format!(" ({requirement})"));
        }
        println!("{line}");
    }
    let errors = parsed.findings.iter().filter(|f| f.is_error()).count();
    let warnings = parsed
        .findings
        .iter()
        .filter(|f| f.level == Level::Warning)
        .count();
    let fails = errors > 0 || (strict && warnings > 0);
    if parsed.findings.is_empty() {
        let meaning = match parsed.valid() {
            Some(f) if !f.has_terms && f.consumer.is_none() => " (no payment terms)",
            _ => "",
        };
        println!("royalty check: {shown} ok{meaning}");
    } else {
        println!("royalty check: {shown}: {errors} error(s), {warnings} warning(s)");
    }
    Ok(if fails {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}
