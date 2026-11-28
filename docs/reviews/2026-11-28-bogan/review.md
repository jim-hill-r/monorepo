# Bogan: executive review, 2026-11-28

Requested by Jim on 2026-11-28: "Make an executive review deck for bogan".
The slides are in [deck.md](deck.md), built from this review. Reviewer:
Claude. Every figure comes from the repo on `main` at `be6078a`
(2026-11-28) or the `evals-results` branch as of 2026-11-28 11:23 UTC.
Guesses are marked as guesses.

## In one paragraph

Bogan is a marketplace for small makers whose difference is supply-chain
transparency: every listing shows who made the item, each hop it took to
the buyer and how well each claim is backed, and no white-labelled or
copy-cat goods are allowed. It has existed as a nine-line README since
2025-12-20, and became buildable only today, when Jim approved the
[product brief](../../../bogan/docs/plans/2026-11-product-brief.md)
(PR #968). The design is strong and cheap: a curated static catalog that
links out to where sellers already sell, $0 a month on GitHub and
Cloudflare Pages, with no checkout, accounts or payments. But nothing is
built, all eight issues are `backlog` so the task picker will not reach
them while other projects have priority work, and the two real long poles
are not code: finding the first 5 to 10 sellers, and the support library
that phase 3's "Report this listing" waits on.

## Purpose and goals

The goal, from the brief: a buyer who opens a listing can see who made it,
where its materials came from, each hop it took and how well each claim is
backed, and can trust nothing is a relabelled generic or a knock-off; a
small maker can show the work behind their goods and reach buyers who care.
Three users: buyers, sellers (makers, workshops, farms, resellers of named
makers) and a curator (Jim at first). Bogan is a `product` tier project
(decision 0019), so it counts toward Jim's 50% product share.

## Where it stands

| Goal | State | Evidence |
| --- | --- | --- |
| Clear goal and design | Done | Brief approved 2026-11-28, all four recommendations |
| Supply chain as data (schema, rules, summary) | Not started | Phase 1 issue, order 20 |
| Requirements and evals | Not started | 0 requirements in `verdicts.json`; phase 1 issue, order 10 |
| Catalog site live | Not started | Phase 2 issues, orders 30 and 40 |
| No white-label enforcement | Designed | Five layers in the brief; none built |
| Buyer reports | Blocked | Needs user feedback phase 5, and phases 1 to 4 before it are open |
| First sellers | Not started | Phase 3 issue, order 60; no sellers named |
| Payments | Deferred on purpose | `propose-payments-decision.md`; v1 links out |

### What is holding it up, ranked

1. **Priority.** All eight issue files are `priority: backlog`. The picker
   (`chores next-task`) takes every `priority` issue first, so Bogan waits
   unless Jim promotes phase 1 or the product share nudge picks it.
2. **Sellers.** The catalog is only worth building if 5 to 10 real sellers
   will fill it. Nobody is named yet, and the human task to invite them is
   planned only in phase 3. This is the slowest step and only Jim can do it.
3. **The support library.** "Report this listing" and the seller form wait
   on decision 0020's phase 5, which sits behind four open platform phases.
4. **Jim's review time.** Phase 1's requirements PR is `needs-human`, and
   in v1 every listing is curated through Jim's PR review.

## Where the code lives

Everything is in `bogan/` today, plus a handful of references elsewhere:

| What | Where |
| --- | --- |
| README, CONTRIBUTING, tasks | `bogan/README.md`, `bogan/CONTRIBUTING.md`, `bogan/mise.toml` (docs-only tasks) |
| Product brief | `bogan/docs/plans/2026-11-product-brief.md` |
| Cost and threat files | `bogan/docs/cost.toml`, `bogan/docs/threat.toml` |
| Work items | `bogan/issues/` (8 files) |
| Config root | `mise.toml:24` |
| Requirement prefix `BOGN` | `standards/docs/naming.md:49` |
| Product tier | `standards/docs/decisions/0019-review-roles.md:105`, `chores/src/tiers.rs:31` |
| User docs baseline | `standards/user-docs.toml:15` |
| Eval results and badge | `evals-results` branch: `verdicts.json`, `badges/bogan.svg` |

Planned by the brief: `bogan/listings/`, `bogan/core`, `bogan/web`,
`bogan/cloudflare`. It will stay one project; it shares only libraries
(design system, support, `auth_sdk`).

## Dependencies and vendors

Today: none. The project is Markdown and TOML, and its mise tasks come from
`standards/tasks/docs.toml`, which needs no tools.

Planned (from the brief), all already in use elsewhere in the repo:

| Dependency | Version elsewhere | For |
| --- | --- | --- |
| Rust | repo toolchain | `bogan/core` schema, rules, photo checks |
| Dioxus | 0.7.2 (bear_down, cookbook) | static catalog site |
| design_system_dioxus | path dep | tokens and components |
| Playwright | as cookbook | site tests and demos |
| A perceptual hash crate | not chosen | duplicate photo check (phase 2) |

| Vendor | Plan | Cost a month | Billing check | Approved |
| --- | --- | --- | --- | --- |
| None today | | $0 | | |
| GitHub (planned) | Free, public repo | $0 | billing-check.sh | Yes |
| Cloudflare Pages (planned) | Free | $0 | billing-check.sh | Yes |
| Cloudflare Workers + D1 (phase 3) | Free | $0 | billing-check.sh | Yes |
| Auth0 (later, only with seller accounts) | Free | $0 | settings check | Yes (exception) |
| Squarespace domain (optional) | about $20 a year | about $1.67 | settings check | Yes |

## Security and personal data

Nothing runs, so there is nothing to attack today; `docs/threat.toml` is a
placeholder that says to write the real model before the first deploy. The
brief gets the hard part right: the repo is public and git history keeps
everything, so proof files (invoices, certificates) are never committed and
listings record only what was seen, by whom, on which date. Seller and
maker names and towns are published on purpose, with consent, and a
removal request deletes files from `main` but not from history, which the
application must say. Findings: Medium, the threat model has no seller-data
surface yet (phase 1 adds it); Low, a seller's link-out URL is
seller-controlled and could later point somewhere harmful, so curation
should check it and the site should mark outbound links.

## Quality, operations and cost

`mise run ci` for a docs-only project prints a line and checks nothing
(root issue `issues/docs-only-ci-checks-nothing.md`). Eval coverage is 0
requirements. No user guide (in the DOC-008 baseline). Nothing deploys.
Cost is $0 a month now and stays $0 through phase 3; Claude Code effort is
about six medium PRs, against the $25 a month budget.

## Tech selection against the radar

Every planned choice is on the radar and fits it: Rust, Dioxus, Playwright,
GitHub, Cloudflare Pages and Auth0 are Adopt; Workers and D1 are Trial,
used for what the radar trials them for (runtime writes); R2 is Hold, and
the brief rightly turns photo storage past about 1,000 listings into its own
decision. No radar change now: Bogan should become an exemplar only once it
ships code.

## Risks

- **Empty shelves.** A transparency catalog with few sellers proves
  nothing. Seller supply, not code, decides whether v1 teaches anything.
- **The name.** "Bogan" is common slang in Australia and New Zealand for an
  unrefined person. That is a guess at how it lands with buyers there, but
  worth a check before any domain or brand spend.
- **Curator load.** All curation is Jim's PR review until the management
  app exists.
- **Repo size.** Photos in git cap the catalog at about 1,000 listings.
- **Trust in "documented".** The level rests on one curator's look at proof
  that is never kept; a later dispute has only the recorded note.

## Recommendations

| # | Do this | Impact | Effort | Moves |
| --- | --- | --- | --- | --- |
| 1 | Promote phase 1 (requirements, core schema) to `priority` | High | S | Progress |
| 2 | Start the seller invitations now, not in phase 3 | High | S (Jim's time) | Goals |
| 3 | Launch reports and applications with a contact link; move to the support library when it ships | High | S | Progress |
| 4 | Check the name before buying a domain | Medium | S | Brand |
| 5 | Mark outbound buy links and check them in curation | Low | S | Security |

Every recommendation either needs Jim's answer first (1 to 4) or fits an
existing issue (5 belongs in phase 1's `bogan/docs/curation.md`), so this
review adds no new issue files.

## Questions for Jim

1. Promote Bogan's phase 1 issues to `priority`? ($0; about two medium
   Claude Code PRs now instead of later.)
2. Do you have sellers in mind, and should the invite task start now?
   ($0; your time.)
3. Ship "Report this listing" and the seller form as a contact link in
   phase 3 rather than wait for the support library? ($0; saves waiting on
   four platform phases; one small PR to switch later.)
4. Keep the name Bogan, knowing the Australian slang meaning? ($0.)

## Method

Read `bogan/` in full, PR #968's brief and Jim's answers, the eight issue
files, decisions 0004, 0016, 0018 to 0021, the tech radar and limits, the
`evals-results` verdicts, and `git log -- bogan`. There is no app to run,
so there are no screenshots and no demo videos; the deck says so.
