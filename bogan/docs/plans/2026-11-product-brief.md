# Plan: Bogan product brief

Started 2026-11-28 from `bogan/issues/write-product-brief.md`. Bogan has
only a short README, so no agent can start building it. This brief says
who Bogan is for, what "supply-chain transparency" means as data, how the
no white-label rule is enforced, the smallest first version worth
shipping, and how it fits the repo rules. Every choice that is Jim's is in
[Questions for Jim](#questions-for-jim), each with its cost.

Status: **proposed**, waiting on Jim. Nothing in it is built yet.

It builds on, and doesn't repeat:

- [AGENTS.md](../../../AGENTS.md): approved vendors, free tiers only, files
  before databases, products are separate companies;
- [decision 0004](../../../standards/docs/decisions/0004-database-engine.md)
  (files, then committed SQLite, then D1) and
  [0021](../../../standards/docs/decisions/0021-tenant-scoped-product-data.md)
  (product runtime data in tenant-scoped D1 tables);
- [decision 0020](../../../standards/docs/decisions/0020-user-feedback-and-support.md)
  (feedback and support as platform libraries) and 0018 (a management app
  per product);
- [decision 0016](../../../standards/docs/decisions/0016-holding-and-handling-money.md)
  (where money is held, no card processor yet), which is why payments are
  their own decision (`issues/propose-payments-decision.md`).

## Goal

A buyer who opens a Bogan listing can see who made the item, where its
materials came from, each hop it took to reach them, and how well each of
those claims is backed, and can trust that nothing on Bogan is a relabelled
generic or a knock-off. A small maker can show the work behind their goods
and reach buyers who care about it.

## Who it's for

| Person | What they want from Bogan | What they do |
| --- | --- | --- |
| **Buyer** | Real choice beyond big marketplaces; to know where a thing comes from before buying it. | Browses and searches listings, reads the supply chain, follows the link to buy, reports a listing that looks wrong. |
| **Seller** (a maker, a small workshop, a farm, a reseller of named makers' goods) | To be found for what makes their goods different, without competing with relabelled generics. | Applies, describes each item and its supply chain, backs claims with evidence, keeps listings current. |
| **Curator** (Jim at first, later anyone he names) | To keep the rule and the claims honest. | Reviews applications and listings, checks evidence, handles reports, removes listings and sellers. |

Sellers and buyers are the product's users. The curator works in Bogan's
management app (decision 0018) once it exists; until then the curator's
review is the pull request review of a listing file.

## Supply-chain transparency as data

A listing is an item plus its **chain**: an ordered list of **hops** from
raw material to the buyer. Each hop is one party doing one thing in one
place, and each hop carries the evidence behind it.

```toml
# listings/<seller>/<item>.toml (proposed shape; the schema lands in phase 1)
id = "oak-cutting-board"
seller = "ridge-woodshop"
title = "White oak cutting board"
price = { amount = 85_00, currency = "USD" }   # shown only; no checkout
buy_url = "https://ridgewoodshop.example/boards/oak"
photos = ["oak-board-1.jpg", "oak-board-2.jpg"]

[[hop]]
stage = "material"          # material | component | making | finishing | packing | shipping | selling
what = "Kiln-dried white oak boards"
who = "Hollow Creek Sawmill"
where = "Brown County, Indiana, US"
evidence = "documented"     # declared | documented | verified
proof = ["mill-invoice-2026-10.pdf"]  # redacted copies, committed beside the listing

[[hop]]
stage = "making"
what = "Cut, glued, planed and shaped"
who = "Ridge Woodshop (Sam Ridge)"
where = "Bloomington, Indiana, US"
evidence = "verified"
proof = ["workshop-video.md"]
verified = { by = "curator", on = "2026-12-03", note = "Video call of the workshop and a board in progress" }

[[hop]]
stage = "shipping"
what = "Shipped by the maker"
who = "USPS"
where = "Bloomington, Indiana, US"
evidence = "declared"
```

The three **evidence levels** are the whole trust model, and the page
shows them plainly beside each hop:

| Level | Means | Who sets it |
| --- | --- | --- |
| `declared` | The seller says so. | The seller. |
| `documented` | The seller attached proof (an invoice, a certificate, a photo of a label or a batch), with personal and price details redacted. | The seller; the curator checks the proof matches the claim. |
| `verified` | A curator checked it directly (a video call of the workshop, a visit, a call to the supplier), and records who, when and how. | The curator only. |

From the hops Bogan derives a listing's **transparency summary**: how many
hops there are, how many are documented or verified, and whether the
`making` hop is at least documented. It is a summary, never a single
score, so a buyer sees what is known and what is only claimed. Two rules
apply to every listing:

1. **A named maker.** Every listing has at least one `making` hop with a
   named person or workshop in `who`. "Factory in China" is not a maker;
   a reseller lists the maker it buys from, by name, as that hop.
2. **No hidden hops.** The chain ends with the hop that reaches the buyer
   (`selling` or `shipping`), and a hop the seller doesn't know is written
   as unknown (`who = "unknown"`), which counts against the summary,
   rather than left out.

## Enforcing the no white-label rule

A white-labelled listing is a generic product relabelled as the seller's
own brand; a copy-cat is a knock-off of another maker's design. Both pass
as "made by us", so the rule is enforced at the `making` hop, in layers,
cheapest first:

1. **The schema** (automatic, phase 1). A listing without a named maker,
   or whose chain leaves out the step that reaches the buyer, fails the
   project's CI and can't be published.
2. **Curation before publishing** (Jim, phase 1). Every new seller and every
   new listing is reviewed before it goes live. A first listing from a
   seller needs its `making` hop at least `documented`. The curator's
   checklist lives in `bogan/docs/curation.md`: a quick reverse image
   search of the photos by hand (Google Lens or TinEye in a browser, free),
   the maker's own web presence, and whether the price is plausible for
   hand work.
3. **Duplicate photo check** (automatic, phase 2). The build computes a
   perceptual hash of every listing photo and fails when two listings from
   different sellers share a near-identical photo, which is how resold
   stock photos show up. It runs inside Bogan's own files at build time:
   $0, no vendor.
4. **Buyer reports** (phase 3). Every listing has a "Report this listing"
   link through the platform's support requests (decision 0020), with
   "white-label or copy-cat" as a reason. A reported listing is reviewed
   within a week; while under review it shows a notice.
5. **Strikes.** A listing found to break the rule is removed; a seller with
   two removed listings is removed. Removals are recorded with the reason
   in the listing's git history, never deleted silently.

An automated reverse image search across the web is left out: the APIs
that do it (TinEye, Google Cloud Vision web detection) are paid or need a
billing account, which the free-tier rule rules out.

## The smallest first version: a curated catalog that links out

The first version is a **catalog**, not a store. It has no checkout, no
accounts and no money: each listing links to where the seller already
sells (their own site, Etsy, a farm stand page, an email address), and the
buyer buys there. That gives buyers the transparency, which is the product,
without payments, sales tax, refunds or seller payouts, all of which
need the payments decision first.

What a buyer sees:

- **Home.** What Bogan is, its rule, and the newest listings.
- **Browse.** Listings by category and by region, filterable by evidence
  ("making hop verified").
- **Listing.** Photos, price, the seller, the chain drawn as a short
  timeline with each hop's evidence level and proof, the transparency
  summary, the "Buy from the seller" link and "Report this listing".
- **Seller.** Who they are, where they work, how they were verified, and
  their listings.
- **Sell on Bogan.** The rule, what evidence helps, and how to apply.

What a seller does in the first version: applies by a short form through
the platform's support requests once they exist (decision 0020 phase 5),
and until then by a link to contact Jim. An agent turns an accepted
application into listing files in a pull request; Jim's review of that
pull request is the curation, and merging publishes it. Sellers edit a
listing the same way, by asking. Seller self-service (accounts and a
listing editor) is a later phase that needs sign-in.

Starting size: 5 to 10 sellers Jim invites, around 50 listings. That is
enough to learn whether buyers read the chain before building more.

## How it is built

Bogan follows bear_down's shape, a static site rebuilt from committed
files, so it needs no server and no database:

| Part | What it is | Notes |
| --- | --- | --- |
| `bogan/listings/` | One TOML file per listing and one per seller, with their photos and redacted proof files | Files before databases (AGENTS.md). Photos resized to at most 1600 px and about 300 KB by the build. |
| `bogan/core` | Rust crate: the listing schema, validation (the two chain rules), the transparency summary, the duplicate photo check | Unit tests per rule; `chores`-style checks in CI. |
| `bogan/web` | Dioxus static site rendered at build time | Same stack as bear_down and cookbook; the design system's tokens; Playwright tests. |
| `bogan/cloudflare` | Pages deploy | Pages project `jimhillr-bogan`, created through `free-tiers.sh` (FT-006 naming). |

Search and filters run in the browser over a JSON index the build writes
(a few hundred KB at 50 listings), so there is still no backend. A Worker
and D1 come in only with runtime writes: reports go through the shared
support library (decision 0020), and seller accounts, if Jim wants them
later, would put drafts in `jimhillr-products` under the `bogan` tenant
(decision 0021). Sign-in, when needed, is Auth0, as for cookbook and
cahokia.

## Fit with the repo rules

- **Vendors.** GitHub and Cloudflare only. Buyers leave Bogan to pay, so
  no payment vendor is needed for the first version.
- **Free tiers.** One Pages project of the 25 budgeted (100 free); a
  weekly or on-merge deploy is one direct upload, no Pages builds; 50
  listings with three photos each is about 200 files of the 20,000 per
  site. No Worker or D1 until phase 3.
- **Files before databases.** Listings, sellers, photos and proof are
  committed files; the git history is the audit trail of every claim and
  every curation call.
- **Separate company.** Bogan serves from its own origin, owns its listing
  files, and answers its own sellers and buyers; it uses only shared
  libraries (design system, feedback and support, auth_sdk).
- **Personal data.** Seller and maker names and towns are published on
  purpose, so the application asks for consent to publish each one, and
  proof files are committed only after names of private people, prices
  and account numbers are redacted. A seller can ask for removal, which
  removes the files; they stay in git history, so proof must be redacted
  before it is ever committed. The threat model gets this surface in
  phase 1.
- **Honest claims.** Bogan shows the evidence level of every claim and
  says plainly that a `declared` hop is the seller's word. It never adds
  its own "ethical" or "local" label beyond what the chain shows.

## Cost

| Piece | Money | Free-tier headroom |
| --- | --- | --- |
| Pages project `jimhillr-bogan` | $0 | 1 of 25 budgeted projects; ~200 of 20,000 files |
| GitHub Actions build and deploy | $0 (public repo) | A few minutes a week |
| Duplicate photo check | $0 | Runs in the build |
| Reports through support requests (phase 3) | $0 | Shares the `jimhillr-products` D1 and a product Worker; tens of rows a month |
| Custom domain (optional) | About $20 a year at Squarespace | Only if Jim wants one (question 4) |

Claude Code build effort: about six medium agent PRs for the first version
(phases 1 to 3 below, two PRs each), against the $25 a month budget.

## Phases

Each phase becomes issue files with `epic: Bogan first version` once Jim
approves this brief.

1. **Schema and listings.** `bogan/docs/requirements.md` (prefix `BOGN`),
   `bogan/core` with the schema, the two chain rules and the transparency
   summary, three sample listings that are clearly examples, the curation
   checklist, and the threat model's seller-data surface.
2. **The catalog site.** `bogan/web` and `bogan/cloudflare`: home, browse,
   listing, seller and "Sell on Bogan" pages, the browser search index,
   the duplicate photo check, the user guide in `docs/user/`, Playwright
   tests and production evals.
3. **Reports and first sellers.** "Report this listing" through the
   support library, the seller application form, and the first real
   sellers Jim invites.
4. **Later, each its own decision.** Seller accounts and a listing editor
   (Auth0 plus the `bogan` tenant in D1); the management app's curation
   queue; payments (`issues/propose-payments-decision.md`), which a link-out
   catalog doesn't need until Jim wants Bogan to take orders itself.

## Questions for Jim

1. **First version.** Recommended: **A, a curated catalog that links out**
   to where sellers already sell ($0; one Pages project; about six agent
   PRs). B: seller accounts and self-service listings from the start ($0 on
   Auth0 and D1 free plans, 1 Worker of 25, about ten PRs, and curation
   moves into a management app before launch). C: a full store with
   checkout ($0 to build on the free tiers but card fees of about 2.9% +
   $0.30 per sale, a new payment vendor, and sales tax and payout duties;
   it needs the payments decision first; about fifteen PRs or more).
2. **Who sells at first.** Recommended: **invite-only**, 5 to 10 sellers
   Jim picks ($0; curation is a handful of PR reviews). Alternative: open
   applications from day one ($0, but every application is Jim's time to
   review, and spam needs the support library's rate limits first).
3. **Evidence bar to publish.** Recommended: a seller's first listing
   needs its making hop **documented**, later listings may be declared
   ($0; a few minutes of curation per seller). Alternatives: every making
   hop documented (stricter, slower to grow), or declared is enough
   (fastest, but the rule then rests on reports alone).
4. **Address.** Recommended: **`jimhillr-bogan.pages.dev`** for now ($0).
   Alternative: a custom domain bought at Squarespace (about $20 a year,
   Jim's purchase, a human task with a runbook) once the first sellers are
   live.

## Decisions

None yet. Jim's answers are recorded here, and the phases become issue
files, in the PR that follows his review.
