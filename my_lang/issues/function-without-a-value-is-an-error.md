---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The README says a `function` "cannot be void", but a `function` whose body is empty or ends in an assignment (no `return`, no trailing expression) still compiles and gives 0 (`test_function_no_explicit_return`, `test_jit_body_ending_in_assignment_gives_zero`). Decide whether that should be a clear codegen error naming the function (LANG-002), and if so add the error, update those tests and the README's "Keywords" section. Found while making a trailing expression the function's value.
