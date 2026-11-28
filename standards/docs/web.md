# Websites: search, agents and performance

What every site we deploy must do so that search engines find it, AI agents
can read and use it, and people get it fast. The tools that check these
rules, and why we chose them, are in
[decision 0028](decisions/0028-web-checks.md).

Every rule has a stable ID so checks, reviews and issue files can cite it,
and names its **enforcer**:

- **Lighthouse**: [Lighthouse CI](#lighthouse-ci) asserts the audit or score
  on the site's build in PR CI and on the live site after deploy.
- **Site check**: `chores site-check`, an offline Rust check of the build
  output, in `mise run //:audit` and PR CI.
- **Field**: real-user Core Web Vitals from Cloudflare Web Analytics, read
  by `metric` evals in the `observe` stage ([evals.md](evals.md)). It
  reports; it doesn't block.
- **Review**: the PR reviewer and the `code-review` rubric.

Until a check lands (see [Rollout](#rollout)), review enforces its rule.
Accessibility, which agents depend on too, stays in [testing.md](testing.md)
(axe in Playwright) and is asserted by Lighthouse here (WEB-204).

## Which rules apply

Each site is one of two kinds:

- **Public**: meant to be found (sparkhill's website, Blue Eel, the
  cookbook's recipes, bear_down, controlled_chaos, the design system).
  Every rule applies.
- **Private**: signed-in or internal (pane, cahokia's family tree). It is
  kept out of search and agent indexes (WEB-104) and only the performance
  rules (WEB-3xx) and WEB-101 to WEB-102 apply.

A site's kind, URL, build folder and the pages to test are listed in
`standards/web.toml` (WEB-401). A site with both kinds of page (the
cookbook: public recipes, a private management app) lists each part.

## Summary

| ID | Rule | Enforcer |
| --- | --- | --- |
| WEB-101 | Every page has a unique title and description | Lighthouse (`document-title`, `meta-description`), site check |
| WEB-102 | `lang`, charset and viewport on every page | Lighthouse (`html-has-lang`, `viewport-insight`), site check |
| WEB-103 | One canonical URL per page | Lighthouse (`canonical`), site check |
| WEB-104 | Index what is public, nothing else | Lighthouse (`is-crawlable`), site check |
| WEB-105 | `robots.txt` and `sitemap.xml` on public sites | Lighthouse (`robots-txt`), site check |
| WEB-106 | Real status codes: missing pages are 404 | Lighthouse (`http-status-code`), `http` eval |
| WEB-107 | Links are real links with descriptive text | Lighthouse (`crawlable-anchors`, `link-text`) |
| WEB-108 | Social cards: Open Graph tags | Site check |
| WEB-109 | Images have `alt`, size and a modern format | Lighthouse (`image-alt`, `unsized-images`, `image-delivery-insight`) |
| WEB-110 | SEO score at least 95 | Lighthouse |
| WEB-201 | Public content is in the HTML, not only in JavaScript | Site check, Lighthouse |
| WEB-202 | An `llms.txt` that follows llmstxt.org | Site check, Lighthouse (`llms-txt`) |
| WEB-203 | A Markdown copy of each content page | Site check |
| WEB-204 | Semantic HTML and a clean accessibility tree | Lighthouse (`agent-accessibility-tree`, accessibility at least 95), axe |
| WEB-205 | Structured data as JSON-LD | Site check |
| WEB-206 | State the AI crawler stance in `robots.txt` | Site check |
| WEB-207 | Agentic Browsing score at least 90 | Lighthouse (warning while experimental) |
| WEB-208 | WebMCP for apps agents act in (assess) | Review |
| WEB-301 | Core Web Vitals "good" for real users | Field |
| WEB-302 | Performance score at least 90 on mobile | Lighthouse |
| WEB-303 | Lab timing budgets | Lighthouse |
| WEB-304 | Size budgets for the first load | Lighthouse (`resource-summary`) |
| WEB-305 | Hashed assets cached forever, HTML revalidated | Site check, Lighthouse (`cache-insight`) |
| WEB-306 | Something useful paints before the wasm runs | Lighthouse (`first-contentful-paint`, `font-display-insight`), review |
| WEB-307 | Release builds are optimized for size | Review, Lighthouse (WEB-304) |
| WEB-308 | No render-blocking third-party code | Lighthouse (`render-blocking-insight`, `third-parties-insight`) |
| WEB-401 | Every site is listed in `standards/web.toml` | Site check |
| WEB-402 | Lighthouse CI on every PR and after every deploy | CI |
| WEB-403 | A baseline that only shrinks | Site check, CI |
| WEB-404 | New assertions warn for a week first | CI |
| WEB-405 | Red results become issue files | Eval triage |

## WEB-1xx: search

- **WEB-101: every page has a unique title and description.** `<title>`
  names the page then the site (`Pumpkin bread · Cookbook`), at most 60
  characters. `<meta name="description">` says what the page offers in 70
  to 160 characters. No two pages of a site share either. For a Dioxus app
  each route sets them with `document::Title` and `document::Meta`, and the
  prerendered HTML (WEB-201) must carry them.
- **WEB-102: `lang`, charset and viewport on every page.** `<html lang="en">`
  (or the page's language), `<meta charset="utf-8">` first in `<head>`, and
  `<meta name="viewport" content="width=device-width, initial-scale=1">`.
- **WEB-103: one canonical URL per page.** Each public page has
  `<link rel="canonical">` with its absolute production URL, so the
  `*.pages.dev` host, a custom domain, trailing slashes and query strings
  don't split its ranking. Pick one host per site (its custom domain once it
  has one) and redirect the others to it with a `301` in `_redirects`.
- **WEB-104: index what is public, nothing else.** Private sites and pages
  send `X-Robots-Tag: noindex, nofollow` from `_headers` and disallow
  everything in `robots.txt`. Preview deploys are never indexed: Pages adds
  `X-Robots-Tag: noindex` to preview URLs, and our `_headers` must not
  remove it. A public page never carries `noindex` by mistake
  (Lighthouse's `is-crawlable`).
- **WEB-105: `robots.txt` and `sitemap.xml` on public sites.** `robots.txt`
  is valid, names the sitemap (`Sitemap: https://.../sitemap.xml`) and
  follows WEB-206. `sitemap.xml` lists every public, canonical page with
  its `lastmod`, and nothing that redirects, 404s or is `noindex`. Generate
  it at build time from the same route list the app uses, never by hand.
- **WEB-106: real status codes: missing pages are 404.** An unknown path
  answers `404` with a helpful page, not the app shell with `200` (a "soft
  404"). On Pages that means shipping a `404.html` and prerendered pages for
  the real routes (WEB-201); a pure single-page app with no `404.html`
  returns 200 for everything ([evals.md](evals.md#checking-a-live-page)).
  Moved pages redirect with `301`.
- **WEB-107: links are real links with descriptive text.** Navigation uses
  `<a href>` (Dioxus `Link`), never a click handler on a `div`, so crawlers
  and agents can follow it. Link text says where it goes ("Pumpkin bread
  recipe"), not "click here".
- **WEB-108: social cards: Open Graph tags.** Public pages set `og:title`,
  `og:description`, `og:url` (the canonical URL), `og:type` and `og:image`
  (an absolute URL, 1200 by 630), plus `twitter:card` =
  `summary_large_image`, so links shared in chat and social apps show a
  card.
- **WEB-109: images have `alt`, size and a modern format.** Every `<img>`
  has `alt` (empty for decoration), `width` and `height` (or a CSS aspect
  ratio) so it doesn't shift the layout, and is served as WebP or AVIF at
  the size it is shown. Below-the-fold images use `loading="lazy"`; the
  largest image above the fold doesn't.
- **WEB-110: SEO score at least 95.** Lighthouse's SEO category scores at
  least 0.95 on every tested page of a public site.

## WEB-2xx: agents

AI agents read sites in three ways: crawlers and fetch tools read the raw
HTML (most run no JavaScript), browsing agents read the accessibility tree,
and some read `llms.txt` and Markdown first. These rules make all three
work. Sources: [llmstxt.org](https://llmstxt.org/), Lighthouse's
[Agentic Browsing audits](https://developer.chrome.com/docs/lighthouse/agentic-browsing/llms-txt),
[schema.org](https://schema.org/).

- **WEB-201: public content is in the HTML, not only in JavaScript.** A
  public page's main content (headings, text, links, structured data) is in
  the HTML the server sends, so it is readable with JavaScript off. For
  Dioxus apps that means static generation (prerendering each public route
  at build time) or server rendering; Perseus sites already do this. The
  app may still hydrate and add interaction on top. The site check fails a
  public page whose `<body>` has no text outside `<script>` and
  `<noscript>`.
- **WEB-202: an `llms.txt` that follows llmstxt.org.** Public sites serve
  `/llms.txt`: an `# H1` with the site's name, a `>` one-paragraph summary,
  then `##` sections of Markdown links (`- [Title](url): what it is`) to
  the pages an agent most needs, with the less important ones under
  `## Optional`. It is generated at build time with the sitemap. Large
  content sites may add `/llms-full.txt` with the full text.
- **WEB-203: a Markdown copy of each content page.** Each content page
  (a recipe, an article, a guide) is also served as Markdown at the same
  path plus `.md` (`/recipe/pumpkin-bread.md`), linked from `llms.txt` and
  from the page with `<link rel="alternate" type="text/markdown">`. This is
  our free stand-in for Cloudflare's Markdown for Agents, which needs a paid
  plan. App screens (a form, a game) don't need one.
- **WEB-204: semantic HTML and a clean accessibility tree.** One `<h1>` per
  page and headings in order; landmarks (`<header>`, `<nav>`, `<main>`,
  `<footer>`); buttons are `<button>` and inputs have labels; ARIA only
  where HTML has no element for it. Lighthouse's `agent-accessibility-tree`
  passes and its accessibility category scores at least 0.95; axe runs in
  the Playwright tests ([testing.md](testing.md)).
- **WEB-205: structured data as JSON-LD.** Public pages describe what they
  are in one `<script type="application/ld+json">` block using
  [schema.org](https://schema.org/) types: `WebSite` and `Organization` on
  the home page, and the type that fits each content page (`Recipe` for the
  cookbook with `recipeIngredient` and `recipeInstructions`, `Article`,
  `SoftwareApplication` or `VideoGame` for the apps and games,
  `BreadcrumbList` where there is a path). The site check fails JSON that
  doesn't parse, an unknown `@type`, or a type missing the properties
  Google requires for it. Check a new type once by hand in Google's Rich
  Results Test, since Lighthouse's `structured-data` audit is manual.
- **WEB-206: state the AI crawler stance in `robots.txt`.** Public sites
  allow search engines and the agents that fetch a page because a person
  asked (`ChatGPT-User`, `Claude-User`, `Perplexity-User`), and block
  crawlers that collect training data (`GPTBot`, `ClaudeBot`, `CCBot`,
  `Google-Extended`, `Applebot-Extended`, `Bytespider`, `meta-externalagent`).
  The list lives once in `standards/web.toml`, and the build writes it into
  each site's `robots.txt`, so a change is one edit. The site check fails a
  `robots.txt` that doesn't match it.
- **WEB-207: Agentic Browsing score at least 90.** Lighthouse's Agentic
  Browsing category scores at least 0.9. It is asserted as a **warning**
  while Google calls the category experimental; then it fails like the
  others.
- **WEB-208: WebMCP for apps agents act in (assess).** For the interactive
  apps where an agent would do something for a user (plan a week of meals,
  add a person to a family tree), consider registering WebMCP tools or
  declarative forms, which Lighthouse checks with its `webmcp-*` audits.
  The API is new, so this is not required yet; a project that tries it
  records the result on the [tech radar](tech-radar.md).

## WEB-3xx: performance

Targets are Google's "good" thresholds for
[Core Web Vitals](https://web.dev/articles/vitals), at the 75th percentile
of page loads.

- **WEB-301: Core Web Vitals "good" for real users.** Largest Contentful
  Paint (LCP) at most 2.5 s, Interaction to Next Paint (INP) at most
  200 ms and Cumulative Layout Shift (CLS) at most 0.1, at p75 over 7 days
  on phones and desktops, from Cloudflare Web Analytics (decision 0001).
  Each site's `evals.toml` has a `metric` eval for each in the `observe`
  stage. A site that stays analytics-free is covered by WEB-302
  and WEB-303 alone.
- **WEB-302: performance score at least 90 on mobile.** Lighthouse's
  performance category scores at least 0.9 with its default mobile
  emulation (slow 4G, a mid-range phone CPU), median of three runs.
- **WEB-303: lab timing budgets.** In the same runs: LCP at most 2.5 s,
  Total Blocking Time (the lab stand-in for INP) at most 200 ms, CLS at most
  0.1, First Contentful Paint at most 1.8 s.
- **WEB-304: size budgets for the first load.** Compressed bytes over the
  wire for a page's first load: at most 1 MB in total, of which at most
  600 KB is JavaScript and wasm together, and at most 30 requests. A site
  that needs more says why in its `lighthouserc.json` and in review; the
  budget then only goes down (WEB-403).
- **WEB-305: hashed assets cached forever, HTML revalidated.** Files whose
  names carry a content hash (the `dx bundle` wasm and JavaScript, built CSS)
  are served with `Cache-Control: public, max-age=31536000, immutable`
  from `_headers`. HTML, `robots.txt`, `sitemap.xml` and `llms.txt` are
  served with `Cache-Control: public, max-age=0, must-revalidate`, so a
  deploy shows at once. Cloudflare compresses text, JavaScript and wasm
  itself; don't turn it off.
- **WEB-306: something useful paints before the wasm runs.** The first
  HTML shows the page (prerendered, WEB-201) or at least the site's header
  and a loading state styled inline, never a blank page while the wasm
  downloads. Preload the wasm and the main font
  (`<link rel="preload">`), and use `font-display: swap`.
- **WEB-307: release builds are optimized for size.** Web release profiles
  use `opt-level = "z"` or `"s"`, `lto = true`, `codegen-units = 1`,
  `panic = "abort"` and `strip = true`, and run `wasm-opt`
  (`dx bundle --release` does). Don't pull a large crate into the web build
  for one function.
- **WEB-308: no render-blocking third-party code.** Third-party scripts
  (PostHog's replay, Web Analytics' beacon) load `async` or `defer` after the
  page, never in the critical path, and their share shows in Lighthouse's
  `third-parties-insight`.

## WEB-4xx: process

- **WEB-401: every site is listed in `standards/web.toml`.** One entry per
  deployed site or part of a site: its Pages project, production URL, build
  folder, kind (`public` or `private`), the pages Lighthouse tests (the home
  page and one of each page type, at most three) and its baseline
  (WEB-403). The site check fails a `_headers` file under a project that has
  no entry. The AI crawler lists (WEB-206) live here too.
- **WEB-402: Lighthouse CI on every PR and after every deploy.** A
  workflow runs Lighthouse CI on each site a pull request changes, against
  its build output, and after each production deploy against the live
  URL, with the assertions above in the site's `lighthouserc.json`. See
  [Lighthouse CI](#lighthouse-ci).
- **WEB-403: a baseline that only shrinks.** A site that fails a rule today
  lists it in its `standards/web.toml` baseline with the measured value (a
  score, a size). The check fails a result worse than the baseline and asks
  for the entry to be tightened when the result is better, so the baseline
  only goes one way, like the code quality ratchet (QUAL-501) and the user
  docs baseline. A new site starts with no baseline.
- **WEB-404: new assertions warn for a week first.** A new Lighthouse
  assertion or site check runs as a warning for at least a week, then fails
  CI, so a noisy lab number is caught before it blocks anyone (QUAL-507).
- **WEB-405: red results become issue files.** A failing rule on a live
  site is fixed through the normal loop: the `cd` and `observe` evals turn
  red, and eval triage writes a `FIX` in the project's `issues/`
  ([evals.md](evals.md#continuous-improvement)). Each public site's
  `docs/requirements.md` has a requirement that it meets this standard,
  with these evals linked to it.

## Lighthouse CI

How the checks in decision 0028 run:

- **Tool:** `@lhci/cli`, exact version pinned (0.16.1, with Lighthouse
  13.6.0, when this was written), run with `npx` in a GitHub Actions job on
  `ubuntu-24.04` with `CHROME_PATH` set to the runner's Google Chrome. No
  browser download (decision 0005). Agents can run the same command in a
  cloud sandbox against a local build, since the sandbox has Chrome and
  npm.
- **Config:** each site's `lighthouserc.json` next to its `_headers`:
  `collect.staticDistDir` (the build folder) or `collect.url` (production),
  `collect.numberOfRuns = 3`, and `assert.assertions` with the scores and
  budgets above (`categories:seo`, `categories:performance`,
  `categories:accessibility`, `categories:agentic-browsing` and the audit
  IDs in the summary table). Use `"error"` for a rule that is live,
  `"warn"` for one in its first week (WEB-404) or experimental (WEB-207).
- **Reports:** `upload.target = "filesystem"`, kept as a 14-day Actions
  artifact and linked from the job summary. Never
  `temporary-public-storage` and no LHCI server (decision 0028).
- **Cost:** $0. About 1 to 2 minutes of standard runner time per site per
  run, free on a public repo ([free-tiers.md](free-tiers.md)).

## Rollout

The checks land in phases, each an issue file:

1. `issues/web-standard-phase-1-site-list.md`: `standards/web.toml` with
   every site, its kind and its pages.
2. `issues/web-standard-phase-2-lighthouse-ci.md`: the Lighthouse CI
   workflow and a `lighthouserc.json` per site, warnings only, with the
   measured baseline.
3. `issues/web-standard-phase-3-site-check.md`: `chores site-check`.
4. One issue per public site for its gaps (prerendering, `llms.txt`,
   sitemap, JSON-LD), starting with the cookbook and sparkhill's website.
5. Field Core Web Vitals once Web Analytics is on (decision 0001).
