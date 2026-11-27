---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The README says a `function` "must return a single value" and its examples (`function add(a, b) { a + b }`, in the code generator and JIT sections) leave out `return`, but a body with no `return` gives 0 (`test_jit_no_args_returns_constant`, `test_e2e_integer_literal`). So the JIT example returns 0, not 7. Either make the last expression the value, or reject a `function` with no `return` with a clear error (LANG-002), and fix the README examples to match. Found while writing LANG-001 (docs/requirements.md).
