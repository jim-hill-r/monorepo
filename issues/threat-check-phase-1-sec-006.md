---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Threat check
order: 40
added: 2026-11-26
---

Threat check phase 1. Add SEC-006 to `//:audit` (every `Cargo.lock` and `package-lock.json` has a `dependabot.yml` entry, and every entry's directory exists), and fix `dependabot.yml`: drop `ciam`, `dioxus_ssg`, `dioxus_static_website`, `example/binary` and `example/example_rust_library`, add every Cargo and npm lockfile, and keep the pinned `dioxus` and `sycamore` bumps ignored.
