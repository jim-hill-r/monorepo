---
added: 2026-11-26
unblocks: "dropping infra's own import-block code once terrars has Stack::add_import, and generating bindings with OpenTofu from a released terrars-generate"
runbook: standards/docs/runbooks/terrars-upstream-prs.md
---

## Requirement

Open two pull requests on [andrewbaxter/terrars](https://github.com/andrewbaxter/terrars), one per patch in `infra/upstream/` (agents can only reach this repo, so they can't open them). Each file is a `git format-patch` commit with its own message: in a fork of terrars, run `git am infra/upstream/<file>.patch` on `master` and open a PR from it. `terrars-generate-opentofu-schema-key.patch` makes `terrars-generate` work with `"binary": "tofu"`; `terrars-import-blocks.patch` adds `Stack::add_import`. Cost: $0, about 10 minutes. Once they merge, delete this file; the agent issue `infra/docs/issues/iac-terrars-import-upstream.md` covers the switch.

## Why it can't be automated

Agents can only reach jimhillr/monorepo, so they can't fork andrewbaxter/terrars or open a PR there.

## Runbook

[Open the terrars upstream PRs](../../../../standards/docs/runbooks/terrars-upstream-prs.md)
