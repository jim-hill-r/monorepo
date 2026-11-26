---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 16
added: 2026-11-26
---

Dependency advisories in `homer`. `cargo update` in homer, or move it off reqwest 0.11, to clear RUSTSEC-2026-0098, -0099, -0104 and -0382 (rustls-webpki), -0346 and -0349 (rustls), -0258 (h2), -0381 (webpki-roots), -0592 (url) and the unsound anyhow (RUSTSEC-2026-0190). Most come through reqwest 0.11's rustls 0.21; reqwest 0.12 or later fixes them. (SEC-003, found by `bash standards/scripts/deps-check.sh --all`, 2026-11-26.)
