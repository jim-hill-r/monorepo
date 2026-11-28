# Product lifecycle

Every project starts on a subdomain of `jimhillr.com`. A product moves to a
domain of its own only when it earns it: it passes a promotion gate in
every domain of concern this repo tracks, and Jim approves an executive
review deck that makes the case
([decision 0029](decisions/0029-product-promotion-path.md)).

Jim (2026-11-28): "All projects will start out as a subdomain of
jimhillr.com with a promotion path to a fully fledged product of its own
with its own domain. We need to establish that path in the product
lifecycle and what would allow a promotion such as revenue goals, cost,
stability, quality etc... each domain of concern should be include in the
promotion path. An executive review deck needs to be made making the case
for promotion and reviewed by jim."

How a project lands on its subdomain and how a promoted product's domain
is pointed at it (DNS, certificates, redirects) is the jimhillr.com
hosting decision and its
[plan](plans/2026-11-jimhillr-com.md) (PR #961). This page says *when* a
product may move and *how the case is made*.

## Stages

| Stage | Where it is served | Who it is for | How it gets here |
| --- | --- | --- | --- |
| `incubating` | `<name>.jimhillr.com` (the Pages project or Worker name without `jimhillr-`, such as `bear-down.jimhillr.com`; see the hosting decision) | Early users, friends, Jim | Every new project starts here |
| `candidate` | Still `<name>.jimhillr.com` | The same users | An agent or Jim opens a promotion review (below) |
| `promoted` | Its own domain; `<name>.jimhillr.com` redirects there with a permanent `301`, forever | The public, as its own company | Jim approves the promotion review |

Two things sit beside the stage, not inside it:

- **Tier** ([decision 0019](decisions/0019-review-roles.md)). Only
  `product` projects can be promoted. `technical-platform` and
  `software-development-platform` projects (pane, standards, auth_sdk,
  design_system...) stay on a `jimhillr.com` subdomain, or have no site at
  all, because they serve us, not a market.
- **Status** (`active` or `on-hold`, [decision 0022](decisions/0022-project-metadata-and-product-docs.md)).
  An on-hold product keeps its stage and is not reviewed for promotion.

The stage will be recorded as `stage` in the project's own
`docs/project.toml` (decision 0022), added with the check that reads it.
Until then the stage is `incubating` for every project unless a merged
promotion review says otherwise.

A product moves back only through another review deck: a promoted product
that has failed a gate for 90 days gets a deck proposing a fix, a move back
to its subdomain, or a sunset, and Jim decides. The domain is kept (and
keeps redirecting) at least until its next renewal, so no link breaks.

## Promotion gates

A product is ready for promotion when every gate below is **Met**, with
evidence a reader can open. Only Jim can **waive** a gate, in his approval
of the deck, with the reason written in the deck. Each gate points at the
standard that owns its domain; when a standard tightens, the gate tightens
with it.

The window is the 30 days before the deck's date unless the gate says
otherwise.

| # | Domain | Gate | Evidence | Owning standard |
| --- | --- | --- | --- | --- |
| G1 | Mission and roadmap | `docs/mission.md` and `docs/roadmap.md` pass DOC-009 to DOC-012 with no `[pending]` exemption, and the roadmap's Now items are done or in flight | `chores product-docs` output | [decision 0022](decisions/0022-project-metadata-and-product-docs.md) |
| G2 | Requirements | `docs/requirements.md` has at least one `must` for each main journey, and Jim has approved them (requirements-reviewer) | The requirements file and the PRs that changed it | [evals.md](evals.md#requirements) |
| G3 | Users and traction | Real use that isn't us: a usage goal Jim sets in the deck, by default at least 50 distinct weekly visitors (or active users for a signed-in app) in each of the last 8 weeks, from Cloudflare analytics or the product's own counts, never a new tracking vendor | Weekly counts with their source | [free-tiers.md](free-tiers.md), the product's analytics rule (Blue Eel stays analytics-free, so it uses request counts) |
| G4 | Revenue | Self-funding: for each of the last 3 months the product's own revenue in the financial ledger is at least its own money cost, *including* the new domain's renewal spread over 12 months, plus any higher revenue goal Jim sets in the deck | `cost money` ledger lines attributed to the product | [decision 0012](decisions/0012-revenue-and-http-402.md), [decision 0016](decisions/0016-holding-and-handling-money.md), [cost.md](cost.md#accounting-rules) |
| G5 | Cost and free tiers | `docs/cost.toml` is current and `cost check` passes; every service the product uses has stayed green (under the yellow line) on the daily free-tier usage check for the window; the deck states the domain's yearly price and Jim sets its budget in his approval | `cost check` output, the usage check history | [cost.md](cost.md), [free-tiers.md](free-tiers.md) |
| G6 | Stability and observability | The product is watched by the observability project (watchdog and logs) with paging into incident issues; uptime at least 99.5% in the window; no open incident issue; every incident in the last 90 days has a closed follow-up | Watchdog history, incident issues | Decision 0023, observability (PR #924) |
| G7 | Quality and evals | Every `must` requirement is green in production for the whole window; eval coverage at or above 80% of musts with a production-stage eval; no open `FIX` issue in the project; quality findings at or below the ratchet baseline | `mise run evals` results, eval coverage badge, `chores quality` | [evals.md](evals.md), [decision 0015](decisions/0015-code-quality-process.md) |
| G8 | Security and threat | `docs/threat.toml` matches the live surface and the threat check passes; the live check (headers, CSP, TLS) passes; no open Critical or High finding from the last deep review or Dependabot; secrets each have a rotation row | Threat check, live check, Dependabot alerts | [security.md](security.md), [secret-rotation.md](secret-rotation.md) |
| G9 | Data, independence and backups | The product owns its data in tenant-scoped tables and its one-command move-out export works (decision 0021); it has the isolation guide decision 0026 asks for and no runtime request passes through another product's service; every data store has a backups inventory row (BAK-005) and its export passed the backups drill in the last 90 days (BAK-003) | Export run, isolation guide, backups drill result | [decision 0021](decisions/0021-tenant-scoped-product-data.md), decision 0026 (PR #943), decision 0030 backups (`backups/docs/requirements.md`) |
| G10 | User docs | The user guide passes DOC-005 to DOC-008 with no exemption, and covers every main journey in G2 | `chores user-docs` | [documentation.md](documentation.md#user-docs) |
| G11 | Support and feedback | The in-app feedback and support inbox are live on the product's own endpoint; every feedback item in the window became an issue or got an answer; the guide's Getting help section names where support goes | Feedback inbox, the issues it made | [decision 0020](decisions/0020-user-feedback-and-support.md) |
| G12 | Web standard | The WEB rules pass on production: SEO (titles, descriptions, sitemap, canonical), agent readiness and the Lighthouse budgets, accessibility included | `chores site-check` and the Lighthouse run | `web.md`, decision 0028 (PR #988) |
| G13 | Operations and resources | CI and deploy for the project are green on `main`; a rollback has been done or rehearsed; every live resource is claimed in `standards/resources.toml`; a deep review in the last 90 days has no `Poor` in its scorecard | CI history, resources check, the last deck | [resources.md](resources.md), [docs/reviews](../../docs/reviews/README.md) |
| G14 | Name, brand and domain | A domain is chosen and available at a known yearly price; Jim has checked the name (trademark search, no confusing clash); sign-in, if any, still says jimhillr (decision 0027) | The deck's domain slide; Jim's answer | This page, decision 0027 (PR #939) |

Gates G1 to G13 are facts an agent can gather. G4's revenue goal, G3's
usage goal, G5's domain budget and G14's name check are Jim's calls, made
in his approval of the deck.

## The promotion review

A promotion review is an executive review deck that makes the case for one
product, in the same Marp format and folder as the weekly deep reviews
([docs/reviews/README.md](../../docs/reviews/README.md#promotion-reviews)),
written with the
[`deep-project-review`](../../.agents/skills/deep-project-review/SKILL.md)
skill's promotion mode.

**When to open one.** When Jim asks, or when an agent finds every
agent-checkable gate (G1, G2 and G5 to G13) met for a product in
`incubating`. A product that falls short gets no deck; its unmet gates
become ordinary `TODO` or `FIX` issue files in its own `issues/`, the same
loop as evals, so each product works toward promotion through its normal
issues.

**What the deck holds**, in this order:

1. **Title**: product, date, the domain asked for.
2. **The ask**: one slide. The domain, its yearly price, the redirect from
   `<name>.jimhillr.com`, and the decisions Jim is asked to make (usage
   goal, revenue goal, domain budget, name check, any waiver).
3. **Promotion scorecard**: a `| Gate | Domain | Status | Evidence |` table
   with one row per gate above, Status exactly `Met`, `Not met` or
   `Waiver asked`, under a heading that says Promotion scorecard.
4. **One slide per gate** with its evidence: numbers, links, screenshots.
5. **Cost of promotion**: the domain, any new budget, the Claude Code
   effort of the move, and the product's ledger after the move.
6. **Migration plan**: the steps in "Promoting" below, with who does each.
7. **Risks**: what could go wrong on the new domain (SEO loss, broken
   sign-in redirects, email), and the rollback.
8. **Questions for Jim**.

**Approval.** The deck's PR carries the `needs-human` label (a domain is
real money and a name is Jim's to choose) and the `cost` label, so it is in
the human review gate (PR-004) and only a human's approval merges it.
Jim's approval of that PR, and his sign-off on the deck in pane once deck
sign-off exists (PR #926), is the promotion decision. His answers to the
ask go into the deck before merge, so the record says what he set.
Agents never approve a promotion deck and never buy a domain.

## Promoting

After the deck merges:

1. Jim buys or assigns the domain: a human task with a runbook, because it
   needs a payment and a name only he picks. Its price goes into the
   product's `docs/cost.toml` and the ledger as a real cost.
2. An agent points the domain at the same Pages project or Worker and sets
   the permanent redirect from `<name>.jimhillr.com`, as the hosting plan
   describes, through `infra` where a provider covers it (decision 0007).
3. The same PR moves the product's URLs everywhere they are written: the
   deploy URL, `docs/threat.toml`'s surface, evals and the live check, the
   user guide, canonical links and the sitemap, Auth0 callback and logout
   URLs, and the support endpoint.
4. The stage becomes `promoted` in `docs/project.toml`, and any resource
   the move replaces is retired in `standards/resources.toml` (never
   deleted on the spot).
5. After 30 days on the new domain, an agent checks G6, G7 and G12 there
   and files issues for anything that slipped.
