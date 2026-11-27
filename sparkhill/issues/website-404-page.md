---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

The website's export has no `404.html`, so Cloudflare Pages treats the site
as a single-page app and answers every unknown URL (such as `/nope`) with
the root page and status 200 instead of a 404
([Pages serving](https://developers.cloudflare.com/pages/configuration/serving-pages/#not-found-behavior)).
Search engines see those as soft 404s, and the visitor gets the locale
redirect rather than a "page not found". Have `release` write the 404 error
view to `dist/exported/404.html` (`perseus export-error-page --code 404`),
give it a link back to the home page, and add a Playwright test in `e2e/`
(with `e2e/serve.mjs` serving `404.html` the way Pages does) that an unknown
URL gets status 404 and the page-not-found text.
