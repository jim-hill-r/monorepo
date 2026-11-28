---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-28
---

Dividing by zero crashes the program instead of giving an error: `function main(a, b) { a / b }` run with `my_lang run div.my 1 0` dies with "Floating point exception" (SIGFPE, exit 136), because `/` compiles to LLVM `sdiv` with no check. `i64::MIN / -1` has the same problem. LANG-002 says a mistake gives a clear error and never crashes. Add a check before each division that reports "division by zero" (and the overflow case) from `run` and from an executable `build` writes, with tests in `tests/cli_tests.rs`, then remove the "Floating point exception" entry from the user guide's Getting help.
