# <Task in plain words>

<!-- Copy this file to standards/docs/runbooks/<name>.md and fill in every
section. A section that doesn't apply says "None" and why. See README.md. -->

- **Kind:** one-time | recurring (every N days) | on demand (when X happens)
- **Status:** open (<project>/docs/issues/human/<file>.md) | done <date> (keep for a rebuild)
- **Time:** about N minutes
- **Cost:** $0 (which free tier it uses, and how close to its limit)

## What it's for

What this sets up or checks, and what breaks or stays blocked without it.

## Why a person

Why no workflow or agent can do it: the vendor has no API for it, it needs
a signed-in account, it creates the first credential that automation would
need, it is a sign-off an agent may not give itself, or it deletes
something (IaC never deletes, decision 0007).

## Before you start

- Accounts you must be signed in to, and the role or permission needed.
- Anything that must already be done (other runbooks).

## Steps

1. Exact page (menu path or URL) and what to click or type.
2. ...

## Where secrets go

Each secret's name and where it is stored (a repository secret, or a
secret of the `infra` or `infra-credentials` environment). Never in git,
in OpenTofu state, a chat or a note. "None" if the task makes no secret.

## Verify

How to tell it worked: a page to look at, a workflow run that now passes,
or an agent to ask. Then tell any thread "done" so it deletes the issue.

## Undo or rotate

How to roll it back, or make a new credential and retire the old one.
