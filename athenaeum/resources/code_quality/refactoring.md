# Refactoring

- **Owner:** Martin Fowler. *Refactoring: Improving the Design of Existing
  Code*, Addison-Wesley, 2nd edition 2018.
- **Original:** the book, and Fowler's free
  [catalog of refactorings](https://refactoring.com/catalog/) and
  [bliki](https://martinfowler.com/bliki/).
- **Licence:** copyright, summary only.
- **Added:** 2026-11-27, from the code quality process (decision 0015).

## What we take

- **Code smells as the review vocabulary**: long function, long parameter
  list, duplicated code, feature envy (a function that uses another type's
  data more than its own), primitive obsession (a `String` or `u32` standing
  in for a domain type), shotgun surgery (one change touches many files),
  divergent change, data clumps. Name the smell in a finding.
- **The rule of three**: duplicate once if you must; on the third copy,
  extract it.
- **Refactor in small, tested steps**, separate from behaviour changes, so
  each step is easy to review. Tests come first.

## What we leave

The Java and JavaScript mechanics of individual refactorings; Rust's
compiler and rust-analyzer do most of them.

## Where it shows up

The `code-review` skill's rubric and the QUAL rules of the code quality
process (decision 0015).
