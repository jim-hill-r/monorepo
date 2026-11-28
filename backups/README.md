# backups

Backups, restore drills and disaster recovery for everything in the repo's
architecture that holds state (Jim, 2026-11-28: "We need to make a plan for
this and create a project for it"). The design is
[decision 0030](../standards/docs/decisions/0030-backups-and-disaster-recovery.md).

| Need | How |
| --- | --- |
| Know what to protect | [docs/inventory.md](docs/inventory.md): every stateful thing, its tier, RPO and RTO |
| A copy of the repo outside GitHub | An offline copy Jim downloads every 90 days (a settings check reminds him) |
| A copy of Cloudflare data outside Cloudflare | Nightly `wrangler d1 export`, encrypted with age, kept 90 days as a GitHub Actions artifact |
| Proof a restore works | A monthly drill workflow, and a yearly tabletop drill with Jim |
| Steps when something is lost | [disaster-recovery.md](../standards/docs/runbooks/disaster-recovery.md) |

The cookbook content is the product, and a future book, so it comes first:
it is in tier 0, and the offline copy check asks about its recipes by name.

Status: planned. The work is in [issues/](issues/), epic "Backups", phases
1 and 2. Until phase 1 lands this is a docs-only project. It is a technical
platform project.

Cost: $0 a month. No new vendor.
