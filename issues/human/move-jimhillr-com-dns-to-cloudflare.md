---
added: 2026-11-28
unblocks: issues/jimhillr-com-phase-3-dns-as-code.md
runbook: standards/docs/runbooks/move-jimhillr-com-dns-to-cloudflare.md
---

## Requirement

Add jimhillr.com to the Cloudflare account on the Free plan, check the imported records against the domain check's list (email records above all), and change the nameservers at the registrar (Squarespace or AWS) to Cloudflare's. Unblocks phase 3 and every site's move to <name>.jimhillr.com (plan standards/docs/plans/2026-11-jimhillr-com.md, decision 0031). Do it only after Jim accepts 0031 and phase 1's check has run.

## Why it can't be automated

Adding a zone and changing a domain's nameservers are account-level actions on a domain Jim pays for; the registrar has no API agents may use, and agents never hold a credential that can move a domain.

## Runbook

[Move jimhillr.com's DNS to a free Cloudflare zone](../../standards/docs/runbooks/move-jimhillr-com-dns-to-cloudflare.md)
