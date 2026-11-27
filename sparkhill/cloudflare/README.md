# sparkhill/cloudflare

Deploys the sparkhill website (`../website`) to Cloudflare Pages at
<https://jimhillr-sparkhill.pages.dev> (Pages project `jimhillr-sparkhill`).
It holds no application code, only the wrangler and mise config. Why the
site is static: [decision 0001](../docs/decisions/0001-static-site-on-cloudflare-pages.md).

- `mise run release` runs the website's own `release` task,
  `perseus export --release`, which writes the static site to
  `../website/dist/exported`.
- `mise run deploy` runs `release`, creates the Pages project within the free
  tier budget (`standards/scripts/free-tiers.sh pages-project`) and uploads
  the site with `wrangler pages deploy`. It needs `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID`, the repo's existing Cloudflare secrets.
- `mise run ci` checks this config (`tests/config.rs`): the Pages name, the
  output directory, free-plan settings only, and that the website stays
  export-only.

CI deploys on every push to `main` that touches this project or the
website (`# affected-by: ../website` in `mise.toml`), and Nightly CD
rebuilds it every night, like the other `cloudflare` projects.

The Blue Eel web app (`../webapp`) deploys separately, from
`../webapp_cloudflare`, to <https://jimhillr-sparkhill-webapp.pages.dev>.
