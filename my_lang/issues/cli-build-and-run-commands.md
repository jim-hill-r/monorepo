---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

Add a `my_lang` command-line binary with `build <file> -o <out>` (calls
`aot::build_executable`) and `run <file>` (JIT-runs `main` and prints the
result), so programs can be compiled without writing Rust.
