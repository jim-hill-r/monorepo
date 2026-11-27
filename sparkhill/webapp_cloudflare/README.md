# sparkhill/webapp_cloudflare

Deploys the Blue Eel web app (`../webapp`) to Cloudflare Pages at
<https://jimhillr-sparkhill-webapp.pages.dev> (Pages project
`jimhillr-sparkhill-webapp`). It holds no application code, only the wrangler
and mise config. The app is static files (its letter patterns are bundled),
so it needs no Functions, Worker or outside host
([decision 0001](../docs/decisions/0001-static-site-on-cloudflare-pages.md)).

- `mise run release` runs the web app's own `release` task, `dx bundle`,
  which writes the site to `../webapp/dist/public`.
- `mise run deploy` runs `release`, creates the Pages project within the free
  tier budget (`standards/scripts/free-tiers.sh pages-project`) and uploads
  the site with `wrangler pages deploy`. It needs `CLOUDFLARE_API_TOKEN` and
  `CLOUDFLARE_ACCOUNT_ID`, the repo's existing Cloudflare secrets.
- `mise run ci` checks this config (`tests/config.rs`): the Pages name, the
  output directory and free-plan settings only.

The Pages name is `jimhillr-sparkhill-webapp` rather than
`jimhillr-sparkhill` because that one already serves the sparkhill website
(`../cloudflare`). The app's routes (`/letter`, `/word`, ...) load directly
because the bundle has no `404.html`, so Pages serves `index.html` for them.

CI deploys on every push to `main` that touches this project or the web app
(`# affected-by: ../webapp` in `mise.toml`), and Nightly CD rebuilds it every
night, like the other `cloudflare` projects.
