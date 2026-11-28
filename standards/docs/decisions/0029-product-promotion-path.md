# 0029. Every project starts on a jimhillr.com subdomain, and a product earns its own domain through a promotion review

- Status: Accepted (the rule, Jim 2026-11-28); the default goals in G3 and
  G4 are proposed and open to Jim's review in the PR
- Date: 2026-11-28
- Asked by: Jim
- Standard: [product-lifecycle.md](../product-lifecycle.md)
- Builds on: [0012](0012-revenue-and-http-402.md) (revenue),
  [0019](0019-review-roles.md) (tiers and review roles),
  [0022](0022-project-metadata-and-product-docs.md) (`docs/project.toml`),
  the weekly deep reviews ([docs/reviews](../../../docs/reviews/README.md)),
  and every standard each gate points at
- Works with: the jimhillr.com hosting decision and
  [plan](../plans/2026-11-jimhillr-com.md) (PR #961), which say how a
  project lands on its subdomain and how a promoted domain is pointed at it

## Context

Jim (2026-11-28): "All projects will start out as a subdomain of
jimhillr.com with a promotion path to a fully fledged product of its own
with its own domain. We need to establish that path in the product
lifecycle and what would allow a promotion such as revenue goals, cost,
stability, quality etc... each domain of concern should be include in the
promotion path. An executive review deck needs to be made making the case
for promotion and reviewed by jim."

Until now the repo had no product lifecycle: each project picked its own
address (`jimhillr-<project>.pages.dev`, Blue Eel's own domain), and a
custom domain was a one-off question per project (bear_down decision
0010). Every domain of concern already has its own standard and checks
(evals, cost, free tiers, threat, quality, user docs, feedback, web,
observability, backups), but nothing tied them to a product's growth.

A domain costs real money (about $10 to $20 a year), which sits outside
the free-tier rule and needs a budget only Jim sets. And
"products are separate companies" (decision 0021) is most true for a
product on its own domain.

## Decision

1. **Three stages**: `incubating` on `<name>.jimhillr.com`, `candidate`
   while a promotion review is open, `promoted` on its own domain, with the
   subdomain redirecting there forever. The stage will be a `stage` field
   in `docs/project.toml`.
2. **Only `product` tier projects are promoted.** Platform projects stay
   on a subdomain or have no site.
3. **Fourteen gates, one per domain of concern** (mission, requirements,
   traction, revenue, cost, stability, quality, security, data and
   backups, user docs, support, web, operations, name and domain), each
   pointing at the standard that owns it, all in
   [product-lifecycle.md](../product-lifecycle.md#promotion-gates). Only
   Jim can waive one, in writing in the deck.
4. **The case is an executive review deck** in `docs/reviews/`, in the
   deep review format with a Promotion scorecard, on a PR labelled
   `needs-human` and `cost`. Jim's approval of that PR is the promotion
   decision. Agents never approve one and never buy a domain.
5. **Unmet gates are issues.** A product short of a gate gets an issue
   file for it, so products grow toward promotion through the normal agent
   loop, as with evals.
6. **Default goals**, which Jim can change per product in the deck:
   traction is 50 distinct weekly visitors or users for 8 weeks; revenue is
   self-funding (own revenue at least own money cost, domain included) for
   3 months.

## Options considered for the revenue gate

| Option | Money cost | Build effort | For | Against |
| --- | --- | --- | --- | --- |
| Self-funding for 3 months, plus any goal Jim sets (chosen as default) | $0 to check; the domain itself is about $10 to $20 a year | Small: a ledger query per product | Promotion never adds a cost nobody pays; works with the ledger we have | Products with no paid feature can't pass until one exists |
| Only a goal Jim sets per product | $0 | None | Most flexible | No standing bar; every deck is a negotiation |
| No revenue gate, traction only | $0 | None | Lets free products promote | A domain becomes a cost with no funding, against the free-tier rule |

## Cost at scale

Projects now: 27 (`ls */docs/cost.toml | wc -l`, 2026-11-28). The
worst case below assumes every project is promoted, though only `product`
tier projects can be. The rule and its checks cost nothing to run; the
only money is each promoted product's domain, about $15 a year each
(about $1.25 a month), and gate G4 makes each product's own revenue cover
it before it is bought, so the repo's net cost stays $0 at every scale.

| Scale | Projects | Money a month (gross, all promoted) | Tightest free limit | Claude Code effort |
| --- | --- | --- | --- | --- |
| Now | 27 | $0 today (none promoted); up to about $34 if all were, paid by their own revenue | None binds: Pages allows 100 custom domains per project and each uses one; the checks read files in the audit, seconds of Actions time | 2 small PRs once, then about $1 per promotion deck |
| 5x | 135 | Up to about $170, paid by the products' revenue | None binds: one custom domain per Pages project; a Cloudflare zone, if used, is one free zone per domain | Same build; about $1 per deck, so a few dollars a month if several promote |
| 100x | 2,700 | Up to about $3,400, paid by the products' revenue | Jim's review time binds first, not a free limit: one deck each; Cloudflare's free account zone limit would need checking before moving thousands of domains | Decks alone could exceed the $25 a month budget, so promotions would be batched or paced |

What gives first as it grows: Jim's time to review decks, then the
Claude Code budget for writing them. Then we would batch promotions into
one deck per review and let `chores promotion` do the gathering.

## Consequences

- Cost: $0 for the rule and its checks; each promotion adds one domain
  (about $10 to $20 a year) that Jim budgets in his approval. Writing the
  checks is a few small agent PRs against the Claude Code budget.
- Follow-up issues: a `stage` field and a `chores promotion` report of the
  agent-checkable gates (after decision 0022 phase 1), and the promotion
  mode in the deep review skill (in this PR).
- Projects already on `pages.dev` move to their `jimhillr.com` subdomain
  through the hosting decision's plan, not through this one.
- Blue Eel already has its own domain; its first promotion review records
  where it stands against the gates rather than moving anything.
