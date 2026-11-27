---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The website's `head` in `website/src/templates/index.rs` has only a title,
an icon and stylesheets: no `<meta name="description">` and no Open Graph
tags (`og:title`, `og:description`, `og:image`), so search results and
shared links show no summary or picture. Add them (the image from
`static/images/`, as an absolute URL on `https://jimhillr-sparkhill.pages.dev`)
and a test that the exported page has them.
