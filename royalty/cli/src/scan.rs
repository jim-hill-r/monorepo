//! The Cargo scanner: reads a consumer's dependency graph from
//! `cargo metadata --locked --offline`, finds the `royalty.toml` beside
//! each dependency's manifest, and groups packages into producer families.
//! The rules are "Exact rules" in `docs/royalty-toml.md`. No network call
//! (ROYL-005): `--offline` stops cargo from fetching anything.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use royalty_model::{Dependency, Deployment, RoyaltyFile, Unit, Usage, parse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Metadata {
    packages: Vec<MetaPackage>,
    workspace_members: Vec<String>,
    resolve: Option<Resolve>,
}

#[derive(Debug, Deserialize)]
struct MetaPackage {
    id: String,
    name: String,
    version: String,
    manifest_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: String,
    #[serde(default)]
    deps: Vec<NodeDep>,
}

#[derive(Debug, Deserialize)]
struct NodeDep {
    pkg: String,
    #[serde(default)]
    dep_kinds: Vec<DepKind>,
}

#[derive(Debug, Deserialize)]
struct DepKind {
    /// `None` for a normal dependency, `"dev"` or `"build"`.
    kind: Option<String>,
}

impl NodeDep {
    fn is_normal(&self) -> bool {
        self.dep_kinds.is_empty() || self.dep_kinds.iter().any(|k| k.kind.is_none())
    }
}

/// A family is its name and its payees (by name and GitHub account).
type FamilyKey = (String, Vec<(String, Option<String>)>);

/// One package of the consumer's graph (workspace members left out).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub dependency: Dependency,
    /// Reached through normal dependencies from a workspace member; false
    /// when only dev or build dependencies lead to it.
    pub production: bool,
    pub manifest_dir: PathBuf,
}

/// A producer family the scan found, with the terms that apply.
#[derive(Debug, Clone)]
pub struct ScannedFamily {
    pub family: String,
    /// The newest version's file.
    pub file: RoyaltyFile,
    /// The version whose terms apply.
    pub version: String,
    /// The family's packages in the graph, `name@version`, sorted.
    pub packages: Vec<String>,
    pub usage: Usage,
}

/// Something the consumer should know about the scan.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Notice {
    pub package: String,
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct Scan {
    pub families: Vec<ScannedFamily>,
    /// Packages with no payment terms, `name@version`, sorted.
    pub no_terms: Vec<String>,
    pub notices: Vec<Notice>,
    /// Every package in the graph, for unit counting.
    pub packages: Vec<ScannedPackage>,
}

/// Runs `cargo metadata` in `dir` and scans the result. `deployments` are
/// the consumer's `[[consumer.deployment]]` entries: each manifest's graph
/// is one production deployment.
pub fn scan(dir: &Path, deployments: &[Deployment]) -> Result<Scan> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--locked", "--offline"])
        .current_dir(dir)
        .output()
        .context("can't run cargo metadata (is cargo installed?)")?;
    if !output.status.success() {
        bail!(
            "cargo metadata failed in {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).context("can't read cargo metadata's output")?;
    let deployments: Vec<(String, PathBuf)> = deployments
        .iter()
        .map(|d| {
            let path = dir.join(&d.manifest);
            (d.name.clone(), fs::canonicalize(&path).unwrap_or(path))
        })
        .collect();
    Ok(scan_metadata(&metadata, &deployments))
}

/// The packages reachable from `start` through normal dependencies.
fn reachable<'a>(start: &[&'a str], nodes: &HashMap<&'a str, &'a Node>) -> BTreeSet<&'a str> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut queue: VecDeque<&str> = start.iter().copied().collect();
    while let Some(id) = queue.pop_front() {
        for dep in nodes.get(id).map(|n| n.deps.as_slice()).unwrap_or(&[]) {
            if dep.is_normal() && seen.insert(dep.pkg.as_str()) {
                queue.push_back(dep.pkg.as_str());
            }
        }
    }
    seen
}

