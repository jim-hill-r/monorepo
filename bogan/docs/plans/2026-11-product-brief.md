# Plan: Bogan product brief

Started 2026-11-28 from the agent-generated issue `write-product-brief`
(closed by this plan's PR): Bogan had only its README, so no agent could start
building it.

Status: **proposed**, waiting for Jim's review. Nothing here is built yet.
The choices that are Jim's to make are in
[Decisions for Jim](#decisions-for-jim), each with a recommendation. Once
he approves, the next agent writes `bogan/docs/requirements.md` (prefix
`BOGN`) and breaks the first version into issue files
([brief-approved-requirements-and-issues](../../issues/brief-approved-requirements-and-issues.md)).

It builds on, and doesn't repeat:

- the [README](../../README.md): what Bogan is, its seller rule and its
  name;
- [decision 0004](../../../standards/docs/decisions/0004-database-engine.md)
  (files first, committed SQLite next, D1 when a deployed service must
  write);
- [decision 0021](../../../standards/docs/decisions/0021-tenant-scoped-product-data.md)
  (product runtime data in tenant-scoped D1 tables) and the AGENTS.md rule
  that products are separate companies;
- [decision 0020](../../../standards/docs/decisions/0020-user-feedback-and-support.md)
  (user feedback and support), which gives Bogan its report button;
- the payments question, which stays in its own issue
  ([propose-payments-decision](../../issues/propose-payments-decision.md))
  and is not settled here.

## In one paragraph

Bogan's first version is a **curated catalogue**, not a shop. Each listing
is an item from a small seller or maker, shown with its supply chain: who
made it, what it is made of and where that came from, and every hop it
takes to the buyer, each hop marked with how well it is backed. Buyers
browse and read; to buy, they follow a link to the seller's own shop or
contact page. Listings are TOML files and photos committed to the repo,
reviewed in a pull request before they go live, and built into a static
site on Cloudflare Pages. It costs $0 a month, adds no vendor, and leaves
payments, seller accounts and a hosted store for later decisions.

## Who it's for

- **Sellers.** Makers who make what they sell (a potter, a woodworker, a
  small roaster), small producers (a farm, a mill), and honest resellers
  who can name where their goods come from. What they want: to be found
  for being small and real, and to stand out from relabelled generic goods
  that undercut them on the big marketplaces. What they don't want: fees,
  a new storefront to run, paperwork they can't do from a phone.
- **Buyers.** People who already pay attention to where things come from
  and are tired of not being able to tell a maker from a relabeller. What
  they want: to trust that a listing is what it says, and to see the
  evidence, not just a badge.
- **The curator.** In the first version, Jim (or someone he names) reviews
  each new listing and each report. This is the role that makes the seller
  rule real, so the first version is sized for one person to keep up with.

Not for now: big brands, drop-shippers, anyone who can't name the maker.

## Supply-chain transparency as data

A listing is a claim about an item's history. Bogan stores that claim as a
chain of hops, and every hop says how it is backed.

### The records

| Record | What it holds |
| --- | --- |
| **Seller** | Display name, a short story, region (city or county, never a street address), the link buyers follow to buy, and how to contact them. |
| **Listing** | Title, description, photos, price as text the seller sets (Bogan takes no money in v0), the seller, the maker, the materials and the hops. |
| **Maker** | Who made the item. Usually the seller; otherwise a named person or workshop, never "our factory". |
| **Material** | What the item is made of, and where each main material came from (a place, and the supplier when known). |
| **Hop** | One step on the way to the buyer, in order: the party, their role (grower, miller, maker, finisher, wholesaler, shipper, seller), the place, and what happened there. |
| **Evidence** | Attached to a maker, material or hop: what backs the claim. |

### How a claim is backed

Each maker, material and hop carries one of three levels, shown on the
listing as plain words, not a score:

1. **Stated**: the seller says so and nothing more. Shown, but marked as
   unbacked.
2. **Documented**: the seller attached evidence the curator can see: a
   photo of the work in progress, a supplier invoice with prices blacked
   out, a certificate with a link to its issuer, a public page of the
   supplier.
3. **Checked**: the curator looked at the evidence and confirmed it, for
   example by contacting the supplier or the maker. Records who checked and
   when.

A gap is shown as a gap. A listing whose maker is only stated says so at
the top, so a buyer never has to work out what is missing. Evidence files
that hold private details (invoices, certificates with addresses) stay out
of the public site: the listing shows only that they exist and who checked
them.

### As files

Files before databases, so v0 needs no store at all:

```text
bogan/catalogue/
  sellers/<seller>.toml
  listings/<seller>/<listing>.toml
  photos/<seller>/<listing>-1.webp
```

```toml
# listings/rivertown-pottery/stoneware-mug.toml
title = "Stoneware mug, ash glaze"
seller = "rivertown-pottery"
price = "$32, plus shipping"

[maker]
name = "Ana Ruiz, Rivertown Pottery"
backing = "documented"
evidence = ["photo: throwing the mug on the wheel"]

[[material]]
name = "Stoneware clay"
from = "Sheffield Pottery, Massachusetts"
backing = "stated"

[[hop]]
party = "Rivertown Pottery"
role = "maker"
place = "Galena, Illinois"
what = "Thrown, glazed and fired"
backing = "checked"
checked_by = "jim"
checked_on = "2026-12-02"
```

A small Rust command checks every file in CI (required fields, known
roles, a maker on every listing, hops in order, photo sizes) and builds the
site. The exact schema lands with the requirements, not here.

## The seller rule: no white-labelled or copy-cat listings

The rule, as sellers will read it:

- **No white labels.** Don't list goods someone else made as your own
  brand. If you didn't make it, name who did.
- **No copy-cats.** Don't list a knock-off of another seller's or maker's
  design.

A rule that only lives in the terms isn't a rule, so v0 enforces it in four
layers, cheapest first:

1. **The listing shape.** Every listing must name a maker, and a seller who
   isn't the maker must list the hops from maker to seller. "Generic" or
   "our factory" fails the CI check. Relabelling has to be an outright lie,
   not an omission.
2. **Automatic checks in CI.** A perceptual hash of every photo, compared
   across all listings: the same or a near-same photo under two sellers, or
   a photo that matches one already removed, fails the build and points the
   reviewer at both. It runs in Rust in the existing CI, at no cost.
3. **A person reviews every new listing** before it merges, as a pull
   request: reads the maker and hops, opens the evidence, and runs a manual
   reverse image search with a free web tool on the main photo. Automated
   reverse image search against the whole web needs a paid API, so it is
   left out (see [Not now](#not-now)).
4. **Buyers and sellers report.** Each listing has a "Report this listing"
   link, which goes through Bogan's own support path from decision 0020
   (in v0, until that ships, a prefilled email to Bogan's support address).
   A seller who spots a copy of their design reports it the same way.

Consequences are short and public: a listing that breaks the rule is taken
down, and the seller is told why. A second break removes the seller. Each
removal is a commit, so the history of enforcement is in git.

The rule's honest limit: v0 catches relabelling a curator can see and
copies whose photos match. A seller who lies well about a hop gets
"stated" until someone checks it, and the listing says so. That is the
point of showing backing levels instead of a single verified badge.

## The smallest first version worth shipping

**v0, the catalogue.** Worth shipping because it already does the one thing
no big marketplace does: show a real supply chain and refuse relabelled
goods. It needs no money, accounts or database.

- 5 to 20 sellers Jim invites, 3 to 10 listings each, at most 100 listings.
- Pages: home (recent listings), a listing page (photos, the maker, the
  materials, the hop chain with backing levels, the seller's buy link), a
  seller page, a "how Bogan checks listings" page with the rule, and the
  report link on every listing.
- Search and filters: by category and region, built at build time into a
  small static index. No server.
- How listings get in: the seller sends their details and photos (email or
  a shared folder); an agent turns them into files and opens a PR; the
  curator reviews and merges; the site redeploys. Sellers never touch git.
- Evals: production checks that the home page lists listings, every listing
  page shows a maker and a hop chain, and no listing is missing its buy link.

**v1, seller self-service** (a later epic, only once v0 has real sellers):
a seller submission form whose Worker opens the PR for them, the same
pattern as the cookbook's management app, with seller sign-in through
Auth0 (approved). Listings and photos still land as files and still pass
review.

**Later, each with its own decision first:** taking payment
([propose-payments-decision](../../issues/propose-payments-decision.md)),
photos outside git (R2 is on hold without a decision record), a hosted
store for seller data that changes at runtime (a tenant-scoped D1 table,
decision 0021), and a custom domain.

## How it fits the repo rules

| Rule | How Bogan v0 meets it |
| --- | --- |
| Approved vendors | GitHub (repo, Actions, review) and Cloudflare Pages only. No new vendor. |
| Free tiers only | Every piece is on a free plan; see [Cost and headroom](#cost-and-headroom). |
| Files before databases | Listings, sellers and photo metadata are TOML files; the search index is built at build time. |
| Products are separate companies | Bogan has its own Pages project and origin (`jimhillr-bogan.pages.dev`), owns its catalogue files, and answers its own users. It shares crates and patterns, not a service. |
| Cloudflare Pages names | The project is `jimhillr-bogan` (FT-006). |
| User docs | The guide in `docs/user/` covers browsing and reading a supply chain for buyers, and sending a listing for sellers. |
| Evals | `BOGN` requirements and production evals ship with v0. |
| Threat model | `docs/threat.toml` gains the public site surface. Sellers' personal data is kept to a name, a region and a contact they choose; private evidence stays out of the repo (see below). |
| Cost file | `docs/cost.toml` gains the Pages deploy, at $0. |

**Private evidence.** Invoices and certificates can hold addresses and
prices. They must not be committed, even in a private path, because the
repo is the product's public source. In v0 the curator views them where the
seller sent them and records only "documented, checked by jim on date". If
Bogan later needs to keep them, that is a hosted, private store and a
decision of its own.

## Cost and headroom

All amounts are per month. Limits are from
[`standards/radar/limits.toml`](../../../standards/radar/limits.toml),
checked 2026-11-26.

| Piece | Money | Free limit | v0 use | Headroom |
| --- | --- | --- | --- | --- |
| Cloudflare Pages project | $0 | 100 projects per account (budget 25) | 1 | One more of the 25. |
| Pages files per site | $0 | 20,000 files, each up to 25 MiB | 100 listings, 3 photos each, plus pages: about 700 | About 3% used. |
| Pages deploys | $0 | Direct upload, no Pages builds used | One per merged listing PR | None used. |
| GitHub Actions minutes | $0 | Public repo: no minute limit | A few minutes per listing PR | No limit. |
| Repo size (photos) | $0 | GitHub warns at files over 50 MB and asks to keep repos well under 5 GB | 300 photos at 150 KB or less: about 45 MB | Fine for v0. Photos are the first real limit; past about 1,000 listings they need R2 and a decision. |
| Workers, D1, Auth0 | $0 | Not used in v0 | 0 | All headroom kept for v1. |

**Claude Code build effort**, which counts against the $25 a month budget:
requirements and issue files, one short session; the catalogue schema,
check and site builder, about three sessions; the site pages, evals and
user guide, about three more. Roughly seven sessions for v0, spread over
the normal agent loop. Adding each batch of listings is a short session.

## Not now

- **Checkout and payments**: blocked on the payments decision.
- **Seller accounts and self-service**: v1, after v0 has real sellers.
- **Automated reverse image search**: every web-scale service found costs
  money per search; the CI photo hash and a manual search cover v0.
- **Reviews and ratings**: they invite fake reviews and need moderation;
  the backing levels do the trust work for now.
- **Shipping, tax and returns**: the seller handles them, since the sale
  happens on their side.

## Decisions for Jim

Each option gives its money cost, free-tier use and Claude Code effort.

1. **First version.**
   - **A. Curated catalogue with buy links (recommended).** $0, no new
     vendor, about 3% of Pages' file limit, about seven sessions.
   - B. Catalogue plus seller self-service from day one. $0, uses Workers
     and Auth0 free tiers, about twelve sessions, and more to review before
     any seller has signed up.
   - C. A full marketplace with checkout. Blocked until the payments
     decision; adds a payment vendor and its fees.
2. **Who reviews listings and reports.**
   - **A. Jim as curator for v0 (recommended)**, at about ten minutes a
     listing. $0.
   - B. Any collaborator, with Jim reading the merged list on pane. $0,
     faster, but the rule's judgement calls spread across people.
3. **Where it lives.**
   - **A. `jimhillr-bogan.pages.dev` (recommended).** $0.
   - B. A custom domain. Roughly $12 to $20 a year through Squarespace,
     which breaks the free-only rule unless Jim approves the spend.
4. **The first sellers.** Bogan needs real sellers before v0 is worth
   launching. Who should be invited first, and does Jim want to reach out
   himself or have an agent draft the invitation for him to send? (No
   cost; it is a person's task either way.)
