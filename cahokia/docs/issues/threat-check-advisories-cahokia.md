---
kind: fix
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 13
added: 2026-11-28
---

Moderate and low Dependabot alerts on `cahokia/Cargo.lock` (Security > Dependabot, 2026-11-28). #2 (moderate): glib 0.18 is unsound in its `Iterator` impls (GHSA-wrw7-89jp-8q8g, fixed in 0.20); it comes through the Dioxus desktop webview stack, and dioxus is pinned (AGENTS.md), so record a dated ignore with that reason in `cahokia/deny.toml` and dismiss the alert as tolerable risk, in a PR labelled `needs-human`, unless a patch release of the gtk-rs 0.18 line fixes it. #181 (low): rand 0.8.5 with a custom logger (GHSA-cq8v-f236-94qc), fixed in 0.8.6, so `cargo update -p rand@0.8.5`. #90 (low): rsa 0.9.9 can panic on a prime equal to 1 (GHSA-9c48-w39g-hm26), fixed in 0.9.10, so `cargo update -p rsa`. Run cahokia's CI after the updates. The rsa Marvin advisory stays with threat-check-advisories-openidconnect.md.
