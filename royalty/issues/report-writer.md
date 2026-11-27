---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 70
added: 2026-11-27
---

Add `royalty report`: run the scan and the calculation and write `royalty-report.json` (a versioned format for tools) and `royalty-report.md` (for people), one row per producer family with the tier, the units, the amount before and after each cap and the budget, the reason for the final number, and each payee with their payment methods. List "no terms" and "not billable (transitive)" packages separately. The output is deterministic: sorted, no timestamps except the period, the same bytes for the same inputs. It holds counts and amounts only, never paths, names of people or source.
