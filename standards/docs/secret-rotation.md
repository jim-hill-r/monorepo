# Secret rotation plan

Jim (2026-11-27): "For any long lived tokens we need to create a rotation
plan." This page covers every long-lived secret the repo uses: how often it
is rotated, who or what rotates it, and what reminds them. Values are never
written here, only names and places (AGENTS.md, "No secrets in git or
state").

## Rules

- **Automation first.** A secret that the Infra Mint workflow makes is
  rotated by Infra Mint. Each quarter a scheduled run remakes the
  rotatable set (`rotate` = `auto`, see `ROTATE_AUTO` in
  `infra/scripts/mint.sh`). Like every run in `infra-credentials`, it waits
  for Jim's approval, and that approval request is the reminder.
- **Fail closed.** Minted Cloudflare tokens expire 180 days after each mint.
  If a quarterly rotation is missed, the next one still renews them in
  time. If two in a row are missed, they expire and deploys stop rather
  than run on a stale token.
- **A person rotates only what automation can't make.** These are the
  bootstrap credentials and the secrets outside GitHub. Each one has a
  rotation check in `standards/settings.toml`, so the weekly human issues
  digest raises it as a task when it's due. The steps are in
  [rotate-a-secret.md](runbooks/rotate-a-secret.md).
- **On a leak, rotate now.** Whatever the cadence, revoke at the vendor
  first, then rotate everything that secret could make
  ([rotate-a-secret.md](runbooks/rotate-a-secret.md)).
- **A new long-lived secret adds its row here** in the PR that introduces
  it, with a cadence and who rotates it.

## Every long-lived secret

| Secret | Where | Rotated by | Every | Reminder |
| --- | --- | --- | --- | --- |
| `TOFU_STATE_READ_PASSWORD` | repository | Infra Mint (auto), which redeploys the state Worker | 90 days | scheduled Infra Mint run |
| `TOFU_STATE_WRITE_PASSWORD` | `infra` | Infra Mint (auto), which redeploys the state Worker | 90 days | scheduled Infra Mint run |
| `CLOUDFLARE_API_TOKEN` (`jimhillr-deploy`) | repository | Infra Mint (auto); expires after 180 days | 90 days | scheduled Infra Mint run |
| `CLOUDFLARE_INFRA_READ_TOKEN` | repository | Infra Mint (auto); expires after 180 days | 90 days | scheduled Infra Mint run |
| `CLOUDFLARE_INFRA_APPLY_TOKEN` | `infra` | Infra Mint (auto); expires after 180 days | 90 days | scheduled Infra Mint run |
| `CLOUDFLARE_BILLING_READ_TOKEN` (`jimhillr-billing-read`) | `infra` | Infra Mint (auto); expires after 180 days | 90 days | scheduled Infra Mint run |
| `GITHUB_BILLING_READ_TOKEN` (fine-grained, Plan read) | `infra` | Jim, [billing-read-credentials.md](runbooks/billing-read-credentials.md); it expires after a year | 365 days | settings check `rotate-github-billing-read-token` |
| `REVIEWER_TOKEN` (fine-grained, `jimhillr-reviewer`: Pull requests write, Contents, Checks and Commit statuses read) | `reviewer` | Jim, signed in as `jimhillr-reviewer`, [jimhillr-reviewer-account.md](runbooks/jimhillr-reviewer-account.md); it expires after a year | 365 days | settings check `rotate-reviewer-token` |
| `AUTH0_INFRA_READ_CLIENT_SECRET` | repository | Infra Mint (auto, Auth0 rotate-secret) | 90 days | scheduled Infra Mint run |
| `AUTH0_INFRA_APPLY_CLIENT_SECRET` | `infra` | Infra Mint (auto, Auth0 rotate-secret) | 90 days | scheduled Infra Mint run |
| `GH_INFRA_CREDENTIALS_APP_KEY` (GitHub App private key) | `infra-credentials` | Jim, [github-app-infra.md](runbooks/github-app-infra.md) | 180 days | settings check `rotate-github-app-infra-key` |
| `GH_INFRA_READ_APP_KEY` (read-only GitHub App private key) | repository | Jim, [github-app-infra-read.md](runbooks/github-app-infra-read.md) | 365 days | settings check `rotate-github-app-infra-read-key` |
| `AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET` (bootstrap) | `infra-credentials` | Jim, [infra-credentials-environment.md](runbooks/infra-credentials-environment.md) | 180 days | settings check `rotate-auth0-infra-credentials` |
| `CLOUDFLARE_INFRA_CREDENTIALS_TOKEN` (bootstrap) | `infra-credentials` | Jim, the same runbook; set a 200-day expiry on it when you make it | 180 days | settings check `rotate-cloudflare-infra-credentials` |
| `TOFU_STATE_PASSPHRASE` | repository | not rotated yet: changing it needs state re-encrypted first (`infra/docs/issues/iac-rotate-state-passphrase.md`) | on a leak only, until that issue lands | none |
| `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD` | repository | not rotated: a new key can't update an installed APK, so testers would have to reinstall | on a leak only | none |
| `RAPIDAPI_KEY` (homer) | Jim's shell, not GitHub | Jim, [rapidapi-zillow.md](runbooks/rapidapi-zillow.md) | 365 days | settings check `rotate-rapidapi-key` |
| `POSTHOG_PERSONAL_API_KEY` | repository, once made | Jim, [posthog-sign-up.md](runbooks/posthog-sign-up.md) | 180 days | settings check, added in the PR that first uses it |
| `POSTHOG_BILLING_READ_KEY` | `infra`, once made | Jim, [posthog-sign-up.md](runbooks/posthog-sign-up.md) | 180 days | settings check `rotate-posthog-billing-read-key` |

These aren't secrets, or aren't ours to rotate:

- `CLOUDFLARE_ACCOUNT_ID`, `TOFU_STATE_ORIGIN`, `GH_INFRA_CREDENTIALS_APP_ID` and `GH_INFRA_READ_APP_ID` are identifiers, not secrets.
- `GITHUB_TOKEN` and the GitHub App installation tokens last an hour at most.
- The Auth0 domain and client IDs are public.
- The Claude GitHub App and the claude.ai connectors are signed in and renewed by their own platforms. Their settings checks already cover them.

## The quarterly run

On the 1st of January, April, July and October at 14:17 UTC (9:17 AM New
York), **Infra Mint** starts with `rotate` = `auto` and waits for Jim's
approval in `infra-credentials`. Once he approves, it:

1. Generates new state passwords and queues **Deploy tofu_state**, which
   copies them into the Worker. That deploy queues behind the mint in the
   `infra` concurrency group, so no plan or apply runs between the two.
2. Rolls each minted Cloudflare token, renews its 180-day expiry, and
   writes the new value straight into its GitHub secret.
3. Rotates the Auth0 `infra-read` and `infra-apply` client secrets and
   writes them the same way.

A workflow that started before the roll and uses the old token, such as a
Pages deploy, fails once; re-running it fixes it. GitHub lets an approval
wait for up to 30 days. If the run times out, Jim can start Infra Mint by
hand with `rotate` = `auto`.

## Cost

$0 in money:
- Cloudflare tokens, GitHub secrets and GitHub Apps are free.
- Each quarterly run uses one Auth0 Machine to Machine token out of the
  free 1,000 a month, so 4 a year.
- The run and the Worker redeploy add a few Actions minutes a quarter.
- Building this was a small change to Infra Mint, roughly $1 of the $25
  monthly Claude Code budget.

Jim's time is about 1 minute a quarter to approve, plus about 15 minutes
twice a year for the three bootstrap credentials, and once a year for the
RapidAPI key.
