# bear_down/cloudflare

Deploys the bear_down site to Cloudflare Pages at
<https://jimhillr-bear-down.pages.dev>.

- `mise run release` runs the season simulation (`bear_down_cli`, which
  downloads nflverse `games.csv`) into `../web/public/rooting.json`, then
  bundles the web app into `../web/dist/public`.
- `mise run deploy` runs `release`, creates the Pages project within the free
  tier budget (`standards/scripts/free-tiers.sh pages-project`) and uploads
  the site with `wrangler pages deploy`. It needs `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID`.

CI deploys on every push to `main` that touches bear_down, and Nightly CD
rebuilds it every night so the odds stay fresh. See
[decision 0009](../docs/decisions/0009-cloudflare-pages-nightly-cd.md).
