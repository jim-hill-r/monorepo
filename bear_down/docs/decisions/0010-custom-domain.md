# 0010. Custom domain for the site

- Status: Superseded by repo decision [0031](../../../standards/docs/decisions/0031-jimhillr-com.md)
- Date: 2026-11-28
- Follows: the "custom domain later" consequence of [0009](0009-cloudflare-pages-nightly-cd.md)

## Context

The site is served at <https://jimhillr-bear-down.pages.dev>. Decision 0001
planned to replace that address with a custom domain once one was chosen,
and 0009 kept the option open. Picking one is a decision only Jim can make,
because it depends on which domains he owns and on money:

- The repo has no domain on Cloudflare. Jim's domains are on Squarespace,
  and their names and prices aren't recorded yet
  (`issues/human/list-squarespace-domains.md`).
- Cloudflare Pages lets a project add custom domains on the free plan. A
  subdomain (such as `bears.example.com`) needs only a `CNAME` record to
  `jimhillr-bear-down.pages.dev` at whatever DNS host the domain uses, and
  Cloudflare issues the certificate. An apex domain (`example.com`) needs the
  domain's DNS zone on Cloudflare.
- A new domain is real money outside the free-tier rule, and Cloudflare
  Registrar needs a payment method on the account, which the repo never
  adds ([free-tiers.md](../../../standards/docs/free-tiers.md)).
- Changing the address means updating the deploy URL
  (`bear_down/cloudflare/mise.toml`), the threat model's surface URL, the
  user guide and the evals, and keeping the `pages.dev` address working
  (Pages keeps serving it, so old links don't break).

## Options

| Option | Money | Free tier | Jim's part | Claude Code effort |
| --- | --- | --- | --- | --- |
| A. Subdomain of a domain Jim already owns, by `CNAME` at Squarespace | $0 more (the domain's renewal is already paid) | Pages custom domains are free, far under the per-project limit | Name the domain and subdomain; add one `CNAME` record at Squarespace (about 5 minutes, runbook written with the change) | One small PR: add the domain to the Pages project, update URLs and evals |
| B. Keep `jimhillr-bear-down.pages.dev` | $0 | No change | None | None beyond this record; the issue closes |
| C. Move one Squarespace domain's DNS to a free Cloudflare zone | $0 (the free plan) | Uses one free zone | Change the domain's nameservers at Squarespace, after checking nothing else (email, a Squarespace site) breaks | One PR plus a zone in `infra`; more risk to whatever else uses the domain |
| D. Buy a new domain for Bear Down | About $10 to $20 a year | Outside the free-tier rule; needs a budget from Jim | Buy it (Squarespace, since Cloudflare Registrar needs a card) and set a budget | Same as A |

## Recommendation

Option A: a subdomain of a domain Jim already owns. It costs nothing more,
keeps the free-tier rule, needs no new vendor and no payment method, and
leaves every other use of the domain alone. If no owned domain fits a
football site, option B keeps the site as it is at no cost.

## Decision

Jim (2026-11-28) answered for every site at once: "I have a jimhillr.com
domain where all these pages apps will eventually need to land." So this
is option A on a domain Jim owns, decided repo-wide in
[0031](../../../standards/docs/decisions/0031-jimhillr-com.md): the site
moves to `bear-down.jimhillr.com`
(`bear_down/issues/jimhillr-com-phase-4-move-to-subdomain.md`). Until
then it stays on `jimhillr-bear-down.pages.dev`.

## Consequences

- With A, the follow-up PR adds the custom domain to the `jimhillr-bear-down`
  Pages project, adds a human task with a runbook for the `CNAME` record,
  and moves the deploy URL, threat surface, user guide and evals to the new
  address. `pages.dev` keeps working.
- The custom domain issue is replaced by the jimhillr.com epic's phase 4
  issue for bear_down.
