---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 50
added: 2026-11-26
---

Threat check phase 1. Add SEC-003 (cargo-deny: advisories, sources, licences) and SEC-004 (`npm audit --omit=dev --audit-level=high`) as warnings, run only for affected projects whose lockfile changed plus nightly over all of them. Triage every existing advisory into a `FIX` issue file or a dated `deny.toml` ignore with a reason; add gitleaks and cargo-deny to `standards/radar/radar.json` as tool blips.
