---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

A parse error prints the parser's Rust debug form, not words: `my_lang run` on `function main() { return (1 + 2 }` says `my_lang: parse error: UnexpectedToken { expected: "RightParen", found: RightBrace }`, with no line or column (see `tests/cmd/LANG-002-parse-error.stderr`). LANG-002 asks for an error that names the problem for someone learning the language. Give `ParseError` a `Display` that reads like `expected ")" but found "}" at line 1, column N`, track token positions in the lexer for it, and update the transcript with `TRYCMD=overwrite cargo test --test cli_transcripts`.
