# Move jimhillr.com's DNS to a free Cloudflare zone

- **Kind:** one-time
- **Status:** open (issues/human/move-jimhillr-com-dns-to-cloudflare.md)
- **Time:** about 20 minutes, plus up to a day for the nameservers to take
- **Cost:** $0. One zone on Cloudflare's Free plan. The domain stays
  registered where it is, so its renewal doesn't change. If the DNS is a
  Route 53 hosted zone today, deleting it afterwards saves $0.50 a month.

## What it's for

Every site moves to `<name>.jimhillr.com`
([decision 0031](../decisions/0031-jimhillr-com.md),
[plan](../plans/2026-11-jimhillr-com.md)). Pages custom domains, Worker
custom domains and DNS as code in `infra` all need the domain's zone on
Cloudflare. Without it, phases 3 to 5 stay blocked.

## Why a person

Adding a zone and changing a domain's nameservers are account-level
actions on a domain Jim pays for. Squarespace has no API for them, and
agents never hold a credential that can move a domain (AWS or Cloudflare).

## Before you start

- Decision 0031 is accepted.
- The weekly domain check (`all-domains.yml`, plan phase 1) has run at
  least once. Its summary says where the domain is registered, which
  nameservers answer for it, and its `MX`, `TXT` and `CAA` records. Keep
  that run open.
- Signed in to the Cloudflare account (the one that holds the
  `jimhillr-*` Pages projects) as a Super Administrator, and to the
  registrar the check names: Squarespace (account.squarespace.com) or AWS
  (the account with the Route 53 registration).

## Steps

1. In the Cloudflare dashboard, choose **Add a domain** (Websites > Add a
   site), type `jimhillr.com`, keep **Quick scan for DNS records** on,
   and pick the **Free** plan. Never pick a paid plan or add a card.
2. On the review page, compare the records Cloudflare found with the
   domain check's summary. Add any that are missing, above all `MX`, SPF
   and DKIM `TXT` records and any site verification `TXT` records. For
   records that serve email or point at Squarespace, set the proxy status
   to **DNS only** (grey cloud).
3. Continue. Cloudflare shows two nameservers, such as
   `xxx.ns.cloudflare.com`. Copy them.
4. Change the nameservers at the registrar:
   - **Squarespace:** account.squarespace.com > Domains > jimhillr.com >
     DNS > Domain nameservers > Use custom nameservers. Enter the two
     Cloudflare nameservers and save.
   - **AWS:** Route 53 > Registered domains > jimhillr.com > Actions > Edit
     name servers. Replace all of them with the two Cloudflare nameservers
     and save.
   - If DNSSEC is on at the registrar, turn it off first, and turn it on
     again later from Cloudflare (DNS > Settings).
5. Back in Cloudflare, choose **Check nameservers**. The zone turns
   **Active** within minutes to 24 hours, and Cloudflare emails when it
   does.
6. Only if the old DNS was a Route 53 hosted zone: a week after the zone is
   Active and email still works, delete that hosted zone (Route 53 >
   Hosted zones > jimhillr.com > delete its records, then the zone). This
   stops its $0.50 a month. IaC never deletes, so this stays your step.

## Where secrets go

None. This makes no credential. The token scope for DNS edit is added
later by the Infra Mint workflow (plan phase 3).

## Verify

- Cloudflare shows the `jimhillr.com` zone as **Active** on the Free plan.
- The next domain check run shows Cloudflare's nameservers and the same
  `MX` and `TXT` records as before.
- Send a test email to an address on the domain, if it has one, and check
  that it arrives.
- Tell the project "jimhillr.com is on Cloudflare" so an agent deletes the
  issue file and starts phase 3.

## Undo or rotate

Put the old nameservers back at the registrar (the domain check's earlier
run lists them). Leave the old DNS host's records in place until the
Cloudflare zone has been Active for a week, so switching back keeps
working. Removing the zone from Cloudflare is a deletion, so only a
person does it.
