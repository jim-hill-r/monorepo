//! Reading the repo's files, in the working tree or at a commit, through
//! git so only tracked files count.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// Runs git in `root` and returns its stdout.
pub fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn split_z(text: &str) -> Vec<String> {
    text.split('\0')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Tracked files under `prefix` in the working tree. Outside a git repo it
/// walks the directory instead, skipping hidden directories.
pub fn tracked(root: &Path, prefix: &str) -> Vec<String> {
    if let Ok(out) = git(root, &["ls-files", "-z", "--", prefix]) {
        return split_z(&out);
    }
    let mut out = Vec::new();
    walk(root, prefix, &mut out);
    out.sort();
    out
}

fn walk(root: &Path, rel: &str, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(root.join(rel)) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{rel}/{name}");
        match entry.file_type() {
            Ok(t) if t.is_dir() && !name.starts_with('.') => walk(root, &path, out),
            Ok(t) if t.is_file() => out.push(path),
            _ => {}
        }
    }
}

/// Every file in the tree at `rev`.
pub fn tree(root: &Path, rev: &str) -> Result<Vec<String>> {
    Ok(split_z(&git(
        root,
        &["ls-tree", "-r", "-z", "--name-only", rev],
    )?))
}

/// The text of each of `paths` at `rev`, through one `git cat-file
/// --batch`. Missing and non-UTF-8 files are left out.
pub fn read_at(root: &Path, rev: &str, paths: &[String]) -> Result<BTreeMap<String, String>> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .context("running git cat-file")?;
    let mut stdin = child.stdin.take().context("git stdin")?;
    let input: String = paths.iter().map(|p| format!("{rev}:{p}\n")).collect();
    let writer = std::thread::spawn(move || stdin.write_all(input.as_bytes()));
    let mut reader = BufReader::new(child.stdout.take().context("git stdout")?);
    let mut out = BTreeMap::new();
    for path in paths {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 {
            break;
        }
        let fields: Vec<&str> = header.split_whitespace().collect();
        if fields.len() != 3 {
            continue; // "<object> missing"
        }
        let size: usize = fields[2].parse().context("git cat-file size")?;
        let mut body = vec![0; size + 1]; // the object and a newline
        reader.read_exact(&mut body)?;
        body.pop();
        if fields[1] == "blob"
            && let Ok(text) = String::from_utf8(body)
        {
            out.insert(path.clone(), text);
        }
    }
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("git cat-file writer panicked"))??;
    child.wait()?;
    Ok(out)
}

/// The text of each of `paths` in the working tree.
pub fn read_here(root: &Path, paths: &[String]) -> BTreeMap<String, String> {
    paths
        .iter()
        .filter_map(|p| Some((p.clone(), fs::read_to_string(root.join(p)).ok()?)))
        .collect()
}
