# Clean Code

- **Owner:** Robert C. Martin. *Clean Code: A Handbook of Agile Software
  Craftsmanship*, Prentice Hall, 2008.
- **Original:** the book ([publisher page](https://www.oreilly.com/library/view/clean-code-a/9780136083238/)).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- **Intention-revealing names**: a name says what a thing is for, so it
  needs no comment. No abbreviations a newcomer can't decode.
- **Functions do one thing** at one level of abstraction, and read top
  down, a caller before the functions it calls.
- **Comments say why**, not what. Delete commented-out and dead code; git
  keeps it.
- **The boy scout rule**: leave the code a little cleaner than you found it,
  which is how the code quality ratchet asks for no new findings in files
  you touch.

## What we leave

- The "functions of two to four lines" and "almost no comments" extremes.
  They split code into many shallow pieces; see
  [A Philosophy of Software Design](philosophy-of-software-design.md).
- The class-heavy object-oriented advice (inheritance hierarchies, many
  small classes), which doesn't fit Rust's enums, traits and modules.

## Where it shows up

The QUAL-2xx code rules of the code quality process (decision 0015) and
the `code-review` skill.
