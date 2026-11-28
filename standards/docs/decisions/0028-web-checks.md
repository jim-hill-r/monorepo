# 0028. Check SEO, agent readiness and web performance with Lighthouse CI and a Rust site check

- Status: Proposed
- Decided by: Jim (on merging the pull request that adds this record)
- Date: 2026-11-28
- Asked by: Jim ("We should add some standards around seo, agent
  optimization, and website performance. Suggest a technology solution for
  these and add a standard.")

## Context

We deploy eight sites to Cloudflare Pages (`jimhillr-cookbook`,
`jimhillr-cahokia`, `jimhillr-pane`, `jimhillr-bear-down`,
`jimhillr-controlled-chaos`, `jimhillr-design-system`,
`jimhillr-sparkhill` and `jimhillr-sparkhill-webapp`). Most are
client-rendered Dioxus apps: Pages serves one `index.html` shell for every
path and the content arrives in a wasm bundle
([evals.md](../evals.md#checking-a-live-page)). Today none of them has a
`robots.txt`, a `sitemap.xml`, an `llms.txt` or structured data, only
`sparkhill/webapp` has a meta description and only `sparkhill/website`
(Perseus, server-rendered) has Open Graph tags. Nothing measures page speed.

The standard these checks enforce is [web.md](../web.md). This record is
about the tools.

What a tool must do (AGENTS.md):

- **$0**, on the free tiers we already use, with no new vendor unless it is
  clearly cheaper.
- Run **in GitHub Actions** on pull requests and after deploy, with no local
  clone.
- Use a browser we already have, not a download (decision 0005).
- Cover all three areas: search (SEO), AI agents, and performance.

### What was checked in this sandbox

Lighthouse 13.6.0 (the version `@lhci/cli` 0.16.1 depends on, published
2026-11-11) ran from npm on the sandbox's `/usr/bin/google-chrome` with no
browser download, against a small static page. Its report has five
categories: Performance, Accessibility, Best Practices, SEO and a new
**Agentic Browsing** category, scored like the others. Agentic Browsing's
audits are `agent-accessibility-tree`, `llms-txt`, `cumulative-layout-shift`,
`webmcp-form-coverage`, `webmcp-registered-tools`, `webmcp-schema-validity`
and `ard-schema`. Google added the category in Lighthouse 13.3 (May 2026)
and still calls it experimental.

Lighthouse leaves gaps that matter here:

- A missing `robots.txt` or `llms.txt` is **not applicable**, not a failure,
  so Lighthouse can't say a site must have one.
- `structured-data` is a **manual** audit: Lighthouse doesn't validate
  JSON-LD.
- It tests one URL at a time, so it can't check a sitemap against the pages
  or that every page has a unique title.
- Lab numbers on shared runners vary from run to run; they are not what real
  users see.

## Options

| | Money | Covers | New vendor | Runs where | Notes |
| --- | --- | --- | --- | --- | --- |
| **A. Lighthouse CI (`@lhci/cli`) in Actions** | $0 (Actions minutes are free on a public repo) | Performance, SEO, accessibility, best practices, agentic browsing | No (open source, Apache 2.0, run on our runner) | Built output on PRs (`staticDistDir`), production after deploy | Assertions and budgets in a `lighthouserc` file; Node, like Playwright already is |
| **B. Our own Rust site check (`chores site-check`)** | $0 | What Lighthouse can't: required files, JSON-LD, sitemap, titles across pages, `_headers` | No | Offline, on the build output | Small to write; deterministic, so no tokens per run |
| C. PageSpeed Insights API | $0 (25,000 queries a day with a key) | Lighthouse plus Chrome UX Report field data | Google (not approved) | Google's servers, public URLs only | Needs a key and a new vendor; can't test a PR's build |
| D. Unlighthouse | $0 | Lighthouse on every page of a crawled site | No | Actions | Same engine as A; useful later for whole-site scans, but weaker CI assertions |
| E. sitespeed.io | $0 | Performance in depth, some SEO | No | Actions (Docker image, about 2 GB) | Heavy; no agent checks |
| F. Cloudflare Observatory | Limited on the Free plan | Lighthouse tests from Cloudflare | No (Cloudflare) | Zones only | Our sites are on `*.pages.dev`, not a zone we own |
| G. Cloudflare Markdown for Agents | Pro plan and up | Serves pages as Markdown to agents | No (Cloudflare) | Zones only | Not on the Free plan, so ruled out by the free-tiers rule |
| H. Cloudflare Web Analytics | $0 | Real-user Core Web Vitals (LCP, INP, CLS) | No (Cloudflare) | The live site, as a beacon | Already planned in decision 0001; field data, not a gate |

## Decision

Use **A, B and H together**, each for what it does best:

1. **Lighthouse CI** (`@lhci/cli`, exact version pinned) runs in a GitHub
   Actions workflow on Google Chrome from the runner image (decision 0005:
   `CHROME_PATH`, no browser download):
   - on every pull request that changes a site, against the site's built
     output served by `lhci` itself (`collect.staticDistDir`), so it needs
     no preview deploy and no network;
   - after each production deploy, against the live URL, as a `cd` eval
     stage.
   Three runs per URL, median, mobile emulation (Lighthouse's default). Each
   site's assertions and budgets live in its own `lighthouserc.json`, as
   [web.md](../web.md) sets out. Reports are uploaded with
   `upload.target = "filesystem"` and kept as a 14-day Actions artifact.
   We **don't** use `temporary-public-storage` (reports sit on Google's
   storage, public, for 7 days) or run an LHCI server (it would need
   hosting and a database).
2. **`chores site-check`**, a new offline `chores` command, reads a site's
   build output and checks the rules Lighthouse can't (web.md says which):
   required files (`robots.txt`, `sitemap.xml`, `llms.txt`), that
   `llms.txt` follows [llmstxt.org](https://llmstxt.org/), JSON-LD that
   parses and has the required properties for its type, titles and
   descriptions unique across pages, sitemap entries that exist, and
   caching and `noindex` rules in `_headers`. Like every `chores` command
   it never uses the network. It runs in `mise run //:audit` and PR CI.
3. **Cloudflare Web Analytics** gives real-user Core Web Vitals for the sites
   that allow analytics, read by `metric` evals in the `observe` stage, as
   decision 0001 planned. It informs; it doesn't block a PR. A site that stays
   analytics-free relies on the lab checks alone.

Rejected: C needs a new vendor and can't test a PR; D and E duplicate A
with more weight; F and G don't work on `*.pages.dev`, and G isn't free.
WebMCP (the `webmcp-*` audits) is too new to require; it stays on the
radar to assess for the interactive apps.

## Consequences

- **Cost:** $0. Lighthouse CI adds about 1 to 2 minutes of standard runner
  time per site per run (three runs of one to three URLs), free on a public
  repo; reports add a few MB of artifacts, kept 14 days. No new vendor, no
  secret, no Cloudflare resource. Claude Code effort: this record and the
  standard now (one thread); the workflow and `chores site-check` are about
  one thread each; each site's fixes are product work, filed as issues.
- **Node stays** for one more tool. `lhci` is pinned and cached; it uses
  the runner's Chrome, so the download problem of decision 0005 doesn't
  come back. Cloud sandboxes already have Chrome and npm, so an agent can
  run it there against a build.
- **Lab numbers are noisy.** Medians of three runs and a week of warnings
  before any assertion fails (QUAL-507) keep flaky failures down; the
  field data from Web Analytics is the real measure.
- **Agentic Browsing is experimental.** Its score is asserted as a warning
  until Google drops the label; its `llms-txt` and accessibility audits are
  covered by `chores site-check` and the accessibility category meanwhile.
- **Most sites fail today.** Enforcement starts with a baseline per site in
  `standards/web.toml` (added with the check) that may only shrink, the same
  ratchet as decision 0015 and `standards/user-docs.toml`.
- The follow-up issues are listed in [web.md](../web.md#rollout).
