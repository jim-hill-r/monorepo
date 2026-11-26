---
added: 2026-11-26
unblocks: "infrastructure as code phase 1 (decision 0007), which then manages the Auth0 callback URLs instead of the dashboard"
---

In the Auth0 dashboard, create two Machine to Machine applications for the Auth0 Management API: "infra-read" with `read:clients`, and "infra-apply" with `read:clients` and `update:clients`. Add their domain, client ID and client secret as GitHub secrets: the read one as repository secrets `AUTH0_INFRA_READ_*`, the apply one in a new `infra` environment limited to `main` as `AUTH0_INFRA_APPLY_*`. Also add `TOFU_STATE_PASSPHRASE` (a long random string) to the `infra` environment. The read app is also the read-only token the resource cleanup plan's phase 4 needs.
