# A Philosophy of Software Design

- **Owner:** John Ousterhout. *A Philosophy of Software Design*, Yaknyam
  Press, 2nd edition 2021.
- **Original:** the book ([author page](https://web.stanford.edu/~ouster/cgi-bin/aposd.php)),
  and his [talk at Google](https://www.youtube.com/watch?v=bmSAYlu0NcY).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- **Complexity is the enemy**: it shows as change amplification, cognitive
  load and unknown unknowns, and it builds up one small shortcut at a time.
- **Deep modules**: a small interface over a lot of functionality. A module
  whose interface is as complex as its body adds cost and no value.
- **Information hiding**: each module keeps one design decision to itself.
  Leaking it (shared formats, call order rules) is the main source of
  coupling.
- **Define errors out of existence**: design an API so a case can't fail
  (an empty range, an idempotent delete) instead of adding an error for it.
- **Design it twice**: for anything public, sketch two different designs
  before picking one.
- **Comments describe what the code can't**: the interface contract and
  the reasons, written before or with the code.

## What we leave

Nothing in particular. We use it as the counterweight to
[Clean Code](clean-code.md)'s very small functions: when the two disagree,
prefer the deeper function.

## Where it shows up

The QUAL-1xx design rules of the code quality process (decision 0015) and
the `code-review` skill.
