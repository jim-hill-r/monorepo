---
added: 2026-11-25
unblocks: the telemetry issues in the root issues/
---

In the Cloudflare dashboard (My Profile > API Tokens), give the `CLOUDFLARE_API_TOKEN` secret's token "Account Analytics: Read", so it can query Analytics Engine. About 2 minutes, $0; it can be done in the same edit as `cloudflare-dashboard-give-cloudflare-api-token.md` (Workers Scripts: Read).

Web Analytics for each production Pages project is no longer a dashboard step: the `cloudflare` stack in `infra` turns it on (`cloudflare_web_analytics_site`, Jim chose IaC on 2026-11-26), once `cloudflare-iac-credentials.md` and the plan and apply workflows are done.
