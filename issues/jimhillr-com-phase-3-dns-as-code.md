---
kind: todo
priority: backlog
tags: [agent-generated, agent-ignore]
epic: "jimhillr.com / Phase 3: DNS as code"
order: 30
added: 2026-11-28
---

Once the jimhillr.com zone is active on Cloudflare (issues/human/move-jimhillr-com-dns-to-cloudflare.md): have infra's cloudflare stack read the zone and own its records (no change on first apply); add DNS edit on this zone to the deploy token through Infra Mint (credential change, settings.toml and secret-rotation.md); add cloudflare_zones and Pages custom domain limits to standards/radar/limits.toml; register the zone in standards/resources.toml and the isolation standard's shared resource table (owner infra); point the dangling DNS check at it. Plan: [2026-11-jimhillr-com.md](../standards/docs/plans/2026-11-jimhillr-com.md), decision [0031](../standards/docs/decisions/0031-jimhillr-com.md). Remove agent-ignore from the epic once Jim accepts 0031.
