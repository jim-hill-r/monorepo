# 0016. Every project has a user guide, checked by the audit with a shrinking baseline

- Status: Accepted
- Decided by: Jim (project thread, 2026-11-27)
- Date: 2026-11-27
- Asked by: Jim
- Standard: "User docs" in [documentation.md](../documentation.md#user-docs)
- Builds on: [0006](0006-one-file-per-issue.md) (issue files) and
  [0015](0015-code-quality-process.md) (the ratchet pattern)

## Context

Jim (2026-11-27): "Now that our projects are starting to move along, we
should enforce documentation. All projects should have user docs for
supporting our end users."

Every project already needs a README (what it is) and a CONTRIBUTING
(how to build it), DOC-001 to DOC-004. Neither is for the people who use
what a project ships: a cook on cookbook, a player of controlled_chaos, a
developer calling auth_sdk, or an agent running `chores`. On 2026-11-27 no
project had a user guide, so a check that failed on a missing guide would
turn every project's CI red at once.

## Decision

1. **One guide per top-level project**, at `<project>/docs/user/README.md`,
   with four `##` sections: `Who it's for`, `Getting started`, `How to ...`
   and `Getting help`. Sub-projects (`cookbook/web`, `pane/cloudflare`) are
   covered by their parent's guide; a project with more than one product
   adds pages beside the README. Internal tools count: their users are the
   agents and people who run them.
2. **A deterministic check**, `chores user-docs` (Rust, no tokens), run by
   `mise run //:audit` in the hooks and PR CI. DOC-005 (no guide), DOC-006
   (a section missing) and DOC-007 (a section empty) are errors.
3. **A baseline that only shrinks.** `standards/user-docs.toml` lists the
   projects without a passing guide, each with the issue file that tracks
   it, or `on hold`. Listed projects may fail DOC-005 to DOC-007; every
   other project, and every new one, must pass. DOC-008 fails when a listed
   project passes (remove its line), the issue file is gone, or the folder
   is not a project.
4. **The normal agent loop writes the guides.** Each listed project has a
   `write-user-guide.md` issue (epic "User docs"); on-hold projects
   (luggage, starcraft, profiles) get none until they come off hold.

## Consequences

- Cost: $0. The check reads files and adds about a minute to a cold audit
  (building `chores`), cached in CI. Writing 24 guides is ordinary agent
  work against the $25 a month Claude Code budget, one small PR each.
- The check proves a guide exists and has its sections, not that it is
  right. Each guide's issue asks the writer to check every step against
  what ships; evals that test the guide's steps can follow per project.
- Pane can read every guide from one path later.
