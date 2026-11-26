# 0009. OpenTofu state in a Cloudflare Worker with D1

- Status: Accepted
- Decided by: Jim (decision card in the IaC thread, 2026-11-26, after the
  comparison below with its costs)
- Date: 2026-11-26
- Asked by: Jim
- Changes: [0007](0007-infrastructure-as-code.md) item 7 (state on an
  `infra-state` branch)

## Context

Jim (2026-11-26): "Never commit secrets to git or store them in tf state",
then "How are we storing tf state btw? Don't we need a remote backend?" and
"Lets do another deep dive into whether we need r2 for the tf state backend.
Can we use git lfs? Does git lfs provide better security guarantees than raw
git?"

Decision 0007 kept OpenTofu state, encrypted, on a branch of this public
repo. With the 0007 amendment (IAC-002 keeps every secret out of state)
that state holds no secrets, but it still puts resource IDs and settings
into git, where anyone can fetch every version. Nothing has been applied
yet, so no state exists to move.

## Findings

**Git LFS gives no better security than plain git.** LFS swaps large files
for pointer files and keeps the content on GitHub's LFS server, but the
access rule is the repository's: anyone who can read this public repo can
download every LFS object, every version. It adds no encryption or access
control of its own. It is harder to clean up than plain git: GitHub can only
remove LFS objects by deleting the whole repository (or through support),
while a plain-git file can at least be rewritten out of history. OpenTofu
has no LFS backend, so a workflow would commit the file, with no locking.

**R2 needs a payment method.** Cloudflare asks for a card before R2 can be
turned on, even for its free 10 GB, and R2 has no hard spending cap. That
breaks the free-tiers rule ("never add a payment method").

**OpenTofu's `http` backend** stores state with `GET` and `POST` on one URL
per stack and locks with `LOCK` and `UNLOCK` requests, with basic-auth
credentials from `TF_HTTP_USERNAME` and `TF_HTTP_PASSWORD`. A small Worker
can serve it, with D1 as the store.

## Options

Every option keeps state encrypted (0007, item 7) and secret-free
(IAC-002), so what differs is who can fetch the encrypted file, locking,
deletion, money and effort.

| | Who can read the file | Locking | History and deletion | Money and limits | Payment method | Vendor | Claude Code effort |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Branch of this public repo (0007) | Anyone | None (Actions `concurrency` only) | Every version kept; removal needs a history rewrite | $0 | No | GitHub | None |
| Git LFS in this repo | Anyone, every version | None | Only by deleting the repo | $0 inside 10 GiB storage and bandwidth | No | GitHub | About 1 PR |
| A separate private repo | Repo collaborators only | None (Actions `concurrency` only) | Every version kept; removal needs a history rewrite | $0 | No | GitHub | About 1 PR; Jim makes the repo and a write token |
| **Worker + D1 HTTP backend** (chosen) | Only holders of a backend password | Real: `LOCK` and `UNLOCK` | We choose (keep the last 10 versions); delete with one query | $0: 1 of our 25 Workers, a few rows against D1's 5 GB and 100,000 writes a day | No | Cloudflare | About 2 PRs, plus upkeep of a small Worker; Jim makes two passwords |
| R2 (S3 backend) | Only holders of an R2 key | Real: `use_lockfile` | Optional versioning; delete any time | $0 expected inside 10 GB, but no hard cap | **Yes** | Cloudflare | About 1 PR |
| HCP Terraform free | Our HCP organization | Real | Kept by HCP | $0 up to 500 resources | No | HashiCorp (Hold) | About 1 PR |
| Postgres `pg` backend on a free host | Database users | Real | Our choice | $0 on a free tier | Depends on host | New vendor | About 1 PR |

Ruled out: GitHub Actions artifacts and caches (they expire, at most 90 days
for artifacts and 7 days unused for caches), and AWS S3 or Google Cloud
Storage (new vendors that need a card).

Jim first chose R2 on a card that named the payment method, then asked for
this comparison and chose the Worker.

## Decision

1. **A Worker, `jimhillr-tofu-state`, serves OpenTofu's `http` backend**,
   with state in a D1 database, `jimhillr-tofu-state`. Each stack has its own
   URL, `/states/<stack>`, so stacks lock and fail on their own. It is
   deployed with wrangler like the other Workers, never by `infra` itself
   (IaC can't manage the store its own state lives in), and it counts
   against the Workers budget in `free-tiers.sh` like any other Worker.
2. **Locking is real.** `LOCK` takes a row lock in D1 with the lock holder's
   ID and fails with `409` and the current holder when it's held; `UNLOCK`
   releases it only for the same ID. A write from anyone but the lock holder
   fails. The one `concurrency` group for infra jobs (0007, item 5) stays.
3. **History**: the Worker keeps the last 10 versions of each stack's state,
   so a bad apply can be rolled back with a query, and deletes older ones.
4. **Two passwords, read and write.** The read password (repository secret
   `TOFU_STATE_READ_PASSWORD`, for PR plans, which run with `-lock=false`)
   can only `GET`. The write password (`TOFU_STATE_WRITE_PASSWORD`, only in
   the `infra` environment) can also `POST`, `LOCK` and `UNLOCK`. Jim makes
   both as long random strings; the Worker's deploy copies them into Worker
   secrets from the GitHub secrets, so they are never typed twice or written
   down. They are compared in constant time.
5. **State is still encrypted** with OpenTofu state encryption (0007, item 7).
   The passphrase (`TOFU_STATE_PASSPHRASE`, as `TF_VAR_tofu_state_passphrase`)
   is only in GitHub, never in the Worker, so the Worker and D1 only ever see
   ciphertext.
6. **Nothing environment-specific in the code.** The synthesized backend
   block holds only the HTTP methods; the workflows pass the stack's URL
   (`address`, `lock_address`, `unlock_address`) with `-backend-config`, and
   the password in `TF_HTTP_PASSWORD`.
7. **The free-tiers rule stands unchanged**: no payment method, R2 stays on
   Hold, and IAC-003 keeps rejecting R2 resources.

## Cost

$0. One Worker of our 25, a few hundred requests a month against 100,000 a
day, and one D1 database with a few rows (well under a megabyte, keeping 10
versions of three stacks) against 5 GB and 100,000 writes a day. No payment
method and no new vendor. Claude Code effort: this PR (backend config and
this record) and one more for the Worker, with tests, plus its upkeep.

## Consequences

- Jim's one-time setup adds two random passwords as GitHub secrets
  (`issues/human/tofu-state-passwords.md`).
- State never touches git, so the cleanup sweep has no `infra-state` branch
  to keep, and the Worker and its D1 database are claimed by `infra`.
- The Worker becomes something we run. If it is ever dropped, the state moves
  to any other backend with `tofu init -migrate-state`.

## Sources

Read 2026-11-26.

- [OpenTofu HTTP backend](https://opentofu.org/docs/language/settings/backends/http/).
- [D1 pricing](https://developers.cloudflare.com/d1/platform/pricing/) and
  [Workers limits](https://developers.cloudflare.com/workers/platform/limits/).
- [R2 pricing](https://developers.cloudflare.com/r2/pricing/).
- [GitHub: removing Git LFS objects](https://docs.github.com/en/repositories/working-with-files/managing-large-files/removing-files-from-git-large-file-storage).
