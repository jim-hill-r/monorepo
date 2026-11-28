---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

Parse errors print Rust debug output and no position, such as `parse error: UnexpectedToken { expected: "RightParen", found: RightBrace }`, and a number too big for 64 bits shows as `found: Illegal('0')`. Give `ParseError` a `Display` that names the token in source form (`expected ')', found '}'`) with its line and column, and give an oversized integer its own "integer too large for 64 bits" error, which LANG-002 lists. Update "How to read a compile error" in `docs/user/README.md` to match, and the transcript in `tests/cmd/LANG-002-parse-error.stderr` with `TRYCMD=overwrite cargo test --test cli_transcripts`.
