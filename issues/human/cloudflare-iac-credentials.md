---
added: 2026-11-26
unblocks: "the infra cloudflare stack (decision 0007), which then turns on Web Analytics for every production Pages project instead of the dashboard"
---

In the Cloudflare dashboard (My Profile > API Tokens > Create Token, Custom token), create two tokens limited to this account: "infra-read" with Account Web Analytics Read, and "infra-apply" with Account Web Analytics Edit. Neither needs API Tokens permissions, so IaC can't mint or change tokens. Add infra-read as the repository secret `CLOUDFLARE_INFRA_READ_TOKEN` and infra-apply as `CLOUDFLARE_INFRA_APPLY_TOKEN` in the `infra` environment (limited to `main`, the one `auth0-iac-credentials.md` creates). Cost: $0; Web Analytics is free. When phase 3 adds Pages and Access, the same two tokens get those permissions added.
