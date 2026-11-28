# my_lang requirements

What my_lang promises the person writing programs in it. The format and the
rules are in [standards/docs/evals.md](../../standards/docs/evals.md); the
evals that check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the README, `docs/threat.toml`,
`docs/cost.toml`, the issue files and the existing tests. my_lang has no
deep review yet, so these are a first guess at Jim's intent. It is a Rust
library today, with no command line tool, release or deploy, so every eval
runs in `ci`. Its production evals arrive with the CLI
([docs/issues/production-evals-with-the-cli.md](issues/production-evals-with-the-cli.md)).

## LANG-001: A program runs and gives the right answer

- Priority: must
- Why: A compiler is only useful if what it builds computes what the
  program says.
- Accept: a program of `function` and `output` declarations compiles and
  runs in-process; `return` gives the function's value, and a body that
  ends in an expression without `return` gives that expression's value; `+`, `-`, `*` and
  `/` work on 64-bit integers; `==`, `!=`, `<`, `>`, `<=` and `>=` give 1
  or 0; variables hold what was assigned; one function can call another;
  functions take up to 8 integer arguments.

## LANG-002: A mistake gives a clear error, never a crash

- Priority: must
- Why: Someone learning the language will write broken programs, and a
  panic or a silently wrong program hides what they got wrong.
- Accept: an unknown character, an unterminated string, an integer too
  large for 64 bits, a missing bracket or an unexpected token, an undefined
  variable or function, calling a function that doesn't exist, and calling
  with too many or too few arguments, and a `function` that gives no value
  (empty, or ending in an assignment, with no `return`) each return an
  error that names the problem, and none of them panics.

## LANG-003: Side effects stay where they are declared

- Priority: must
- Why: The point of the language is pure functions with side effects kept
  in `input` and `output` declarations.
- Accept: an `input` declaration that contains `return` is rejected with
  an error; `function` and `output` declarations may return a value; each
  kind of declaration is parsed and compiled as its own kind.

## LANG-004: The compiled LLVM IR can be read

- Priority: should
- Why: People check what the compiler made from their program, and later
  stages (AOT, optimisation) build on the same module.
- Accept: compiling a program gives one LLVM module with one `i64`
  function per declaration, named as in the source, taking `i64`
  parameters and containing the matching integer instructions (`add`,
  `mul`, `icmp`, `call`); string literals compile to a pointer to a
  constant string.
