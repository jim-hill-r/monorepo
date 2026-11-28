---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 12
added: 2026-11-26
---

Dependency advisories that come through `openidconnect` 4 in `auth_sdk` (and so `cahokia`): RUSTSEC-2023-0071 (rsa 0.9, Marvin timing attack, fixed in rsa 0.10) and, in cahokia, the crypto-bigint 0.5 timing advisory (fixed in 0.7, through elliptic-curve, p256 and p384). Both need an `openidconnect` release on the newer crates. Check for one and upgrade; if there is none, propose dated ignores with a reason in `auth_sdk/deny.toml` and `cahokia/deny.toml` (we only verify Auth0 tokens with a public key, which the rsa attack doesn't touch), in a PR labelled `needs-human`. Everything else in both lockfiles was cleared on 2026-11-26. (SEC-003, `bash standards/scripts/deps-check.sh --all`.)
