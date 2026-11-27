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
- `mise run e2e` makes the release export and runs the Playwright smoke
  tests in `e2e/` on it at desktop and phone widths. `e2e/serve.mjs` serves
  `dist/exported` the way Cloudflare Pages does, so the tests load the same
  files the deploy uploads. `ci` and `test` run it too.

Files in `static/` are served at `/.perseus/static/`, so stylesheets and
templates link them there (a root path such as `/fonts/x.ttf` 404s). The
Niramit font in `static/fonts/` is under the SIL Open Font License
(`Niramit-OFL.txt`).
