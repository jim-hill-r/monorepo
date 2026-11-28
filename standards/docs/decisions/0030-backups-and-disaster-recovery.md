# 0030. Backups and disaster recovery: recovery targets for everything that holds state, a nightly copy of the repo outside GitHub, encrypted exports of Cloudflare data, and drills that prove a restore works

- Status: Proposed
- Decided by: Jim (pending; the recommended options are what this record proposes)
- Date: 2026-11-28
- Asked by: Jim
- Builds on: [0007](0007-infrastructure-as-code.md) (IaC rebuilds what a
  provider covers), [0009](0009-opentofu-state-http-backend.md) (state in
  D1), [0011](0011-mint-secrets-by-workflow.md) (secrets are minted, not
  kept), [0021](0021-tenant-scoped-product-data.md) (product data in one
  D1, exportable per tenant), [0023](0023-observability.md) (alerts)

## Context

Jim (2026-11-28):

> I haven't considered backups and disaster recovery yet. We need to make
> a plan for this and create a project for it.

Nothing in the repo is backed up on purpose today. Some things are safe by
accident (git history is on every clone; the tofu_state Worker keeps 10
versions of each stack), but no copy of anything lives outside the vendor
that holds it, no recovery target is written down, and no restore has ever
been tried.

### What holds state

The full inventory, with each item's owner, how it is rebuilt and its tier,
is [backups/docs/inventory.md](../../../backups/docs/inventory.md). In short:

| What | Where | Can we rebuild it from the repo? |
| --- | --- | --- |
| The repo: all code, plans, decisions, issues, and the **cookbook content** (419 recipe files, the product and a future book) | GitHub, one repository, one owner account | It *is* the repo. GitHub is the single point of failure |
| Data branches: `evals-results`, `coverage-results`, `feedback-data` | GitHub | The first two regenerate from nightly runs; `feedback-data` (weekly fan feedback summaries) does not |
| Repo settings, rulesets, environments | GitHub | Mostly: the `github` IaC stack; the Default ruleset and a few settings are by hand (`standards/settings.toml`) |
| Secrets (GitHub secrets, Worker secrets) | GitHub, Cloudflare | No, by design: never in git or state. Infra Mint re-mints them from the bootstrap credentials |
| OpenTofu state | D1 `jimhillr-tofu-state` | Yes, by `tofu import`, slowly. The Worker keeps 10 versions; D1 Time Travel keeps 7 days |
| Product runtime data (flags, feedback, support, decisions 0018/0020/0021) | D1, planned | No: users wrote it |
| Pages and Workers config | Cloudflare | Yes: `wrangler.toml`, deploy workflows, the `cloudflare` stack |
| Analytics Engine events, Workers Logs | Cloudflare | No, and not needed: short-lived signals |
| Auth0 tenant config | Auth0 | Yes: the `auth0` stack |
| Auth0 users (cahokia and cookbook sign-ins) | Auth0 | No. Few users; they can sign up again |
| PostHog session replays | PostHog | No, and not needed |
| Domains | Squarespace | Registration, renewal and transfer lock are by hand |
| Accounts themselves (GitHub, Cloudflare, Auth0, PostHog, Squarespace, Claude) and their 2FA | Each vendor | No: losing one is the worst disaster on this list |
| Claude project setup and scheduled task prompts | Claude, prompts in `standards/docs/scheduled-tasks/` | Yes, from the prompts and `claude-setup.md` |

### What can go wrong

1. **The repo is lost or damaged**: the GitHub account is locked,
   suspended or compromised; the repository is deleted, or `main` is
   force-pushed or rewritten. Branch protection makes the last unlikely,
   not impossible.
2. **Cloudflare data is lost**: a bad migration, a bug or a person deletes
   D1 rows or a database.
3. **A vendor account is lost**: lost 2FA device, forgotten password,
   compromise or a vendor closing the account.
4. **A credential leaks or is lost**: handled by
   [rotate-a-secret.md](../runbooks/rotate-a-secret.md) and Infra Mint, not
   by backups.
5. **A region or vendor outage**: wait it out; we have no SLA to keep and
   the free tiers give us no second region.

### Constraints

- $0 and free tiers only, approved vendors first (AGENTS.md). Cloudflare
  R2 is on hold (it bills once enabled), and Cloudflare Artifacts needs
  Workers Paid, so neither is an option.
- No secret in git or OpenTofu state. The repo is **public**, so anything
  a workflow uploads as an artifact can be downloaded by any signed-in
  GitHub user: an export of user data must be encrypted before it leaves
  the runner.
- No local clone required: backups and drills run as workflows; only what
  must outlive our accounts lives with Jim.

## Decision

### 1. Recovery targets by tier

RPO is how much recent work we may lose; RTO is how long until it is back.

| Tier | What | RPO | RTO | How |
| --- | --- | --- | --- | --- |
| 0 | The repo, with the cookbook content first | 24 hours | 1 day to read it again, 3 days to work in it again | Nightly git bundle in Workers KV, quarterly offline copy with Jim |
| 1 | User data in D1 (tofu state now, product data when it lands) | 24 hours (minutes within 7 days via Time Travel) | 4 hours | D1 Time Travel, plus a nightly encrypted export kept 90 days as a GitHub Actions artifact |
| 2 | What we deploy (Pages, Workers, Auth0 config) | 0: built from the repo | 1 day | Redeploy from the repo: nightly CD and `infra-apply` |
| 3 | Secrets and credentials | none kept | 2 days, gated on Jim's approval | Bootstrap credentials by hand, then Infra Mint |
| 4 | Signals: Analytics Engine, logs, PostHog replays, eval and coverage results | accepted loss | none | Regenerate or do without |
| - | The accounts themselves | - | as fast as the vendor's recovery | Recovery codes kept offline by Jim |

