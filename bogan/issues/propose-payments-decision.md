---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Propose how Bogan takes money in a decision record in
`standards/docs/decisions/`, before any payment code is written. The
README's note ("use existing currencies or consider this as the front for
bringing money into a more centralized socialist currency") is an open
question, and no approved vendor (GitHub, Cloudflare, Squarespace) processes
marketplace payments, so any payment provider needs Jim's sign-off under
the vendor rule in `AGENTS.md`. Compare the options (for example no
payments at first and link out to sellers, a payment processor with
marketplace payouts, or a custom currency), each with its money cost, fees,
free-tier limits, legal and compliance load, and the Claude Code effort to
build it. Label the PR `needs-human`. Depends on the product brief
(`write-product-brief.md`).
