# sparkhill_website

The Sparkhill website, built with Perseus and Rust as a static export and
deployed to Cloudflare Pages by [`../cloudflare`](../cloudflare) at
<https://jimhillr-sparkhill.pages.dev>. It has no server: the Perseus engine
only runs at build time
([decision 0001](../docs/decisions/0001-static-site-on-cloudflare-pages.md)).

- `mise run dev` exports the site and serves the static files, rebuilding on
  changes (`perseus export -s -w`).
- `mise run release` writes the release export to `dist/exported`, which
  `../cloudflare` uploads.
- `mise run ci` checks formatting, lints, the build and the tests.
