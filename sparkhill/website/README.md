# sparkhill_website

The Sparkhill website, built with Perseus and Rust as a static export and
deployed to Cloudflare Pages by [`../cloudflare`](../cloudflare) at
<https://jimhillr-sparkhill.pages.dev>. It has no server: the Perseus engine
only runs at build time
([decision 0001](../docs/decisions/0001-static-site-on-cloudflare-pages.md)).

- `mise run dev` exports the site and serves the static files, rebuilding on
  changes (`perseus export -s -w`).
- `mise run release` writes the release export to `dist/exported`, which
  `../cloudflare` uploads. It also writes the 404 error view to
  `dist/exported/404.html`: without one, Cloudflare Pages answers every
  unknown URL with the home page and status 200.
- `mise run ci` checks formatting, lints, the build and the tests.
- `mise run e2e` makes the release export and runs the Playwright smoke
  tests in `e2e/` on it at desktop and phone widths. `e2e/serve.mjs` serves
  `dist/exported` the way Cloudflare Pages does, so the tests load the same
  files the deploy uploads. `ci` and `test` run it too.

Files in `static/` are served at `/.perseus/static/`, so stylesheets and
templates link them there (a root path such as `/fonts/x.ttf` 404s). The
Niramit font in `static/fonts/` is under the SIL Open Font License
(`Niramit-OFL.txt`).

Perseus 0.4.2 doesn't really hydrate the export: its client builds the page
before it enters its hydration context, so it renders new nodes and
replaces every exported one about 50 ms after load. Anything that holds an
exported node (the browser's own scroll to a URL's `#section`, a script)
loses it then. `scroll_to_url_section` in `src/templates/index.rs` redoes
that scroll, and the e2e test "a section link still lands on its section
when the app replaces the page" checks it. The flake it fixed shows only in
Chromium's headless shell, the browser CI runs, not in full Chromium.
