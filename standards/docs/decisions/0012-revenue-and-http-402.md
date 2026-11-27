# 0012. Revenue: open books and GitHub Sponsors first, HTTP 402 (x402) staged

- Status: Proposed
- Decided by: waiting for Jim
- Date: 2026-11-27
- Asked by: Jim
- Plan: [2026-11-revenue-402.md](../plans/2026-11-revenue-402.md)
- Builds on: the [cost check](../plans/2026-11-cost-check.md),
  [cost tracking](../plans/2026-11-cost-tracking.md) (PR #756), the billing
  checks (PR #760), [0010](0010-credential-second-approval.md) and
  [0011](0011-mint-secrets-by-workflow.md)

## Context

Jim (2026-11-27) wants to monetize our APIs with HTTP 402, have our own UIs
and CLIs pay for API use like any customer out of free cash flow, show a
donation splash page when free cash flow runs out, and show cost and
revenue to users.

Research on 2026-11-27 (details and sources in the plan):

- x402 (version 2, run by the x402 Foundation, founded by Coinbase and
  Cloudflare) is the main 402 protocol; MPP (Stripe and Tempo) is the other.
- Cloudflare's managed **Monetization Gateway** needs a credit card on file
  and a Cloudflare zone, which breaks the no-payment-method rule. Its open
  Worker middleware and templates do fit Workers Free.
- The Coinbase CDP facilitator settles 1,000 x402 payments a month free,
  then $0.001 each, pays gas, and needs only an API key and a public
  receiving address on our side.
- We have no API that outsiders pay for today, and x402's real commerce is
  small and mostly bots. GitHub Sponsors takes 0% from personal sponsors
  and GitHub is an approved vendor.
- Our only money cost is Claude Code ($25 a month), so stopping apps saves
  nothing; pausing agent work does.

## Decision (proposed)

Architecture E from the plan, with the recommended answer to each of the
plan's eight questions unless Jim says otherwise:

1. Phases 1 to 3 ($0, no new vendor): the books and runway as files,
   GitHub Sponsors with a shared banner and splash, metering with shadow
   billing of first-party use, and a public Open books page.
2. Phase 4 ($0): an x402 pilot on testnet with the x402.org facilitator.
3. Phase 5 (gated): x402 on mainnet through Coinbase CDP only after an API
   has outside demand, Jim approves CDP as a vendor (with no card on file),
   a wallet exists whose key never enters our systems, and a tax
   professional has been consulted.
4. Free cash flow counts Claude Code; Jim's own payments are owner
   contributions. The cutoff is soft for users and hard for new agent work.

## Consequences

- No money, card, domain or paid plan in any phase. Monetization Gateway
  and self-hosted facilitators are ruled out while that holds.
- The books are public, in git, and every number links to its source.
- Wallet private keys never enter git, state, CI, Workers or browsers.
- Later phases add a radar blip, a billing-check line, human issues with
  runbooks, and credential changes through `infra-credentials`, each in the
  PR that needs them.
