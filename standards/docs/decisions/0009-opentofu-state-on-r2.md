# 0009. OpenTofu state in Cloudflare R2, the one payment-method exception

- Status: Proposed (Jim chose R2 on a decision card on 2026-11-26, then
  asked for the deeper comparison below before accepting)
- Decided by: Jim, pending
- Date: 2026-11-26
- Asked by: Jim
- Changes: [0007](0007-infrastructure-as-code.md) item 7 (state on an
  `infra-state` branch), and the free-tiers rule "never add a payment
  method", for R2 only

## Context

Jim (2026-11-26): "Never commit secrets to git or store them in tf state",
and "How are we storing tf state btw? Don't we need a remote backend?"

Decision 0007 kept OpenTofu state, encrypted, on a branch of this public
repo. With the 0007 amendment (IAC-002 keeps every secret out of state)
that state holds no secrets, but it still puts resource IDs and settings
into git. Nothing has been applied yet, so no state exists to move.

## Options

Put to Jim on a decision card, with these costs:

| Option | Money | Free-tier impact | Vendor | Claude Code effort |
| --- | --- | --- | --- | --- |
| A Worker serving OpenTofu's HTTP backend, state in D1 | $0 | 1 of our 25 Workers; far inside D1's free limits | None new | About 2 PRs, plus a Worker to build, secure and keep |
| Encrypted `infra-state` git branch (0007 as written) | $0 | None | None | No extra PRs |
| **Cloudflare R2 with the S3 backend** (chosen) | $0 expected | A few KB of 10 GB, and a few hundred of 1M writes a month | Cloudflare (approved) | About 1 PR |
| HCP Terraform free tier | $0 up to 500 resources | 500-resource cap | HashiCorp (on Hold) | About 1 PR |

The card said R2 needs a payment method on the account even for its free
tier, which the free-tiers rule forbids. Jim chose R2 knowing that.

## Deeper comparison (Jim, 2026-11-26)

Jim: "Lets do another deep dive into whether we need r2 for the tf state
backend. Can we use git lfs? Does git lfs provide better security guarantees
than raw git?"

**Git LFS gives no better security than plain git.** LFS swaps large files
for pointer files and keeps the content on GitHub's LFS server, but the
access rule is the repository's: anyone who can read this public repo can
download every LFS object, every version. It adds no encryption or access
control of its own. It is harder to clean up than plain git: GitHub can only
remove LFS objects by deleting the whole repository (or through support),
while a plain-git file can at least be rewritten out of history. OpenTofu
has no LFS backend either, so a workflow would commit the file, with no
locking. So LFS is no better than the 0007 branch, and slightly worse.

Every option below keeps state encrypted (0007, item 7) and secret-free
(IAC-002), so what differs is who can fetch the encrypted file, whether it
can be deleted, and the money and effort.

| | Who can read the file | Locking | History and deletion | Money and limits | Payment method | Vendor | Claude Code effort |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Branch of this public repo (0007) | Anyone | None (Actions `concurrency` only) | Every version kept; removal needs a history rewrite | $0 | No | GitHub | None |
| Git LFS in this repo | Anyone, every version | None | Only by deleting the repo | $0 inside 10 GiB storage and bandwidth | No | GitHub | About 1 PR |
| A separate private repo (for example `jimhillr/infra-state`) | Repo collaborators only | None (Actions `concurrency` only) | Every version kept; removal needs a history rewrite | $0 (storing state costs no Actions minutes) | No | GitHub | About 1 PR; Jim makes the repo and a write token |
| **Worker + D1 HTTP backend** | Only holders of the backend token | Real: OpenTofu's HTTP `LOCK` and `UNLOCK` | We choose (keep the last N versions); delete with one query | $0: 1 of our 25 Workers, a few rows against D1's 5 GB and 100,000 writes a day | No | Cloudflare | About 2 PRs, plus upkeep of a small Worker; Jim makes one token |
| R2 (S3 backend) | Only holders of an R2 key | Real: `use_lockfile` | Optional object versioning; delete any time | $0 expected inside 10 GB, but no hard cap | **Yes** | Cloudflare | About 1 PR (this one) |
| HCP Terraform free | Our HCP organization | Real | Kept by HCP | $0 up to 500 resources | No | HashiCorp (Hold) | About 1 PR |
| Postgres `pg` backend on a free host | Database users | Real | Our choice | $0 on a free tier | Depends on host | New vendor | About 1 PR |