fn scan_metadata(metadata: &Metadata, deployments: &[(String, PathBuf)]) -> Scan {
    let members: BTreeSet<&str> = metadata
        .workspace_members
        .iter()
        .map(String::as_str)
        .collect();
    let nodes: HashMap<&str, &Node> = metadata
        .resolve
        .iter()
        .flat_map(|r| &r.nodes)
        .map(|n| (n.id.as_str(), n))
        .collect();

    // Direct: a normal dependency of a workspace member.
    let mut direct: BTreeSet<&str> = BTreeSet::new();
    for member in &members {
        for dep in nodes.get(member).map(|n| n.deps.as_slice()).unwrap_or(&[]) {
            if dep.is_normal() {
                direct.insert(dep.pkg.as_str());
            }
        }
    }

    // Production: reachable from a member through normal edges only.
    let member_list: Vec<&str> = members.iter().copied().collect();
    let production = reachable(&member_list, &nodes);

    let mut scan = Scan::default();

    // Each deployment's graph: the packages its manifest's member reaches.
    let mut deployment_graphs: Vec<BTreeSet<&str>> = Vec::new();
    for (name, manifest) in deployments {
        let member = metadata.packages.iter().find(|p| {
            members.contains(p.id.as_str())
                && fs::canonicalize(&p.manifest_path).unwrap_or(p.manifest_path.clone())
                    == *manifest
        });
        match member {
            Some(member) => deployment_graphs.push(reachable(&[member.id.as_str()], &nodes)),
            None => scan.notices.push(Notice {
                package: name.clone(),
                message: format!(
                    "deployment manifest {} is not a member of this workspace; not counted",
                    manifest.display()
                ),
            }),
        }
    }
    // family key (family + payees) -> packages and their files.
    let mut grouped: BTreeMap<FamilyKey, Vec<(ScannedPackage, RoyaltyFile)>> = BTreeMap::new();
    let mut packages: Vec<&MetaPackage> = metadata
        .packages
        .iter()
        .filter(|p| !members.contains(p.id.as_str()))
        .collect();
    packages.sort_by(|a, b| (&a.name, &a.version).cmp(&(&b.name, &b.version)));

    for package in packages {
        let manifest_dir = package
            .manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let scanned = ScannedPackage {
            id: package.id.clone(),
            name: package.name.clone(),
            version: package.version.clone(),
            dependency: if direct.contains(package.id.as_str()) {
                Dependency::Direct
            } else {
                Dependency::Transitive
            },
            production: production.contains(package.id.as_str()),
            manifest_dir: manifest_dir.clone(),
        };
        scan.packages.push(scanned.clone());
        let label = format!("{}@{}", package.name, package.version);

        let Ok(text) = fs::read_to_string(manifest_dir.join("royalty.toml")) else {
            scan.no_terms.push(label);
            continue;
        };
        let parsed = parse(&text);
        let Some(file) = parsed.valid().cloned() else {
            let errors = parsed.findings.iter().filter(|f| f.is_error()).count();
            scan.notices.push(Notice {
                package: label.clone(),
                message: format!(
                    "its royalty.toml has {errors} error(s) (run royalty check on it); treated as no terms"
                ),
            });
            scan.no_terms.push(label);
            continue;
        };
        if !file.covers(&package.name) {
            scan.notices.push(Notice {
                package: label.clone(),
                message: format!(
                    "its royalty.toml is for {:?}, not this package; ignored",
                    file.project.packages
                ),
            });
            scan.no_terms.push(label);
            continue;
        }
        if !file.has_terms {
            scan.no_terms.push(label);
            continue;
        }
        let mut payees: Vec<(String, Option<String>)> = file
            .payees
            .iter()
            .map(|p| (p.name.clone(), p.github.clone()))
            .collect();
        payees.sort();
        grouped
            .entry((file.project.family.clone(), payees))
            .or_default()
            .push((scanned, file));
    }

    let mut families_seen: BTreeMap<String, usize> = BTreeMap::new();
    for (family, _) in grouped.keys() {
        *families_seen.entry(family.clone()).or_default() += 1;
    }

    for ((family, _payees), members) in grouped {
        let Some((newest, file)) = members
            .iter()
            .max_by(|a, b| compare_versions(&a.0.version, &b.0.version))
            .map(|(p, f)| (p.version.clone(), f.clone()))
        else {
            continue;
        };
        let dependency = if members
            .iter()
            .any(|(p, _)| p.dependency == Dependency::Direct)
        {
            Dependency::Direct
        } else {
            Dependency::Transitive
        };
        let is_production = members.iter().any(|(p, _)| p.production);
        let mut usage = Usage {
            production: is_production,
            dependency,
            due: true,
            measured: BTreeMap::new(),
        };
        // Counted from the graph: the family is used, in this one project.
        usage.measured.insert(Unit::Flat, 1);
        usage.measured.insert(Unit::Project, 1);
        // Deployments: each listed deployment whose graph holds the family,
        // or the project itself when none is listed.
        let deployed = if deployments.is_empty() {
            u64::from(is_production)
        } else {
            deployment_graphs
                .iter()
                .filter(|graph| members.iter().any(|(p, _)| graph.contains(p.id.as_str())))
                .count() as u64
        };
        usage.measured.insert(Unit::Deployment, deployed);

        let mut labels: Vec<String> = members
            .iter()
            .map(|(p, _)| format!("{}@{}", p.name, p.version))
            .collect();
        labels.sort();
        labels.dedup();
        let shown = if families_seen.get(&family).copied().unwrap_or(0) > 1 {
            scan.notices.push(Notice {
                package: labels.join(", "),
                message: format!(
                    "family \"{family}\" appears with different payees; billed as separate families (a fork or a hijacked release?)"
                ),
            });
            format!("{family} ({})", labels.join(", "))
        } else {
            family.clone()
        };
        scan.families.push(ScannedFamily {
            family: shown,
            file,
            version: newest,
            packages: labels,
            usage,
        });
    }
    scan.notices.sort();
    scan
}

/// Compares two versions by semver precedence: numeric major, minor and
/// patch, and a pre-release sorts before its release.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    fn split(v: &str) -> (Vec<u64>, Option<&str>) {
        let v = v.split('+').next().unwrap_or(v);
        let (core, pre) = match v.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (v, None),
        };
        (
            core.split('.').map(|n| n.parse().unwrap_or(0)).collect(),
            pre,
        )
    }
    let (a_core, a_pre) = split(a);
    let (b_core, b_pre) = split(b);
    a_core.cmp(&b_core).then_with(|| match (a_pre, b_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => x.cmp(y),
    })
}

#[cfg(test)]
mod tests {
    use super::compare_versions;
    use std::cmp::Ordering;

    #[test]
    fn versions_compare_by_semver() {
        assert_eq!(compare_versions("1.10.0", "1.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("2.0.0-beta.1", "2.0.0"), Ordering::Less);
        assert_eq!(compare_versions("0.4.2", "0.4.2"), Ordering::Equal);
    }
}
