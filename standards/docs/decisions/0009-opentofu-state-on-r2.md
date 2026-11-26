# 0009. OpenTofu state in Cloudflare R2, the one payment-method exception

- Status: Accepted
- Decided by: Jim (decision card in the IaC thread, 2026-11-26, after the
  options below with their costs)
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

## Decision

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
