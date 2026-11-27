---
added: 2026-11-27
unblocks: "the first deploy of the OpenTofu state Worker (tofu_state, decision 0009)"
---

In the Cloudflare dashboard (My Profile > API Tokens), give the `CLOUDFLARE_API_TOKEN` secret's token "Workers Scripts: Edit" and "D1: Edit" for this account, so `mise run deploy` in `tofu_state` can deploy the `jimhillr-tofu-state` Worker and create its D1 database (it already has Pages edit; Workers Scripts: Edit also lets the weekly free-tier usage run count Workers). Cost: $0; the deploy stays within the Workers and D1 free tiers (`free-tiers.sh worker` and `d1-database` check first). About 2 minutes.
