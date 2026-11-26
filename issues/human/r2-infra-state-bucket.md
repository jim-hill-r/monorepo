---
added: 2026-11-26
unblocks: "the infra plan and apply workflows (decision 0007), which keep OpenTofu state in R2 (decision 0009)"
---

One-time R2 setup for OpenTofu state (decision 0009). In the Cloudflare dashboard:

1. Billing: add a payment method. This is the one exception to "no payment method" (decision 0009), for R2 only; don't subscribe to anything else.
2. Billing > Notifications: turn on an alert for any usage-based charge, so an R2 bill shows the same day.
3. R2: turn it on (the free plan), and create one private bucket, `jimhillr-infra-state`, with no public access and no custom domain.
4. R2 > Manage API tokens: create two tokens scoped to that bucket only: "infra-state-read" (Object Read only) and "infra-state-write" (Object Read & Write). Each shows an access key ID and a secret access key once.
5. In GitHub, add the read pair as repository secrets `R2_INFRA_READ_ACCESS_KEY_ID` and `R2_INFRA_READ_SECRET_ACCESS_KEY`, the write pair as `R2_INFRA_WRITE_ACCESS_KEY_ID` and `R2_INFRA_WRITE_SECRET_ACCESS_KEY` in the `infra` environment, and the endpoint `https://<account id>.r2.cloudflarestorage.com` as the repository secret `R2_INFRA_ENDPOINT`.

Cost: $0 expected (a few KB of the free 10 GB a month). About 15 minutes. Also confirm or change the 100 MB R2 budget in `standards/radar/limits.toml` (only you set budgets). The `TOFU_STATE_PASSPHRASE` secret from `auth0-iac-credentials.md` still encrypts the state.
