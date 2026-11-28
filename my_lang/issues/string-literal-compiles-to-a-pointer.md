---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-28
---

A string literal compiles but gives a meaningless number: `function main() { "hi" }` prints something like `139677709557760` (a pointer) from `my_lang run`. The README says strings aren't supported by the code generator yet, so codegen should refuse a string literal with an error such as "strings aren't supported yet" until they are, as LANG-002 asks for mistakes. Add a codegen test, then remove the string entry from the user guide's Getting help.
