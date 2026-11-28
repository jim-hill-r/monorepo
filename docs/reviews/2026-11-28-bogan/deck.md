---
marp: true
theme: default
paginate: true
size: 16:9
title: bogan executive review, 2026-11-28
style: |
  section { font-size: 22px; }
  table { font-size: 18px; }
  section.lead h1 { font-size: 52px; }
  code { font-size: 18px; }
  pre { font-size: 16px; }
---

<!-- _class: lead -->

# Bogan
## Executive review, 2026-11-28

Reviewer: Claude, at Jim's request ("Make an executive review deck for bogan")

The full written review is [review.md](review.md). Figures are from `main`
at `be6078a` and the `evals-results` branch as of 2026-11-28 11:23 UTC.

---

# Executive summary

**What it is:** a marketplace for small makers where every listing shows who
made the item, each hop it took to the buyer, and how well each claim is
backed. No white-labelled or copy-cat goods.

**Where it stands:** designed, not built. The product brief was approved
today (PR #968); there is no code, no site and no requirements yet.

**The three biggest blockers:**
1. All eight issues are `backlog`, so the task picker won't reach them.
2. No sellers are lined up; only Jim can find the first 5 to 10.
3. Buyer reports wait on the platform support library (four phases away).

---

# Scorecard

| Area | Rating | Why |
| --- | --- | --- |
| Goals | Good | Clear goal and users, brief approved with all four recommendations |
| Design | Good | Supply chain as data, three evidence levels, five enforcement layers |
| Progress | Poor | Nothing built; every issue is backlog |
| UX | Fair | Five pages designed in the brief; nothing to use yet |
| Security | Fair | Strong rule that proof never enters the public repo; threat model still a placeholder |
| Quality | Poor | CI checks nothing, 0 requirements, no user guide |
| Operations | Fair | Deploy path designed (Pages, direct upload); nothing deployed |
| Cost | Good | $0 a month through phase 3, approved vendors only |
| Tech selection | Good | Every planned choice is Adopt or Trial on the radar and fits it |

---

# Top recommendations

| # | Do this | Effort |
| --- | --- | --- |
| 1 | Promote phase 1 (requirements, core schema) to `priority` | S |
| 2 | Start inviting sellers now, not in phase 3 | S (Jim's time) |
| 3 | Ship reports and the seller form as a contact link; move to the support library later | S |
| 4 | Check the name before any domain or brand spend | S |
| 5 | Mark outbound buy links and check them during curation | S |

---

<!-- _class: lead -->

# Purpose and long-term goals

---

# Who it's for and why

| Person | Wants | Does |
| --- | --- | --- |
| **Buyer** | Real choice beyond big marketplaces; to know where a thing comes from | Browses, reads the chain, follows the link to buy, reports a bad listing |
| **Seller** (maker, workshop, farm, reseller of named makers) | To be found for what makes their goods different | Applies, describes each item and its chain, backs claims with evidence |
| **Curator** (Jim at first) | To keep the rule and the claims honest | Reviews sellers and listings, checks proof, handles reports |

**Goal** (brief): a buyer can see who made an item, where it came from,
each hop and how well each claim is backed, and trust nothing is a
relabelled generic or a knock-off.

Bogan is a `product` tier project (decision 0019), so its work counts toward
the 50% product share.

---

# The idea in one picture

```text
 material ──► making ──► finishing ──► shipping ──► buyer
 Hollow Creek  Ridge        (none)        USPS
 Sawmill       Woodshop
 documented    verified                   declared
   │              │                          │
   │              └─ curator video call      └─ seller's word
   └─ mill invoice seen, never stored
```

Three evidence levels are the whole trust model: **declared** (the seller
says so), **documented** (the curator saw proof), **verified** (the curator
checked directly). The page shows a summary, never a single score.

---

# History

| When | What |
| --- | --- |
| 2025-12-20 | Nine-line README: "an amazon marketplace clone" focused on transparency |
| 2026-11-25 to 27 | Repo-wide chores: mise tasks, cost and threat files, eval badge |
| 2026-11-27 | First issues recorded (PR #791); README rewritten (PR #831) |
| 2026-11-28 | Product brief approved by Jim; six first-version issues (PR #968) |

Eleven months as an idea, one day as a plan.

---

<!-- _class: lead -->

# Progress against goals

---

# Where each goal stands

| Goal | State | Evidence |
| --- | --- | --- |
| Clear goal and design | Done | Brief approved 2026-11-28 |
| Schema, chain rules, summary | Not started | Phase 1 issue, order 20 |
| Requirements and evals | Not started | 0 requirements in `verdicts.json` |
| Catalog site live | Not started | Phase 2 issues, orders 30 and 40 |
| No white-label enforcement | Designed | Five layers in the brief, none built |
| Buyer reports | Blocked | Needs user feedback phase 5 |
| First sellers | Not started | Phase 3 issue, order 60 |
| Payments | Deferred on purpose | v1 links out; decision issue open |

---

# What is holding it up, ranked

1. **Priority.** Every issue is `priority: backlog`. `chores next-task`
   takes all `priority` issues first, so Bogan waits unless Jim promotes it
   or the product share nudge happens to pick it.
2. **Sellers.** A catalog teaches nothing with empty shelves. No sellers
   are named, and the invite task is planned only for phase 3.
3. **The support library.** "Report this listing" and the seller form wait
   on decision 0020's phase 5; phases 1 to 4 before it are all open
   (`issues/user-feedback-phase-*.md`).
4. **Jim's review time.** The phase 1 requirements PR is `needs-human`, and
   every v1 listing is curated through Jim's PR review.

---

<!-- _class: lead -->

# Features

---

# Features (all planned)

| Feature | Phase | Status |
| --- | --- | --- |
| Listing and seller schema, two chain rules, transparency summary | 1 | Not started |
| Three sample listings, curator's checklist | 1 | Not started |
| Home, Browse (category, region, evidence filter), Listing, Seller, Sell on Bogan | 2 | Not started |
| Browser-side search over a JSON index | 2 | Not started |
| Photo resize and duplicate photo check (perceptual hash) | 2 | Not started |
| Report this listing, with a notice while under review | 3 | Blocked on support library |
| Seller application with consent to publish | 3 | Not started |
| Payments, seller accounts, photo store, curation queue | 4 | Each its own later decision |

---

# Feature demos

**None yet.** Bogan has nothing live to show, so this deck has no demo
videos (decision 0025 skips projects with nothing to demo).

The first demos come with phase 2, once `jimhillr-bogan.pages.dev` is up:

1. A buyer browses, filters to "making hop verified" and reads a chain.
2. A buyer follows "Buy from the seller" and reports a listing.

Screenshots and user experience findings wait for the same phase.

---

<!-- _class: lead -->

# Architecture

---

# How it will be built

```text
bogan/listings/*.toml + photos ──► bogan/core (Rust)
  one file per listing and seller     schema, chain rules, summary,
  proof files never committed         photo resize + duplicate check
                                            │  fails CI on a bad listing
                                            ▼
                                     bogan/web (Dioxus, static)
                                       pages + JSON search index
                                            │  direct upload
                                            ▼
                               Cloudflare Pages: jimhillr-bogan.pages.dev
                                            │  buyer clicks "Buy"
                                            ▼
                               the seller's own shop (Bogan takes no money)
```

The same shape as bear_down: no server, no database. A Worker and D1 come
in only with reports (phase 3), in the `bogan` tenant of
`jimhillr-products` (decision 0021).

---

# Where the code lives

| What | Where |
| --- | --- |
| README, CONTRIBUTING, docs-only tasks | `bogan/README.md`, `bogan/CONTRIBUTING.md`, `bogan/mise.toml` |
| Product brief | `bogan/docs/plans/2026-11-product-brief.md` |
| Cost and threat files | `bogan/docs/cost.toml`, `bogan/docs/threat.toml` |
| Work items (8) | `bogan/issues/` |
| Config root | `mise.toml:24` |
| Requirement prefix `BOGN` | `standards/docs/naming.md:49` |
| Product tier | `standards/docs/decisions/0019-review-roles.md:105`, `chores/src/tiers.rs:31` |
| User docs baseline | `standards/user-docs.toml:15` |
| Eval results and badge | `evals-results`: `verdicts.json`, `badges/bogan.svg` |

One project, all in `bogan/`; planned `listings/`, `core/`, `web/`, `cloudflare/`.

---

# Dependencies

**Today: none.** Markdown and TOML only; its tasks come from
`standards/tasks/docs.toml`, which needs no tools.

**Planned**, all already used elsewhere in the repo:

| Dependency | Version | Used for |
| --- | --- | --- |
| Rust | repo toolchain | `bogan/core`: schema, rules, photo checks |
| Dioxus | 0.7.2 (as bear_down, cookbook) | the static catalog site |
| design_system_dioxus | path dependency | tokens and components |
| Playwright | as cookbook | site tests, evals and future demos |
| A perceptual hash crate | not chosen yet | duplicate photo check (phase 2) |

---

# Vendors

| Vendor | Plan | Cost a month | Billing check | Approved |
| --- | --- | --- | --- | --- |
| None today | | $0 | | |
| GitHub (planned) | Free, public repo | $0 | `billing-check.sh` | Yes |
| Cloudflare Pages (planned) | Free | $0 | `billing-check.sh` | Yes |
| Cloudflare Workers + D1 (phase 3) | Free | $0 | `billing-check.sh` | Yes |
| Auth0 (only with seller accounts) | Free | $0 | Settings check | Yes, exception |
| Squarespace domain (optional) | Paid | about $1.67 ($20 a year) | Settings check | Yes |

No payment vendor: buyers pay sellers directly. Automated reverse image
search (TinEye, Google Vision) is left out because it is paid.

---

<!-- _class: lead -->

# Security, quality and operations

---

# Security and personal data

**Today:** nothing runs. `docs/threat.toml` is a placeholder: "Write the real
model before the first deploy or sign-in."

**What the brief gets right:** the repo is public and git keeps everything,
so proof (invoices, certificates) is never committed. Listings record only
what was seen, who checked it and when. Names and towns are published with
consent; removal deletes from `main`, not from history, and the form says so.

| Finding | Severity | Why |
| --- | --- | --- |
| No seller-data surface in the threat model | Medium | Phase 1 adds it; must land before real sellers |
| Buy links are seller-controlled | Low | A link could later point somewhere harmful; check it in curation, mark outbound links |
| "Documented" rests on proof nobody keeps | Low | A dispute has only the curator's note |

---

# Quality, operations and cost

| Area | Today |
| --- | --- |
| CI | `mise run ci` prints a line and checks nothing (root issue `docs-only-ci-checks-nothing.md`) |
| Requirements and evals | 0 requirements; badge shows no coverage |
| User guide | None; in the DOC-008 baseline (`write-user-guide.md`) |
| Deploy | None; phase 2 adds Pages by direct upload, no Pages builds |
| Observability | None planned yet; bear_down's pattern applies |
| Money | $0 a month now and through phase 3 |
| Free-tier use | 1 of 25 budgeted Pages projects; about 200 of 20,000 files at 50 listings |
| Claude Code effort | About six medium PRs for v1, against $25 a month |

---

# Tech selection against the radar

| Technology | Ring | Fits? | Note |
| --- | --- | --- | --- |
| Rust, Dioxus, Playwright | Adopt | Yes | Same stack as bear_down and cookbook |
| GitHub, Cloudflare Pages | Adopt | Yes | Approved vendors |
| Cloudflare Workers, D1 | Trial | Yes | Only for runtime writes, as the radar trials them |
| Cloudflare R2 | Hold | Yes | Brief defers photo storage to its own decision past ~1,000 listings |
| Auth0 | Adopt | Yes | Only if seller accounts come later |
| Squarespace | Adopt | Yes | Optional domain |

**No radar change.** Bogan should become an exemplar only once it ships code.

---

# Risks and technical debt

- **Empty shelves.** Seller supply, not code, decides whether v1 teaches
  anything about whether buyers read the chain.
- **The name.** "Bogan" is common Australian and New Zealand slang for an
  unrefined person. How it lands with buyers there is a guess; worth a
  check before a domain or brand spend.
- **Curator load.** Every listing is Jim's PR review until the management
  app exists.
- **Repo size.** Photos in git cap the catalog near 1,000 listings.
- **No debt yet**, because there is no code. CI that checks nothing is the
  one gap that already exists.

---

<!-- _class: lead -->

# Recommendations and decisions

---

# Recommendations

| # | Do this | Impact | Effort | Moves |
| --- | --- | --- | --- | --- |
| 1 | Promote phase 1 (requirements, core schema) to `priority` | High | S | Progress |
| 2 | Start seller invitations now, in parallel with phases 1 and 2 | High | S (Jim) | Goals |
| 3 | Ship reports and applications as a contact link; switch to the support library when it ships | High | S | Progress |
| 4 | Check the name before buying a domain | Medium | S | Brand |
| 5 | Mark outbound buy links; check them in `curation.md` | Low | S | Security |

No new issue files: 1 to 4 need Jim's answer first, and 5 fits phase 1's
curation checklist.

---

# Questions for Jim

| # | Question | Cost |
| --- | --- | --- |
| 1 | Promote Bogan's phase 1 issues to `priority`? | $0; about two medium Claude Code PRs now instead of later |
| 2 | Do you have sellers in mind, and should the invite task start now? | $0; your time |
| 3 | Launch reports and the seller form as a contact link rather than wait for the support library? | $0; one small PR to switch later |
| 4 | Keep the name Bogan, knowing the Australian slang meaning? | $0 |

Please answer as review comments on this deck's pull request.

---

# Appendix: method and evidence

- Read all of `bogan/` on `main` at `be6078a`, including the brief and its
  "Decisions" section with Jim's answers (PR #968).
- `git log -- bogan` for history (first commit `781ffb6`, 2025-12-20).
- Issue priorities: `grep priority bogan/issues/*.md` (all `backlog`);
  picker order in `chores/src/next_task.rs:7`.
- Support library status: `issues/user-feedback-phase-1` to `-5` all open.
- Eval coverage: `evals-results` `verdicts.json`, bogan has 0 requirements.
- Radar rings from `standards/radar/radar.json`; Pages budget from
  `standards/radar/limits.toml` (`pages_projects`, budget 25).
- No app to run, so no screenshots, demos or performance measurements.
- No product code changed in this review.
