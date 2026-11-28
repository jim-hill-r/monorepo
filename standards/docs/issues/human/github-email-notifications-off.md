---
added: 2026-11-28
runbook: standards/docs/runbooks/github-email-notifications-off.md
---

## Requirement

Turn off every GitHub email notification on the three accounts
`jimhillr`, `jimhillr-agt` and `jimhillr-reviewer` (the last once it
exists, see `jimhillr-reviewer-account.md`): the email boxes on each
account's Notifications settings page, the per-repository watch emails,
and optional email on its Emails page. Web and GitHub Mobile
notifications stay on, so nothing is missed; only email stops.

## Why it can't be automated

GitHub has no API for an account's notification or email settings: they
are only on each account's own signed-in settings pages, and agents never
hold a password or session for these accounts (decision 0010).

## Runbook

[Turn off GitHub email notifications on the three jimhillr accounts](../../runbooks/github-email-notifications-off.md)
