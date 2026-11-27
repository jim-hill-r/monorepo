//! SEC-005 (standards/docs/security.md, "Workflows"): every action a
//! workflow uses is pinned to a full commit SHA with its version in a
//! comment, so a moved tag can't change what runs. Dependabot bumps both.

use std::fs;
use workflow_tests::*;

#[test]
fn test_every_action_is_pinned_to_a_commit() {
    let dir = get_workflow_path("pull-request-ci.yml")
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| panic!("no workflows directory"));
    let mut checked = 0;
    let mut bad = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display())) {
        let path = entry.unwrap_or_else(|e| panic!("{e}")).path();
        if path.extension().and_then(|e| e.to_str()) != Some("yml") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{e}"));
        for (n, line) in text.lines().enumerate() {
            let Some(rest) = line.trim().trim_start_matches("- ").strip_prefix("uses: ") else {
                continue;
            };
            if rest.starts_with("./") {
                continue; // a reusable workflow in this repo
            }
            checked += 1;
            let (action, comment) = rest.split_once(" # ").unwrap_or((rest, ""));
            let sha = action.rsplit_once('@').map_or("", |(_, r)| r);
            let pinned = sha.len() == 40 && sha.chars().all(|c| c.is_ascii_hexdigit());
            if !pinned || !comment.starts_with('v') {
                bad.push(format!("{}:{}: {rest}", path.display(), n + 1));
            }
        }
    }
    assert!(checked > 0, "found no actions to check");
    assert!(
        bad.is_empty(),
        "pin these to a commit SHA with `# vX.Y.Z` after it:\n{}",
        bad.join("\n")
    );
}
