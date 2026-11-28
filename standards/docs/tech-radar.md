# Tech radar

The tech radar lists every technology, technique and vendor this monorepo has
an opinion on: what is approved, what is under evaluation, what to avoid, and
which projects show each one in use. It follows Thoughtworks'
[Technology Radar](https://www.thoughtworks.com/radar) and their advice on
[building your own](https://www.thoughtworks.com/insights/blog/build-your-own-technology-radar)
and on [an enterprise radar](https://www.thoughtworks.com/insights/blog/technology-strategy/how-to-create-your-enterprise-technology-radar).

The data lives in [`standards/radar/radar.json`](../radar/radar.json). The
tables below are generated from it, so edit the JSON, never the tables.

## Rings

Each entry (a "blip") sits in one ring. Thoughtworks' rings, with what they
mean here:

| Ring | Thoughtworks | Here |
| --- | --- | --- |
| **Adopt** | Safe default, "as close to a no brainer as possible". | Approved, and the default for its job. Use it without asking. |
| **Trial** | Shown to work on real project work in certain circumstances. | Approved for limited use: an exception with conditions (RapidAPI on its free plan, inkwell for compilers), or chosen but not yet rolled out (PostHog, D1). Use it within its conditions. |
| **Assess** | Worth exploring to see how it would affect you. | Under evaluation: an open decision record or PR, or named in the standards but never tried. Don't use it in a project until it moves in. |
| **Hold** | Don't start anything new with it. | Rejected, replaced or risky. Existing use may stay; new use needs a decision record. |

Every Assess blip links the PR or decision that evaluates it. A vendor moves
into Adopt or Trial only with Jim's sign-off, as the vendor rule in
[AGENTS.md](../../AGENTS.md) says.

## Selection criteria

When choosing between options, in this order:

1. **Cost.** Cost always wins. Every blip has a monthly budget, and it is
   $0 (free plans and open source, [free-tiers.md](free-tiers.md)) unless
   Jim sets one. Today only Claude Code has one: $25 a month (Jim,
   2026-11-26). Raising a budget or giving a blip a new one is Jim's
   decision, so a PR that does is labelled `needs-human`.
2. **Well supported by Claude.** Claude Code is our AI coding platform, so
   prefer technologies Claude knows well and vendors that have a Claude Code
   plugin or an MCP server (Jim, 2026-11-26). We enable those integrations
   **read-only**, to give Claude context (logs, analytics, schemas,
   settings); changes still go through the repo, its workflows and PRs. The
   "Claude support" column below says what exists for each blip.
3. **Fewest vendors.** Reach for an approved vendor first; any other needs
   Jim's sign-off ([AGENTS.md](../../AGENTS.md)).

## Quadrants

The four Thoughtworks quadrants, kept stable so change stays visible:

- **Techniques:** how we work and structure software (processes, policies,
  conventions).
- **Tools:** tools we develop, test and run the repo with.
- **Platforms:** what software runs on or talks to: hosting, databases and
  every outside vendor.
- **Languages & Frameworks:** languages, frameworks and libraries.

## Fields

Each blip in `radar.json` has the five columns Thoughtworks'
[Build Your Own Radar](https://www.thoughtworks.com/radar/byor) tool reads,
plus five of ours:

| Field | Meaning |
| --- | --- |
| `name` | The specific technology, not a broad category. |
| `ring` | `Adopt`, `Trial`, `Assess` or `Hold`. |
| `quadrant` | `Techniques`, `Tools`, `Platforms` or `Languages & Frameworks`. |
| `isNew` | `"TRUE"` if the blip was added or moved in the last refresh, else `"FALSE"`. Shown as "(new)" below. |
| `description` | A sentence or two on why it sits where it does, and any conditions. |
| `kind` | `vendor` for an outside service with an account, else `technology`. |
| `exemplars` | Repo paths of the projects to copy from (for a Hold blip, where it still lives). Empty means no project uses it yet; the first that does becomes its exemplar. |
| `sources` | Repo paths or URLs of the rules, decisions and PRs behind the placement. |
| `claude` | The Claude Code plugin or MCP server for it, if any (empty when none is known). |
| `budget` | The most we spend on it a month, in US dollars. `0` means free plans and open source only. |
| `billing` | Vendors only: how its billing is checked (Jim, 2026-11-27: "the billing check should check every vendor in the tech radar"). `api` names the checks [billing-check.sh](../scripts/billing-check.sh) runs weekly with a read-only credential, `human` is the `standards/settings.toml` id Jim checks (or `""`), and `note` says why. An Adopt or Trial vendor needs at least one of them; one with no account says so in `note`. |

To see it as a picture, give the file's raw GitHub URL to the BYOR tool, or
render it in pane later. The hosted BYOR tool now only accepts its own outer
ring name, "Caution", so map Hold to Caution there, or self-host
[build-your-own-radar](https://github.com/thoughtworks/build-your-own-radar),
which keeps custom names.

## Changing the radar

- **Any PR that adds, approves, rejects or drops a technology or vendor
  updates `radar.json` in the same PR**, then runs
  `bash standards/scripts/radar.sh render`. That includes a decision record
  that changes a ring, and a project that becomes the first user (the
  exemplar) of a blip.
- `mise run ci` in `standards` runs `radar.sh check`: valid rings, quadrants
  and fields, unique names, exemplar and source paths that exist, a source
  for every Assess blip, a billing check for every approved vendor
  (RAD-007), and tables that match the JSON. A vendor that moves into Trial
  or Adopt gets its `billing` check in the same PR: an API check in
  `billing-check.sh` where its API allows, otherwise a settings check.
- **Refresh it twice a year** (Thoughtworks says at least once a year, twice
  is better): set `isNew` to `"FALSE"` on blips that didn't move, move blips
  whose evaluation finished, and drop Hold blips nobody has needed for a year.
  Pending changes this edition knows about are noted in the blips themselves.

## The radar

First edition: 2026-11-26.

<!-- radar:start (generated by standards/scripts/radar.sh render; edit radar.json instead) -->

Total budget: **$25 a month** (Claude Code $25). Everything else is $0: free plans and open source.

### Techniques

| Blip | Ring | Kind | Budget a month | Exemplar projects | Why | Claude support | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Agent Skills (new) | Adopt | technology | $0 | [.agents/skills](../../.agents/skills) | Repeatable agent procedures as skills in .agents/skills, validated on every PR, so the repo runs without a local clone. | Native to Claude Code: skills in .claude/skills load automatically. | [AGENTS.md](../../AGENTS.md) |
| Decision records in the repo (new) | Adopt | technology | $0 | [standards](../../standards), [bear_down](../../bear_down) | Plans, decisions and retros are Markdown committed next to what they are about, never chat or hosted docs. | - | [README.md](../../standards/docs/decisions/README.md), [AGENTS.md](../../AGENTS.md) |
| Draft-first PRs with an agent merge rule (new) | Adopt | technology | $0 | none yet | PRs open as drafts, go ready only when green, and any agent may squash-merge a mergeable, approved PR. PR triage drives every PR to closure. | - | [pull-requests.md](../../standards/docs/pull-requests.md) |
| Fewest vendors, cost first (new) | Adopt | technology | $0 | none yet | Reach for GitHub, Cloudflare and Squarespace first. Any other vendor needs Jim's sign-off in a decision record, unless it is clearly cheaper. | - | [AGENTS.md](../../AGENTS.md) |
| Files before databases (new) | Adopt | technology | $0 | none yet | Keep data and results as committed Markdown, JSON, JSON Lines, CSV or TOML; when real queries are needed, commit a SQLite .sql dump read with rusqlite; a hosted database (D1) is the last resort (AGENTS.md, decision 0004). | - | [AGENTS.md](../../AGENTS.md), [0004-database-engine.md](../../standards/docs/decisions/0004-database-engine.md) |
| Free tiers only (new) | Adopt | technology | $0 | [standards](../../standards) | Every service runs on its free plan; when a limit is near, stop rather than pay. Anything automation creates on demand goes through the free-tiers guard. | - | [free-tiers.md](../../standards/docs/free-tiers.md), [AGENTS.md](../../AGENTS.md) |
| Nightly CI and CD (new) | Adopt | technology | $0 | [.github/workflows](../../.github/workflows) | Every project runs CI every night, and every deployable project redeploys the latest main that passed, so drift shows up within a day. | - | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md) |
| One file per issue (new) | Adopt | technology | $0 | [cookbook](../../cookbook), [pane](../../pane) | Each TODO or FIX is one Markdown file with front matter in the docs/issues/ folder of the project it belongs to, instead of GitHub issues or shared ISSUES.md lists, so parallel PRs don't conflict; tasks only a person can do go in that project's docs/issues/human/ (decisions 0002 and 0006). The list is generated by mise run //:issues. | - | [issue-management.md](../../standards/docs/issue-management.md), [0002-human-issues.md](../../standards/docs/decisions/0002-human-issues.md), [0006-one-file-per-issue.md](../../standards/docs/decisions/0006-one-file-per-issue.md) |
| Shared design tokens (design_system) (new) | Adopt | technology | $0 | [pane/web](../../pane/web) | One TOML source of colour, type and spacing tokens for five themes (earthy, airy, professional, colorful, gamey) in light, dark and accessible modes, plus custom app themes such as bear_down's Bears theme, generated as CSS custom properties for web apps and Rust constants for Bevy games, with WCAG contrast checked in CI. In use by pane/web (professional theme) since 2026-11-26; the other apps follow the plan's adoption order. | - | [2026-11-design-system.md](../../standards/docs/plans/2026-11-design-system.md), [PR 657](https://github.com/jimhillr/monorepo/pull/657), [README.md](../../design_system/README.md) |
| Preview deploys and recorded smoke UAT (new) | Trial | technology | $0 | [cookbook/web](../../cookbook/web) | A preview per PR and @smoke Playwright tests recorded against previews and production, linked from the PR. Accepted in decision 0001; rollout under way. | - | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md), [2026-11-deploy-feedback.md](../../standards/docs/plans/2026-11-deploy-feedback.md) |
| Eval engine for product requirements (new) | Assess | technology | $0 | none yet | Proposed in decision 0003: requirements with stable IDs, evals tied to each, verdicts computed after CI, CD, chaos runs and observation, plus in-app feedback from Jim. Results stay in committed files. | - | [PR 612](https://github.com/jimhillr/monorepo/pull/612) |
| Infrastructure as code synthesized from Rust (new) | Assess | technology | $0 | [infra](../../infra) | Accepted in decision 0007 (Jim, 2026-11-26): Rust constructs write committed Terraform JSON (*.tf.json), PR CI plans it, and main applies the saved plan, so generation and apply are decoupled. Dashboard settings move into code; drift is checked weekly. | - | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |
| Shell scripts | Hold | technology | $0 | [standards](../../standards) | The standards say to script in Rust, not shell. The repo scripts in standards/scripts are the existing exception; don't add shell scripts to projects. | - | [README.md](../../standards/docs/README.md) |

### Tools

| Blip | Ring | Kind | Budget a month | Exemplar projects | Why | Claude support | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cargo-deny (new) | Adopt | technology | $0 | [standards](../../standards) | RustSec advisories and crate sources for every Cargo.lock (SEC-003), as warnings: in the hooks and PR CI where a lockfile changed, and nightly over all of them. Pinned in standards/scripts/deps-check.sh; config in standards/deny.toml. Licence checks come later. | - | [security.md](../../standards/docs/security.md), [deny.toml](../../standards/deny.toml) |
| cargo-llvm-cov (new) | Adopt | technology | $0 | [standards](../../standards) | Line coverage of every Rust project each night (COV-001, a warning under 80% for now), stored on the coverage-results branch with the README badge (Jim, 2026-11-27). Pinned in standards/scripts/coverage.sh. | - | [testing.md](../../standards/docs/testing.md) |
| Claude Code (new) | Adopt | vendor | $25 | [.agents/skills](../../.agents/skills) | Our AI coding platform (Jim, 2026-11-26): agents work the projects' docs/issues/ backlogs, open and triage PRs, and run the scheduled reviews and digests. Its support shapes other choices: prefer technologies and vendors Claude knows well or that have a Claude plugin or MCP server we can enable read-only. The one paid service: budget $25 a month (Jim, 2026-11-26). | It is Claude. Skills, plugins and MCP servers extend it. | [AGENTS.md](../../AGENTS.md), [CLAUDE.md](../../CLAUDE.md) |
| criterion | Adopt | technology | $0 | none yet | Benchmarks and regression reports. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| cross | Adopt | technology | $0 | none yet | Always use it for cross compilation. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| gitleaks (new) | Adopt | technology | $0 | [standards](../../standards) | Secret scanning (SEC-001): the staged diff in pre-commit, the pushed commits in pre-push, the PR's commits in PR CI and the whole history weekly. Pinned in standards/scripts/secrets-check.sh; our rules in .gitleaks.toml. | - | [security.md](../../standards/docs/security.md), [2026-11-threat-check.md](../../standards/docs/plans/2026-11-threat-check.md) |
| lefthook (new) | Adopt | technology | $0 | [standards](../../standards) | Git hooks that run the audit and each affected project's CI before every commit and push. Making them work in cloud sandboxes is an open TODO. | - | [git-hooks.md](../../standards/docs/git-hooks.md), [issues](../../standards/docs/issues) |
| Marp (new) | Adopt | technology | $0 | [docs/reviews](../../docs/reviews) | Slide decks written in Markdown, used for the weekly deep project reviews. | - | [SKILL.md](../../.agents/skills/deep-project-review/SKILL.md) |
| mise (new) | Adopt | technology | $0 | [standards](../../standards), [cookbook](../../cookbook) | The task runner and tool installer for every project: the same ci, fix, build, test, dev, release and deploy tasks everywhere, from shared files in standards/tasks. | - | [mise.md](../../standards/docs/mise.md), [2026-11-cast-to-mise.md](../../standards/docs/plans/2026-11-cast-to-mise.md) |
| Playwright | Adopt | technology | $0 | [cookbook/web](../../cookbook/web), [cahokia/web](../../cahokia/web) | Browser end-to-end tests, and the recorded smoke UAT (screenshots, video, trace) that decision 0001 makes the default evidence on PRs. | Playwright MCP server (Microsoft). | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md) |
| samply | Adopt | technology | $0 | none yet | The profiler. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| zizmor (new) | Adopt | technology | $0 | [standards](../../standards) | GitHub Actions workflow audit (SEC-005), as warnings: in the hooks and PR CI (Workflow audit) where a workflow changed, and nightly over all of them. Offline. Pinned in standards/scripts/workflows-check.sh; ignores in .github/zizmor.yml. | - | [security.md](../../standards/docs/security.md), [zizmor.yml](../../.github/zizmor.yml) |
| cargo-apk (new) | Trial | technology | $0 | [controlled_chaos](../../controlled_chaos) | Packages Bevy games as Android APKs for playtesting (release:android). Unmaintained upstream but still what Bevy 0.14's NativeActivity build supports; cross can't make an APK. Revisit (xbuild or Gradle with GameActivity) when Bevy is upgraded. | - | [mise.md](../../standards/docs/mise.md), [bevy-release.sh](../../standards/scripts/bevy-release.sh) |
| Kokoro text-to-speech (new) | Trial | technology | $0 | [docs/reviews](../../docs/reviews) | Open-weight voice (kokoro-onnx, Apache 2.0) that narrates the review deck demo videos, run locally with ffmpeg and Playwright's video recording. Brings Python in for that one script only. Decision 0025 (proposed). | - | [0025-review-deck-demo-videos.md](../../standards/docs/decisions/0025-review-deck-demo-videos.md), [link](https://github.com/thewh1teagle/kokoro-onnx) |
| trycmd and vhs (new) | Trial | technology | $0 | none yet | Transcript tests for CLIs, and terminal recordings for PRs that change CLI output. Chosen in decision 0001; rollout not started. | - | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md) |
| wasm-bindgen (new) | Trial | technology | $0 | [controlled_chaos](../../controlled_chaos) | Builds the web (WebAssembly) version of Bevy games for Cloudflare Pages, through the bevy-game release:web task. The CLI version must match wasm-bindgen in the game's Cargo.lock. | - | [mise.md](../../standards/docs/mise.md), [bevy-release.sh](../../standards/scripts/bevy-release.sh) |
| OpenTofu (new) | Assess | technology | $0 | [infra](../../infra) | Accepted in decision 0007 (Jim, 2026-11-26) as the IaC engine: plans and applies the Terraform JSON the infra crate synthesizes, with built-in state encryption and no account. Moves to Trial when its phase 1 merges. | OpenTofu MCP server (registry and resource docs) and HashiCorp's Terraform MCP server, read-only. | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |
| Playwright on a system browser (new) | Assess | technology | $0 | none yet | Proposed in decision 0005: launch Chromium tests on the machine's own Chrome instead of Playwright's download, so they run in sandboxes. Rust WebDriver (fantoccini, thirtyfour), CDP (chromiumoxide) and WebDriver BiDi clients were compared and not recommended yet. | Playwright MCP server (Microsoft). | [PR 616](https://github.com/jimhillr/monorepo/pull/616) |
| Cast (new) | Hold | technology | $0 | none yet | The old monorepo CLI, replaced by mise and removed. Don't restore it or its project templates. | - | [2026-11-cast-to-mise.md](../../standards/docs/plans/2026-11-cast-to-mise.md), [AGENTS.md](../../AGENTS.md) |
| Pulumi (new) | Hold | vendor | $0 | none yet | No Rust SDK, and its state service is a new vendor. Compared in decision 0007 and not chosen. | Pulumi MCP server. | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |

### Platforms

| Blip | Ring | Kind | Budget a month | Exemplar projects | Why | Claude support | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Auth0 (new) | Adopt | vendor | $0 | [auth_sdk](../../auth_sdk), [cookbook/web](../../cookbook/web), [cahokia/web](../../cahokia/web) | Approved exception, free plan. Always use it for sign-in (CIAM). Tests use mocks, never real tokens. | Official Auth0 MCP server. | [AGENTS.md](../../AGENTS.md), [README.md](../../standards/docs/README.md), [free-tiers.md](../../standards/docs/free-tiers.md) |
| Cloudflare Pages | Adopt | vendor | $0 | [cookbook/cloudflare](../../cookbook/cloudflare), [cahokia/cloudflare](../../cahokia/cloudflare), [controlled_chaos](../../controlled_chaos) | Approved vendor. Hosts the static web apps, deployed with wrangler pages deploy (direct upload, no Pages builds). New projects are created only through free-tiers.sh. | Cloudflare MCP servers (Claude connector directory) and the Cloudflare Claude Code plugin with Workers, Wrangler and D1 skills. | [AGENTS.md](../../AGENTS.md), [free-tiers.md](../../standards/docs/free-tiers.md) |
| GitHub (Actions, Dependabot, CodeQL) | Adopt | vendor | $0 | [.github/workflows](../../.github/workflows), [jimhillr/workflow_tests](../../jimhillr/workflow_tests), [infra/src/stacks/github.rs](../../infra/src/stacks/github.rs) | Approved vendor. Hosts the repo and runs all CI, CD, nightly runs and maintenance workflows on standard runners, which are free on this public repo. | Official GitHub MCP server, which has a read-only mode; Claude Code's own GitHub integration. | [AGENTS.md](../../AGENTS.md), [free-tiers.md](../../standards/docs/free-tiers.md) |
| Squarespace (new) | Adopt | vendor | $0 | none yet | Approved vendor. Jim's existing account; no automation or project in the repo uses it. | - | [AGENTS.md](../../AGENTS.md), [free-tiers.md](../../standards/docs/free-tiers.md) |
| Cloudflare D1 (new) | Trial | vendor | $0 | [tofu_state](../../tofu_state) | The hosted database (decision 0004, Jim 2026-11-26), used only when a deployed service must write data at runtime, until a need comes up that it can't meet. Part of an approved vendor, Free plan only. First used by tofu_state for OpenTofu state (decision 0009). | Cloudflare MCP servers (Claude connector directory) and the Cloudflare Claude Code plugin with Workers, Wrangler and D1 skills. | [0004-database-engine.md](../../standards/docs/decisions/0004-database-engine.md), [0009-opentofu-state-http-backend.md](../../standards/docs/decisions/0009-opentofu-state-http-backend.md) |
| Cloudflare Workers (new) | Trial | vendor | $0 | [pane/cloudflare](../../pane/cloudflare), [telemetry](../../telemetry), [tofu_state](../../tofu_state) | Approved vendor, Free plan only. Written in Rust with workers-rs: tofu_state is OpenTofu's http state backend (decision 0009), telemetry takes events into Analytics Engine with a rate-limit binding (decision 0001), and pane/cloudflare serves pane's Dioxus build as Workers static assets and runs Rust only for /api/* (standards/tasks/cloudflare-worker-assets.toml). Web Analytics is planned. Workers are created only through free-tiers.sh worker. | Cloudflare MCP servers (Claude connector directory) and the Cloudflare Claude Code plugin with Workers, Wrangler and D1 skills. | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md), [0009-opentofu-state-http-backend.md](../../standards/docs/decisions/0009-opentofu-state-http-backend.md), [README.md](../../telemetry/README.md) |
| PostHog (new) | Trial | vendor | $0 | [cookbook/web](../../cookbook/web) | Approved exception, free tier, for one job: real-user session replay (and game events) on production sites, with all text input masked. First used in cookbook (2026-11-27), loaded only on the production host. | PostHog MCP connector in the Claude directory. | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md), [AGENTS.md](../../AGENTS.md) |
| RapidAPI (Zillow data) (new) | Trial | vendor | $0 | [homer](../../homer) | Approved exception for homer only, and only on its free plan. It must never cost money: no paid plan and no scheduled runs. | - | [AGENTS.md](../../AGENTS.md) |
| Cloudflare Flagship (new) | Assess | technology | $0 | none yet | Cloudflare's feature flag service, in public beta with pricing after GA not announced, and no workers-rs binding. Decision 0018 builds a small flags Worker on D1 and a management app per project instead, and revisits Flagship at GA. | Cloudflare plugin (flagship skill) and the Cloudflare MCP server. | [0018-feature-flags.md](../../standards/docs/decisions/0018-feature-flags.md) |
| Neon (new) | Assess | vendor | $0 | none yet | Hosted Postgres with per-PR branches and scale to zero. Decision 0004 chose D1 first; Neon is revisited only if a need comes up that D1 can't meet. Not an approved vendor. | Neon MCP connector in the Claude directory. | [0004-database-engine.md](../../standards/docs/decisions/0004-database-engine.md) |
| SpacetimeDB | Assess | technology | $0 | none yet | Named in the standards README for real-time stateful sync, but no project has tried it. | - | [README.md](../../standards/docs/README.md) |
| Cloudflare R2 (new) | Hold | vendor | $0 | none yet | R2 bills past its free allowance once it is enabled, so don't enable it without a decision record. | Cloudflare MCP servers (Claude connector directory) and the Cloudflare Claude Code plugin with Workers, Wrangler and D1 skills. | [free-tiers.md](../../standards/docs/free-tiers.md) |
| Crossplane (new) | Hold | technology | $0 | none yet | Needs an always-on Kubernetes cluster, which no approved vendor offers free. Compared in decision 0007 and not chosen. | - | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |
| flagd, Unleash and GrowthBook (new) | Hold | technology | $0 | none yet | Open-source feature flag servers considered in decision 0018. Each needs a long-running server and a database with no free host among approved vendors, or a new hosted vendor, so don't start work on them. | - | [0018-feature-flags.md](../../standards/docs/decisions/0018-feature-flags.md) |
| Honeycomb, Sentry and Microsoft Clarity (new) | Hold | vendor | $0 | none yet | Considered for observability and session replay in decision 0001 and dropped in favour of Cloudflare plus PostHog, to keep the vendor count down. | Sentry MCP connector in the Claude directory. | [0001-deploy-feedback-loop.md](../../standards/docs/decisions/0001-deploy-feedback-loop.md) |
| RentCast, ATTOM and HouseCanary (new) | Hold | vendor | $0 | none yet | Property data alternatives to RapidAPI. RentCast's free plan bills overage with no hard cap; ATTOM and HouseCanary are paid only. | - | [free-tiers.md](../../standards/docs/free-tiers.md) |
| Supabase and Turso (new) | Hold | vendor | $0 | none yet | Compared in decision 0004 and not chosen: Supabase pauses idle projects and has paid-only branching; Turso blocks every database once any one limit is hit and is being acquired. | Supabase MCP connector in the Claude directory. | [0004-database-engine.md](../../standards/docs/decisions/0004-database-engine.md) |
| SurrealDB | Hold | technology | $0 | [luggage](../../luggage) | Used only by luggage (on hold). Decision 0004 puts committed files, committed SQLite and then D1 first, so don't start new work on it. | - | [README.md](../../standards/docs/README.md), [0004-database-engine.md](../../standards/docs/decisions/0004-database-engine.md) |
| Terraform CLI and HCP Terraform (new) | Hold | vendor | $0 | none yet | Business Source License, no state encryption, and HCP Terraform is a new vendor with a 500-resource free tier. OpenTofu reads the same providers and JSON (decision 0007). | HashiCorp Terraform MCP server. | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |

### Languages & Frameworks

| Blip | Ring | Kind | Budget a month | Exemplar projects | Why | Claude support | Sources |
| --- | --- | --- | --- | --- | --- | --- | --- |
| anyhow | Adopt | technology | $0 | [homer](../../homer) | Error handling in applications. | - | [README.md](../../standards/docs/README.md) |
| Axum | Adopt | technology | $0 | [luggage](../../luggage) | The HTTP server framework. Its only user, luggage, is on hold. | - | [README.md](../../standards/docs/README.md) |
| Bevy | Adopt | technology | $0 | [controlled_chaos](../../controlled_chaos) | The game engine. | - | [README.md](../../standards/docs/README.md) |
| Clap | Adopt | technology | $0 | [homer](../../homer) | Argument parsing for traditional CLIs. | - | [README.md](../../standards/docs/README.md) |
| Dioxus | Adopt | technology | $0 | [cookbook/web](../../cookbook/web), [cahokia](../../cahokia), [pane/web](../../pane/web) | The framework for web, desktop and mobile apps. New work uses 0.7; blueeel and sparkhill/webapp are still on 0.6, and cahokia's version is pinned on purpose (AGENTS.md). | - | [README.md](../../standards/docs/README.md), [AGENTS.md](../../AGENTS.md) |
| dioxus-ssr | Adopt | technology | $0 | [cahokia/ui](../../cahokia/ui) | Render Dioxus components in plain Rust tests to check text, links and classes without a browser. | - |  |
| nalgebra | Adopt | technology | $0 | none yet | Linear algebra. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| ndarray | Adopt | technology | $0 | none yet | N-dimensional arrays and matrices. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| Pingora | Adopt | technology | $0 | none yet | The reverse proxy framework. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| Ratatui | Adopt | technology | $0 | none yet | The framework for terminal UIs. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| Rayon | Adopt | technology | $0 | none yet | Data parallelism for compute-bound work. No project uses it yet. | - | [README.md](../../standards/docs/README.md) |
| Rust | Adopt | technology | $0 | [cookbook](../../cookbook), [homer](../../homer) | The language for all code in the repo unless an exception says otherwise. | Official rust-analyzer-lsp Claude Code plugin. | [README.md](../../standards/docs/README.md), [rust.md](../../standards/docs/rust.md) |
| thiserror | Adopt | technology | $0 | [auth_sdk](../../auth_sdk), [cahokia/ui](../../cahokia/ui) | Error types in libraries. | - | [README.md](../../standards/docs/README.md) |
| Tokio | Adopt | technology | $0 | [auth_sdk](../../auth_sdk), [luggage](../../luggage) | The async runtime for IO-bound work. | - | [README.md](../../standards/docs/README.md) |
| tracing | Adopt | technology | $0 | [auth_sdk](../../auth_sdk), [cahokia](../../cahokia) | The only logging library. | - | [README.md](../../standards/docs/README.md) |
| utoipa | Adopt | technology | $0 | [luggage](../../luggage) | OpenAPI documents generated from Axum handlers (spelled "Utopia" in the standards README). | - | [README.md](../../standards/docs/README.md) |
| inkwell (LLVM) | Trial | technology | $0 | [my_lang](../../my_lang) | LLVM bindings for the my_lang compiler. Needs LLVM headers, so it can't build in cloud sandboxes. Use only for compiler work. | - | [issues](../../standards/docs/issues) |
| pulldown-cmark (new) | Trial | technology | $0 | [pane/web](../../pane/web) | CommonMark Markdown to HTML in Rust, compiled to wasm; pane renders plans, decisions and retros with it, raw HTML shown as text. | - | [2026-11-review-dashboard.md](../../pane/docs/plans/2026-11-review-dashboard.md) |
| TypeScript and Node.js | Trial | technology | $0 | [cookbook/web](../../cookbook/web), [cahokia/web](../../cahokia/web) | Allowed only for VS Code extensions and the Playwright e2e specs. Jim dislikes the Node dependency; decision 0005 (proposed) keeps it limited to e2e tests. | Official typescript-lsp Claude Code plugin. | [typescript.md](../../standards/docs/typescript.md), [PR 616](https://github.com/jimhillr/monorepo/pull/616) |
| OpenFeature (new) | Assess | technology | $0 | none yet | CNCF standard API for feature flags. Decision 0018 shapes the in-house flags crate after it so a later provider change stays in one crate. | - | [0018-feature-flags.md](../../standards/docs/decisions/0018-feature-flags.md) |
| terrars (new) | Assess | technology | $0 | [infra](../../infra) | Generates strongly typed Rust bindings from Terraform provider schemas and writes .tf.json. Accepted in decision 0007 (Jim, 2026-11-26) as the typed layer under the infra crate's constructs. The phase 1 spike passed (2026-11-26): only the resource types we use are generated, which build in seconds. We fix bugs we find upstream rather than work around them (Jim, 2026-11-26). | - | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md), [0007-terrars-spike.md](../../standards/docs/decisions/0007-terrars-spike.md) |
| CDKTF and CDK Terrain (new) | Hold | technology | $0 | none yet | CDKTF was archived by HashiCorp on 2025-12-10; its community fork CDK Terrain supports TypeScript, Python and Go only. No Rust, and it needs Node (decision 0007). | - | [0007-infrastructure-as-code.md](../../standards/docs/decisions/0007-infrastructure-as-code.md) |
| Perseus and Sycamore 0.8 | Hold | technology | $0 | [profiles/jimhillr](../../profiles/jimhillr), [sparkhill/website](../../sparkhill/website) | Don't start new sites on them; use Dioxus. The existing sites stay pinned to sycamore 0.8 because perseus 0.4.2 needs it. | - | [AGENTS.md](../../AGENTS.md) |

### Billing checks

How the billing of each vendor is checked: weekly by [billing-check.sh](../scripts/billing-check.sh) through a read-only API, or by Jim through a [settings check](../settings.toml). See [Billing checks](free-tiers.md#billing-checks).

| Vendor | Ring | Read-only API check | Jim checks | Why |
| --- | --- | --- | --- | --- |
| Auth0 | Adopt | - | `auth0-billing` | The Management API has no plan, subscription or payment endpoint. |
| Claude Code | Adopt | - | `claude-code-budget` | claude.ai subscription billing has no API. |
| Cloudflare Pages | Adopt | cloudflare | `cloudflare-billing` | One Cloudflare account: Billing Read shows its payment methods and subscriptions. Jim's yearly check is a backstop. |
| GitHub (Actions, Dependabot, CodeQL) | Adopt | github | `github-actions-budget` | The billing usage API shows any charge this month. No API shows a personal account's card or Actions budget, so Jim checks those. |
| Squarespace | Adopt | - | `squarespace-billing` | No public API for account billing. Jim pays for several domains there (2026-11-27), which the settings check confirms are the only charges until the cost framework tracks them. |
| Cloudflare D1 | Trial | cloudflare | `cloudflare-billing` | Same account as Pages. D1 Free has no billing. |
| Cloudflare Workers | Trial | cloudflare | `cloudflare-billing` | Same account as Pages. The check fails on Workers Paid. |
| PostHog | Trial | posthog | - | GET /api/billing/ with a billing:read key shows any subscription or charge. Adding a card is what starts a subscription. |
| RapidAPI (Zillow data) | Trial | - | `rapidapi-free-plan` | No consumer billing API, and polling the quota header on a schedule would break homer's free-plan rule. |
| Neon | Assess | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |
| Cloudflare R2 | Hold | cloudflare | `cloudflare-billing` | Same account as Pages. The check fails if R2 paid is turned on. |
| Honeycomb, Sentry and Microsoft Clarity | Hold | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |
| Pulumi | Hold | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |
| RentCast, ATTOM and HouseCanary | Hold | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |
| Supabase and Turso | Hold | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |
| Terraform CLI and HCP Terraform | Hold | - | - | No account, so nothing can bill. Moving it to Trial or Adopt needs a billing check first (RAD-007). |

<!-- radar:end -->