The tier-0 RTO is longer than tier 1 because rebuilding GitHub (a new
account or repository, the Claude GitHub App, secrets re-minted) needs
Jim at several steps.

### 2. Copy the repo outside GitHub every night (recommended: Workers KV)

A scheduled `backups-nightly.yml` job runs `git bundle create --all` on a
full clone (every branch, including the data branches) and writes it to a
Workers KV namespace `jimhillr-backups` in chunks under 25 MiB. The bundle
is not encrypted: the repo is public. Two kinds of key:

- `repo/<date>/<n>`, with a 35-day expiry, so old copies delete
  themselves and the namespace never grows;
- `repo/latest/<n>`, with no expiry, overwritten each night, so a copy
  survives even if GitHub (and with it the job) is gone for longer than 35
  days.

The repo is about 7 MB packed, so 35 nightly copies plus the latest are
well under 300 MB of KV's 1 GB, and about 2 writes a night of its 1,000 a
day. The KV namespace is made by the `cloudflare` IaC stack; the write
token (Workers KV Storage edit on that account only) is made by Infra Mint
and lives only in the `backups` environment. GitHub and Cloudflare are
separate companies with separate sign-ins, so losing one keeps the other.

Options weighed (each is $0):

| Option | Vendor | Free-tier headroom | Build effort | Notes |
| --- | --- | --- | --- | --- |
| **Workers KV bundles** (recommended) | Cloudflare (approved) | ~30% of KV storage, <1% of writes | 1 thread (~2 hours of Claude Code) | Not browsable; restore is `git clone` from the bundle |
| GitLab.com push mirror | GitLab (new vendor, needs sign-off) | Free plan, 10 GiB a project | 1 thread, plus Jim's sign-up | Browsable and cloneable, a ready second home; another account to secure and recheck |
| Software Heritage "save code now" | Software Heritage (non-profit archive, no account) | Free | Small | Permanent public archive; we can't delete from it, and restore is slow |
| GitHub only (forks, release assets) | GitHub | - | Small | Doesn't survive losing the GitHub account, so it fails the main case |

### 3. Export Cloudflare data every night, encrypted

The same job runs `wrangler d1 export` for every D1 database bound in a
`wrangler.toml` and encrypts each file with
[age](https://age-encryption.org) to a public key committed in
`backups/keys/backups.age.pub`. The encrypted file is uploaded as a
GitHub Actions artifact kept 90 days (the maximum, and free for a public
repo). Only Jim's private key, kept offline and as the `backups`
environment secret `BACKUP_AGE_KEY` for the drill, decrypts it. tofu
state is already ciphertext; product data in decision 0021's shared D1
will not be, which is why every export is encrypted from the start.

D1 Time Travel (7 days on Free) stays the first resort for a recent
mistake; the artifact covers older ones and the loss of the database.

### 4. Keep what must outlive our accounts with Jim

Two human tasks, each with a runbook:

- **Account recovery codes**: save the 2FA recovery codes (or a second
  passkey) for GitHub, Cloudflare, Auth0, PostHog, Squarespace and Claude
  in Jim's password manager and on paper, and re-check yearly.
- **The backup key**: make the age key pair, keep the private key
  offline, store it as `BACKUP_AGE_KEY`, and commit the public key.

And one settings check every 90 days: **an offline copy**, where Jim
downloads the latest bundle (or, until phase 1, the repo's ZIP) to his own
computer, so the cookbook survives losing both GitHub and Cloudflare.

### 5. Prove restores with drills

- **Monthly restore drill** (`backups-drill.yml`): fetch `repo/latest`
  from KV, `git bundle verify`, clone it, check that `main` is less than
  24 hours behind GitHub and that `cookbook/content` has as many recipes
  as GitHub's `main`; download the newest D1 artifact, decrypt it, load it
  into a local SQLite database and run `PRAGMA integrity_check` and a row
  count. A failure opens an observability incident (decision 0023) and an
  issue file. These checks become the evals for the project's
  requirements.
- **Yearly tabletop drill**: Jim and an agent walk through
  [disaster-recovery.md](../runbooks/disaster-recovery.md) for "the GitHub
  account is gone" and fix whatever is wrong on the day.

### 6. Auth0 users, PostHog, analytics: accept the loss

The Auth0 tenant config is in IaC. Its few users can sign up again, and an
export of their emails would be personal data to protect for little gain,
so we don't back them up. Revisit when a product has users who pay or who
would lose their own data with their account.

## Consequences

- A new platform project, `backups`, owns the jobs, drills, inventory and
  user guide. Its work is the epic "Backups" in `backups/issues/`.
- A new credential (the KV write token) and a new GitHub environment
  (`backups`) are credential changes: they apply only after Jim approves
  the `infra-credentials` run (decision 0010).
- Every new stateful resource (a D1 database, a KV namespace, a data
  branch) is added to `backups/docs/inventory.md` with its tier in the
  same PR; phase 3 adds an audit check that fails a D1 database or KV namespace
  named in a `wrangler.toml` or the `cloudflare` stack with no inventory row.
- Restores are documented once, in
  [disaster-recovery.md](../runbooks/disaster-recovery.md), and tested by
  drills instead of trusted.
- Cost: $0 a month. KV headroom and Actions minutes (about 3 minutes a
  night, free on a public repo) are recorded in `backups/docs/cost.toml`
  when phase 1 lands.

## Revisit

- If the repo goes private: Actions artifacts and minutes count against
  the free private quota, and the KV bundle must be encrypted too.
- If the repo passes 300 MB packed, or KV writes pass 50 a day.
- If R2 becomes usable without a payment method.
- When a product's users can lose their own data (Auth0 users, uploads).
