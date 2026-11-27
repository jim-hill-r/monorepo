# Check there is no payment method

- **Kind:** recurring, every 30 days
- **Status:** open (`issues/human/check-there-still-no-payment-method.md`);
  the `*-billing` and `*-budget` settings checks in
  `standards/settings.toml` ask the same each period
- **Time:** about 5 minutes
- **Cost:** $0; it's what keeps everything at $0

## What it's for

Every service is used on its free plan
([free-tiers.md](../free-tiers.md)). With no card on file a vendor can't
bill; a card turns on pay-as-you-go.

## Why a person

These billing pages have no read API an agent can use, and agents must
never hold a token with billing access.

## Before you start

- Signed in to GitHub, Cloudflare, Auth0, PostHog (once it exists) and
  RapidAPI.

## Steps

1. GitHub: github.com/settings/billing > Payment information shows no card;
   Budgets and alerts shows an Actions budget of $0 with "Stop usage" on.
2. Cloudflare: dash.cloudflare.com > Manage Account > Billing shows the
   Free plan, no payment method, and no Workers Paid, R2 paid or Images.
3. Auth0: manage.auth0.com > Settings > Subscription shows Free and no
   card.
4. PostHog: Settings > Billing shows no card.
5. RapidAPI: rapidapi.com > My Apps > Billing shows the free Zillow plan
   and no card.
6. Claude: claude.ai > Settings > Billing: Claude Code spend this month is
   within $25 and no extra usage is on.

## Where secrets go

None.

## Verify

Reply "ok", or what you found. An agent records it and files a fix if
something is wrong.

## Undo or rotate

If a card was added, remove it on that page and reply with what was
charged, if anything.