Ruled out: GitHub Actions artifacts and caches (they expire, at most 90 days
for artifacts and 7 days unused for caches), and AWS S3 or Google Cloud
Storage (new vendors that need a card).

**Recommendation: the Worker + D1 HTTP backend.** It is the only option that
keeps state out of git and off any public URL with real locking, at $0, with
no payment method and no new vendor. R2 gets the same guarantees with less
to build, but only by putting a card on the account with no hard spending
cap. A private repo is the cheapest, but it is still git, which Jim's rule
is moving away from, and it has no locking. If Jim picks the Worker, this
record is rewritten around it before merging, and the R2 parts are dropped.

## Decision (as first chosen: R2)

1. **State lives in one private R2 bucket, `jimhillr-infra-state`**, one
   object per stack (`<stack>.tfstate`), through OpenTofu's `s3` backend
   with R2's S3 endpoint. It is never committed to git, and the
   `infra-state` branch from 0007 is not created.
2. **Locking is OpenTofu's S3 lock file** (`use_lockfile = true`, a
   conditional write next to the state), with no DynamoDB. The one
   `concurrency` group for infra jobs (0007, item 5) stays as well.
3. **State is still encrypted** with OpenTofu state encryption (0007,
   item 7). The passphrase comes from the `TOFU_STATE_PASSPHRASE` secret
   through `TF_VAR_tofu_state_passphrase`, so it is never written in code
   or in state.
4. **Nothing secret in the code.** The bucket name and object keys are in
   the synthesized JSON. The R2 endpoint, which contains the account ID,
   comes from `AWS_ENDPOINT_URL_S3`, and the R2 access key from
   `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`, all GitHub secrets made by
   Jim. The read set (PR plans) can read only that bucket, and the write set
   is in the `infra` environment.
5. **The payment-method exception is for R2 only.** Jim adds a payment
   method to Cloudflare once, for R2. The rule stays for every other service
   and for Cloudflare's paid plans: no Workers Paid, no other paid add-on,
   and no second R2 bucket without a decision record.
6. **Guards against a bill.** R2 has no hard spending cap, so:
   - our budget is 100 MB of storage (state is a few KB), set in
     `standards/radar/limits.toml`. 100 MB is a proposal; only Jim sets a
     budget, so he confirms or changes it in this PR;
   - IAC-003 keeps rejecting every `cloudflare_r2_*` resource, so IaC can
     never create a bucket. The state bucket is made by hand, outside IaC;
   - Jim turns on a Cloudflare billing notification for any usage-based
     charge, so a cent of R2 billing shows up the same day;
   - the weekly free-tier usage check adds R2 storage and operations once
     its token can read them (follow-up issue).
7. **Tech radar:** Cloudflare R2 moves from Hold to Trial, with `infra` as
   its only exemplar and this record as its source, and only for state.

## Cost

$0 expected. The state of three stacks is a few KB against 10 GB a month
free, and each plan or apply makes a handful of operations, against 1M Class
A and 10M Class B operations a month free. Egress is free. The risk is a
bill if something else ever writes to R2. The guards above make that
visible within a day, and only a person can create a bucket. Claude Code
effort: this PR, plus a small change to the workflows issue.

## Consequences

- Jim's one-time setup grows by a payment method, a bucket, two R2 access
  keys and a billing notification (`issues/human/r2-infra-state-bucket.md`).
- State never touches git, so the cleanup sweep has no `infra-state` branch
  to keep.
- If R2 is ever dropped, the state objects move to any S3-compatible store or
  to the HTTP backend option, with `tofu init -migrate-state`.

## Sources

Read 2026-11-26.

- [R2 pricing](https://developers.cloudflare.com/r2/pricing/) (free tier,
  and that a payment method is needed to turn R2 on).
- [OpenTofu S3 backend](https://opentofu.org/docs/language/settings/backends/s3/)
  (`use_lockfile`, `skip_*` settings for S3-compatible stores).
