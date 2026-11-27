# Turn on Workers Analytics Engine

- **Kind:** one-time
- **Status:** open (issues/human/cloudflare-enable-analytics-engine.md)
- **Time:** about 2 minutes
- **Cost:** $0. Analytics Engine on Workers Free allows 100,000 data points
  written and 10,000 queries a day; the telemetry Worker expects well under
  1% of that ([free-tiers.md](../free-tiers.md)).

## What it's for

The `telemetry` Worker (`jimhillr-telemetry`) writes every event to the
`telemetry` Analytics Engine dataset. Cloudflare refuses to deploy a Worker
with an Analytics Engine binding until the account has turned the product
on, so without this the telemetry deploy fails with "You need to enable
Analytics Engine" (code 10089), and bear_down's fan votes go nowhere.

## Why a person

Cloudflare has no API for this switch. Its API has only the Analytics
Engine SQL endpoints, and wrangler points to the dashboard, so no workflow,
IaC stack or Infra Mint step can do it.

## Before you start

- Signed in to the Cloudflare dashboard as an account Super Administrator.

## Steps

1. Open **Workers & Pages**, then **Analytics Engine**
   (`https://dash.cloudflare.com/<account id>/workers/analytics-engine`).
2. Click the button that enables Analytics Engine. Don't add a payment method or move to
   Workers Paid; the free tier covers it.
3. On GitHub, open the latest **Push Main CI/CD** run whose
   `ci-cd / ci (telemetry)` job failed, and click **Re-run failed jobs**.
   Or tell the bear_down thread "done" and an agent re-runs it.

## Where secrets go

None. This makes no credential.

## Verify

The re-run's `ci (telemetry)` job passes, and
`https://jimhillr-telemetry.jim-hill-r.workers.dev/event` answers a `GET`
with 405 (only `POST` is allowed). Then delete
`issues/human/cloudflare-enable-analytics-engine.md` and move this runbook
to "Done" in [README.md](README.md).

## Undo or rotate

Nothing to rotate. To stop collecting, delete the `jimhillr-telemetry`
Worker; turning Analytics Engine off isn't needed and costs nothing to keep.
