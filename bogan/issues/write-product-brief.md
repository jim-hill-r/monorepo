---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Write a product brief for Bogan in `bogan/docs/plans/` (plans live in the
repo, per `AGENTS.md`), as a PR for Jim to review, labelled `needs-human`.
Bogan has only a three-line README today, so no agent can start building
it. The brief should cover: who the users are (sellers, buyers), what
"supply-chain transparency" means as data (origin, maker, each hop to the
buyer, and how a claim is backed), how Bogan enforces its rule against
white-labelled and copy-cat listings, the smallest first version worth
shipping, and how it fits the repo rules: approved vendors (GitHub,
Cloudflare), free tiers only, and files before databases (listings as
committed files until a hosted store is decided under decision 0004).
Give each choice its cost and free-tier headroom. Once Jim approves it,
add `bogan/docs/requirements.md` (prefix `BOGN`, already reserved in
`standards/docs/naming.md`) and break the first version into issue files.
