---
kind: todo
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

Most merge conflicts between open PRs land in three files every new command or check appends to: chores/src/main.rs (the Command enum and its match), chores/README.md (the command examples) and standards/scripts/audit.sh (the list of checks). On 2026-11-28 all nine main merges that cost Jim an approval were conflict resolutions there ("Keep it mergeable" in standards/docs/pull-requests.md). Make them append-safe: keep the Command variants and match arms in alphabetical order with one module call each (so new ones don't all land at the end), move each command's README example into its own section in alphabetical order, and have audit.sh run its checks from a sorted list (or a checks directory) instead of one long appended block. Do it in one PR when few chores PRs are open, so the reshuffle itself conflicts with as little as possible.
