# Answer a settings check

- **Kind:** recurring. Each setting in `standards/settings.toml` has its own
  period (`every`, 30 or 90 days).
- **Status:** ongoing. Each due check is an
  `standards/docs/issues/human/settings-check-<id>.md` task, written by the weekly digest.
- **Time:** about 1 to 3 minutes per check
- **Cost:** $0

## What it's for

Settings made by hand in a dashboard (connectors, secrets, token scopes,
Auth0 apps, billing) break silently. Each one is listed once in
`standards/settings.toml` with where it is and what it should be; the
"Human checks" section of [evals.md](../evals.md#human-checks) has the
design.

## Why a person

No file in the repo holds these settings, and agents have no read access
to the pages that show them (and shouldn't: several need admin or billing
access).

## Before you start

- Signed in to the vendor the check names.

## Steps

1. Open the task (or the digest's "Checks due" line). It says the setting's
   id, what it should be, and where to look.
2. Open that page (`where` in `standards/settings.toml`) and compare it
   with `expect`.
3. Reply in the Claude project with "ok", or what is wrong, naming the id.

## Where secrets go

None. Never paste a secret's value in a reply: say only that it exists or
not, and its scopes.

## Verify

An agent records your answer in `standards/settings-checks.jsonl` with
`evals_cli human-checks answer` and deletes the task in a PR. An answer
other than "ok" files a fix, and the check comes back next week until it's
"ok".

## Undo or rotate

A wrong answer is fixed by answering again; the latest answer counts.
