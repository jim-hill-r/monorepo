# Runbooks for human tasks

Every task in this repo's architecture that only a person can do has a
runbook here, including ones done once long ago (creating an account, the
first token), so the whole setup could be rebuilt from these pages alone.
Jim asked for this on 2026-11-27: "whenever asking for human help, document
the requirement, why it can't be automated and create a run book. Link the
run book in the task."

## The rule

- **A human task ships with its runbook.** A PR that adds a file to
  `issues/human/` adds or reuses a runbook here in the same PR. The issue
  file has `runbook: standards/docs/runbooks/<name>.md` in its front matter
  and three sections, `## Requirement`, `## Why it can't be automated` and
  `## Runbook` (a link to this folder). `mise run //:audit` fails
  (ISS-004) if any is missing or the runbook file doesn't exist. The
  format is under "Human issues" in
  [issue-management.md](../issue-management.md#human-issues).
- **Runbooks outlive their issues.** When a one-time task is done, delete
  the issue file but keep the runbook, set its status to "done <date>",
  and fix anything the person found to be different on the day.
- **Automate first.** A task only goes to a person when automation can't
  do it (no API, a signed-in account, a first credential, a sign-off, a
  delete). Credentials that a bootstrap credential could make go to the
  minting workflow instead (Jim, 2026-11-27).
- **No secret values anywhere here**, only a secret's name and where it is
  stored.
- Copy [TEMPLATE.md](TEMPLATE.md) for a new runbook.

## Every human task

### Open now

| Runbook | Kind | Issue |
| --- | --- | --- |
| [Set up the infra-credentials environment](infra-credentials-environment.md) | one-time | `infra-credentials-environment.md` |
| [Create the Auth0 IaC applications](auth0-iac-credentials.md) | one-time | `auth0-iac-credentials.md` |
| [Create the Cloudflare IaC tokens](cloudflare-iac-credentials.md) | one-time | `cloudflare-iac-credentials.md` |
| [Make the OpenTofu state passwords](tofu-state-passwords.md) | one-time | `tofu-state-passwords.md` |
| [Change the Cloudflare deploy token's permissions](cloudflare-deploy-token.md) | on demand | `cloudflare-api-token-workers-d1-edit.md`, `cloudflare-dashboard-turn-web-analytics-production.md` |
| [Turn on GitHub code security](github-code-security.md) | one-time | `github-settings-code-security-turn-secret.md` |
| [Require CI on the main ruleset](github-ruleset-required-check.md) | one-time | `github-settings-rules-rulesets-default-ruleset.md` |
| [Sign up for PostHog](posthog-sign-up.md) | one-time | `create-posthog-cloud-free-plan-no.md` |
| [Delete old Pages projects](delete-old-pages-projects.md) | on demand | `delete-old-pages-projects-after-jimhillr-rename.md` |
| [Open the terrars upstream PRs](terrars-upstream-prs.md) | one-time, optional | `open-terrars-upstream-prs.md` |
| [Make the Android release key](android-release-key.md) | one-time, optional | `optional-create-android-release-key-controlled.md` |
| [Set the TOFU_STATE_ORIGIN variable](tofu-state-origin-variable.md) | one-time | `tofu-state-origin-variable.md` |
| [Check there is no payment method](payment-method-check.md) | recurring, every 30 days | `check-there-still-no-payment-method.md` |

Coming in open PRs, written by the infrastructure thread in the same
format: the infra GitHub App (`github-app-infra.md`, PR #728), the
`TOFU_STATE_ORIGIN` variable (PR #726), and the one setup task that the
token-minting workflow will leave in place of several rows above.

### Recurring and on demand

| Runbook | When |
| --- | --- |
| [Answer a settings check](settings-check.md) | A `settings-check-<id>.md` task appears (weekly digest) |
| [Approve a credential change run](approve-credential-run.md) | GitHub asks you to review an `infra-credentials` deployment |
| [Review a PR that needs a person](review-a-pr.md) | A PR is labelled `needs-human`, or waits for your approval |
| [Set or raise a budget](set-a-budget.md) | An agent asks for a budget number |
| [Approve a new vendor](approve-a-vendor.md) | A decision record or PR proposes a vendor outside the approved list |
| [Rotate or revoke a secret](rotate-a-secret.md) | A token is about to expire, a check says one is wrong, or one may have leaked |

### Done once (keep for a rebuild)

| Runbook | What it covers |
| --- | --- |
| [Set up the GitHub repository](github-repo-setup.md) | The repo, the agent account and its role, merge settings, the Default ruleset, the $0 Actions budget, Dependabot, Jim's passkey |
| [Set up Claude](claude-setup.md) | The Claude account and its budget, the Monorepo project, the Claude GitHub App, connectors and scheduled tasks |
| [Set up Cloudflare](cloudflare-account.md) | The account, the deploy token and the `CLOUDFLARE_*` secrets |
| [Set up Auth0](auth0-tenant.md) | The tenant and the Cahokia and Cookbook applications |
| [Get the RapidAPI Zillow key](rapidapi-zillow.md) | The free RapidAPI plan homer uses |

Squarespace is an approved vendor but Jim's own account: nothing in the
repo uses it, so it has no runbook until something does.
