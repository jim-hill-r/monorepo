# Check there is no payment method

- **Kind:** recurring, every 30 days
- **Status:** open (`cost/docs/issues/human/check-there-still-no-payment-method.md`);
  the `*-billing` and `*-budget` settings checks in
  `standards/settings.toml` ask the same each period
- **Time:** about 4 minutes
- **Cost:** $0; it's what keeps everything at $0

## What it's for

Every service is used on its free plan
([free-tiers.md](../free-tiers.md)). With no card on file a vendor can't
bill; a card turns on pay-as-you-go.

## Why a person

What a read-only API can show is checked weekly by the **Billing check**
(`standards/scripts/billing-check.sh`): Cloudflare's payment methods and
subscriptions and PostHog's subscription (both checked since 2026-11-27),
and GitHub's charges this month once `GITHUB_BILLING_READ_TOKEN` exists. The
rest has no API: a personal GitHub account's card and Actions budget, Auth0's
plan, RapidAPI billing, Squarespace billing and claude.ai billing
([free-tiers.md](../free-tiers.md#billing-checks)).

## Before you start

- Signed in to GitHub, Auth0, RapidAPI, Squarespace and claude.ai.
- Open the latest **All Free Tier Usage** run in jimhillr/monorepo >
  Actions and look at its **Billing check** table. Cloudflare and PostHog
  are checked there, so do steps 2 and 4 only if it shows one of them as
  "not checked" or failing.

## Steps

1. GitHub: github.com/settings/billing > Payment information shows no card;
   Budgets and alerts shows an Actions budget of $0 with "Stop usage" on.
2. Cloudflare, only if not checked: dash.cloudflare.com > Manage Account > Billing shows the
   Free plan, no payment method, and no Workers Paid, R2 paid or Images.
3. Auth0: manage.auth0.com > Settings > Subscription shows Free and no
   card.
4. PostHog, only if not checked and the account exists: Settings >
   Organization > Billing shows the free plan and no subscription.
5. RapidAPI: rapidapi.com > My Apps > Billing shows the free Zillow plan
   and no card.
6. Claude: claude.ai > Settings > Billing: Claude Code spend this month is
   within $25 and no extra usage is on.
7. Squarespace: account.squarespace.com > Billing shows only your existing
   domain renewals, and no new domain, paid plan or add-on since last time.

## Where secrets go

None.

## Verify

Reply "ok", or what you found. An agent records it and files a fix if
something is wrong.

## Undo or rotate

If a card was added, remove it on that page and reply with what was
charged, if anything.
