# Open the terrars upstream PRs

- **Kind:** one-time, optional
- **Status:** open (`infra/docs/issues/human/open-terrars-upstream-prs.md`)
- **Time:** about 10 minutes
- **Cost:** $0

## What it's for

Two fixes the `infra` project carries in `infra/upstream/`:
`terrars-generate-opentofu-schema-key.patch` makes `terrars-generate` work
with `"binary": "tofu"`, and `terrars-import-blocks.patch` adds
`Stack::add_import`. Once they merge upstream, `infra` can drop its own
copies (`infra/docs/issues/iac-terrars-import-upstream.md`).

## Why a person

Agents can only reach jimhillr/monorepo, so they can't fork
andrewbaxter/terrars or open a PR there.

## Before you start

- A GitHub account and a machine with git (or GitHub Codespaces on the
  fork, so nothing is needed locally).

## Steps

1. Fork [andrewbaxter/terrars](https://github.com/andrewbaxter/terrars).
2. In the fork (a Codespace works), on `master`, for each patch:
   `git checkout -b <name> master`, then
   `curl -sL https://raw.githubusercontent.com/jimhillr/monorepo/main/infra/upstream/<file>.patch | git am`,
   and `git push -u origin <name>`.
3. Open a PR from each branch to andrewbaxter/terrars `master`; the commit
   message is already the description.

## Where secrets go

None.

## Verify

Reply with the two PR links. When they merge, delete the issue file.

## Undo or rotate

Close the PRs on GitHub.
