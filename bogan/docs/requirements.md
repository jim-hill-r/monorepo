# Bogan requirements

What Bogan promises its buyers and sellers, in the format of
[standards/docs/evals.md](../../standards/docs/evals.md). Drafted
2026-11-28 from the [product brief](plans/2026-11-product-brief.md), which
Jim approved on 2026-11-28 with every recommended answer: the first
version is an invite-only, curated catalog that links out to where sellers
already sell, with no checkout, accounts or payments. Each requirement's
evals are in [evals.toml](../evals.toml); the tests and pages they read
arrive with the brief's phases (`bogan/core` in phase 1, the catalog site
in phase 2, reports in phase 3).

Words used below, from the brief: a **listing** is an item plus its
**chain**, an ordered list of **hops** (one party doing one thing in one
place); each hop has an **evidence level**: `declared` (the seller says
so), `documented` (a curator checked proof the seller showed) or
`verified` (a curator checked it directly).

## BOGN-001: Every listing names its maker

- Priority: must
- Why: Buyers come to Bogan to know who made a thing; a listing without a
  maker is how a relabelled generic passes as hand work.
- Accept: a listing without at least one `making` hop whose `who` names a
  person or workshop fails `bogan/core` validation and the project's CI,
  so it can't be published; `unknown`, a blank name or a bare place
  ("factory in China") is rejected for a `making` hop; a reseller's
  listing names the maker it buys from; the listing page shows the maker's
  name.

## BOGN-002: A listing has no hidden hops

- Priority: must
- Why: A chain that stops early or skips a step tells the buyer less than
  it seems to, and the gap is where a claim can't be checked.
- Accept: a chain whose last hop is not the one that reaches the buyer
  (`selling` or `shipping`) fails validation; a hop the seller doesn't
  know is written with `who = "unknown"` and shown as unknown on the page,
  never left out; every hop in the listing file appears on the listing
  page, in order.

## BOGN-003: Every hop shows its evidence level

- Priority: must
- Why: The evidence level is the whole trust model; a buyer must see how
  well each claim is backed, not only the claim.
- Accept: every hop has exactly one of `declared`, `documented` or
  `verified`, or fails validation; a `documented` hop records what proof
  was seen, who checked it and on which date, and a `verified` hop records
  which curator checked it, when and how, or fails validation; only the
  curator can set `documented` or `verified`; the listing page shows each
  hop's level beside it, with what backs it, and says plainly that a
  `declared` hop is the seller's word.

## BOGN-004: The transparency summary never hides what is only claimed

- Priority: must
- Why: A single score or a friendly label would let a mostly declared
  chain look as good as a verified one.
- Accept: each listing's summary shows how many hops it has, how many are
  documented or verified, how many are unknown, and whether its `making`
  hop is at least documented; it is never a single score or grade; Bogan
  adds no label of its own ("ethical", "local", "sustainable") beyond what
  the chain shows.

## BOGN-005: No white-label or copy-cat listings

- Priority: must
- Why: This is the seller rule that sets Bogan apart from big
  marketplaces: makers shouldn't compete with relabelled generics or
  knock-offs of their own designs.
- Accept: a seller's first listing is published only with its `making`
  hop at least `documented` (later listings may be `declared`); the build
  fails when two listings from different sellers share a near-identical
  photo (a perceptual hash match); a listing found to break the rule is
  removed with the reason in its git history, and a seller with two
  removed listings is removed.

## BOGN-006: Buyers buy through the seller's own link

- Priority: must
- Why: The first version is a catalog, not a store: with no checkout
  Bogan needs no payment vendor, sales tax or payouts, and buyers deal
  with the seller directly.
- Accept: every listing has a "Buy from the seller" link to a page the
  seller controls (their own site, a shop on another marketplace, a farm
  stand page or an email address), and a listing without one fails
  validation; Bogan has no cart or checkout, takes no payment and stores
  no buyer account, address or card.

## BOGN-007: Proof never enters the repo

- Priority: must
- Why: The repo is public and git keeps everything, so an invoice or
  certificate committed once, or a redaction mistake in one, would be
  published for good.
- Accept: no invoice, certificate or other proof file is committed: CI
  fails when a listing folder holds any file other than listing and seller
  TOML files and the photos a listing names; a hop records only a short
  description of what proof was seen, who checked it and on which date,
  and the listing page shows that description, never a proof file.

## BOGN-008: Bogan runs at $0 a month

- Priority: must
- Why: Every service is used on its free plan (AGENTS.md); a catalog of
  committed files on one Pages project needs nothing paid.
- Accept: `bogan/docs/cost.toml` expects $0 a month and `cost check`
  passes for it; Bogan uses only approved vendors (GitHub and Cloudflare)
  on their free plans, with no Worker or D1 until reports need one.

## BOGN-009: A buyer can report a listing

- Priority: should
- Why: Curation can miss a white-label or copy-cat listing; buyers who
  know a product are the last check.
- Accept: every listing page has a "Report this listing" link with
  "white-label or copy-cat" among its reasons; a reported listing is
  reviewed within a week and shows a notice while it is under review.

## BOGN-010: Buyers can filter by evidence

- Priority: should
- Why: Buyers who care most about provenance want to see only the
  listings whose claims are backed.
- Accept: browse can be filtered by category, by region and by evidence
  ("making hop verified" or "documented"), in the browser with no backend,
  and the filtered list updates within 1 s of a change at 50 listings.

## BOGN-011: Sellers publish their names by consent and can be removed

- Priority: should
- Why: Seller and maker names and towns are published on purpose, so
  each seller must agree to it and be able to leave.
- Accept: a seller's application asks for consent to publish each name
  and place shown; a seller who asks for removal has their listing and
  seller files deleted from `main` within a week, and the application says
  that names stay in git history.
