# chores

Deterministic commands for routine agent work, so agents spend tokens only
on novel work (Jim, 2026-11-27). Each command applies rules already written
in `standards/docs/` and is called from a skill in `.agents/skills/`, which
leaves only the judgment to the model. Why and what comes next is in the
[baked skills plan](../standards/docs/plans/2026-11-baked-skills.md).

It reads and writes only files in the working tree. It never uses the
network and never calls GitHub: agents do that with their own GitHub tools.

## Commands

Run from anywhere in the repo (cargo is in every sandbox; the first run
compiles it, about a minute):

```bash
c() { cargo run --quiet --manifest-path "$(git rev-parse --show-toplevel)/chores/Cargo.toml" -- "$@"; }

# The next issue, in the start-a-new-task order, with its text and epic notes.
c next-task
c next-task --project cookbook            # scoped to one project
c next-task --count 5 --exclude pane,epic:Cost check   # one per project, for new threads
c next-task --skip cookbook/docs/issues/x.md   # an open PR already has it
c next-task --count 5 --working cookbook,pane,repo   # prefer products while under half
c next-task --json

# A PR description in the shape pull-requests.md asks for, checked against
# PR-001 (exit 1 with the warnings when a section is missing or empty, an
# attention tag is unknown, or a summary is over its word limit).
c pr-body --login jimhillr-agt --requested-by Jim --thread-url <url> \
  --what "At most 50 words." --why "At most 50 words." \
  --cost "Under 25 words (default None.)" --risk "Under 25 words (default None.)" \
  --details "Hooks passed (audit, chores CI). [The plan](standards/docs/plans/x.md)." \
  --footer "<platform lines>" --out body.md
c pr-body --check body.md                 # check an existing description

# The Attention section pr-body puts under the attribution lines: one tag
# each for requirement changes (red for a must), cost file changes and
# threat/risk file changes, against the base.
c badges --base origin/main

# The human-review label (PR-004): exit 1 while a PR with a red attention tag
# or a cost change (diff, description or `cost` label) lacks a human's approval
# of its head commit. The Human review workflow runs it and labels the PR;
# agents rarely need to.
c gate --author jimhillr-agt --head <sha> --body body.md --labels cost,docs \
  --reviews reviews.json --json gate.json

# Issue files that pass the audit the first time.
c new-issue --folder cookbook --slug fix-week-header --kind fix --priority backlog \
  --text "The week header overlaps on phones; see ..."
c new-issue --folder . --slug cost-phase-4 --epic "Cost check / Phase 4: evals" --order 10 --text "..."
c new-human-task --slug make-state-passwords --title "Make the OpenTofu state passwords" \
  --requirement "..." --why "..." --unblocks "IaC applies on main"
```

```bash
# Every project's user guide (DOC-005 to DOC-008) against the baseline in
# standards/user-docs.toml; run by `mise run //:audit`. Exit 1 on an error.
c user-docs
c user-docs --json
```

`new-human-task` starts the runbook from
`standards/docs/runbooks/TEMPLATE.md` (title and status filled in) unless
`--runbook` names one that exists; the agent still writes its sections.

| Command | Skill that calls it | Replaces |
| --- | --- | --- |
| `next-task` | `start-a-new-task` | Reading the issue list, `standards/docs/issues/README.md`, folder READMEs and issue files to apply the pick order |
| `pr-body`, `badges` | `open-pull-request` | Rebuilding the description shape, fixing PR-001 warnings after the fact, and Jim judging from the file list how closely to look |
| `gate` | none (the `Human review` workflow) | Judging by hand whether a PR's red or cost tags let anyone but a human approve or auto-merge it |
| `user-docs` | `mise run //:audit` (any skill's hooks) | A bash check, and a reviewer noticing a project has no user guide |
| `new-issue`, `new-human-task` | `write-issue` | Re-reading the issue and runbook formats, and an audit retry |

## Adding a command

Pick routine work from the plan's ranking, write the rule as a pure
function with tests that pin it to the examples in the standards, add a
subcommand in `src/main.rs`, and call it from a skill in the same PR. A
skill and its command must agree; fix both together.
