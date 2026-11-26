---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 12
added: 2026-11-26
---

Dependency advisories in `auth_sdk`. `cargo update` in auth_sdk to clear RUSTSEC-2026-0381 (webpki-roots), RUSTSEC-2026-0592 (url, fixed in 2.5.9) and the yanked spin 0.9.8, then check `cargo deny --config ../standards/deny.toml check advisories` passes. RUSTSEC-2023-0071 (rsa, Marvin timing attack) has no fixed release; auth_sdk only verifies Auth0 tokens with a public key, which the attack doesn't touch, so propose a dated ignore for it in an auth_sdk `deny.toml` (that PR gets `needs-human`). (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
