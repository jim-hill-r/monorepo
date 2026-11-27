# royalty requirements

What royalty promises the producers who publish a `royalty.toml` and the
consumers who depend on their code, in the format of
[standards/docs/evals.md](../../standards/docs/evals.md). Written
2026-11-27 from the design rules of the
[royalty plan](plans/2026-11-royalty-plan.md#design-rules), which Jim agreed
when he approved the plan. The format they constrain is in the
[royalty.toml reference](royalty-toml.md). Evals are added to
[evals.toml](../evals.toml) as Phase 1 code arrives.

## ROYL-001: Royalty adds nothing to a consumer's running program

- Priority: must
- Why: Libraries and frameworks sit in hot loops, so a consumer must be able
  to adopt a producer's terms without any performance cost (Jim: "All
  solutions should not affect a consumers performance").
- Accept: with default settings on both sides, a consumer's built program
  has no added code, dependency, symbol or I/O from royalty; all measuring
  happens in separate commands, from the lockfile, git, declared numbers,
  CI, deploys or logs.

## ROYL-002: Development, tests, CI and staging are always free

- Priority: must
- Why: Integrators need a free way in, and must be able to rely on it
  without reading every producer's terms (decision 2).
- Accept: no `royalty.toml` can make a consumer owe anything for a
  dev-only or build-only dependency, or for a project in `development`
  stage; a tier that tries to name another environment is rejected by
  `royalty check`.

## ROYL-003: Royalty never stops or slows a program

- Priority: must
- Why: A production operator must never be broken by a billing dispute;
  disputes are settled by people.
- Accept: no mode or field can stop, slow or degrade a program at run time;
  the only thing royalty can fail is the consumer's own `royalty check`,
  and only when the consumer's budget or policy asks it to.

## ROYL-004: The lowest ceiling wins, and the report says which

- Priority: must
- Why: A production user needs a hard upper bound on cost before adopting a
  dependency, from the producer's caps and their own budget.
- Accept: the amount owed to a producer is never above any producer cap
  that applies; a `voluntary` amount never pushes the total over the
  consumer's budget; an `agreement` or `license` amount over the budget
  fails `royalty check`; every amount in a report names the cap or budget
  that set it.

## ROYL-005: Offline and private by default

- Priority: must
- Why: A report reveals what a company uses and how big it is, so the
  consumer decides what leaves their machine.
- Accept: `royalty check`, `scan` and `report` make no network call; a
  report holds package names, counts and amounts only, never source, file
  paths, commit emails or names of people.

## ROYL-006: The same files give the same report

- Priority: must
- Why: A producer and a consumer must be able to check each other's numbers.
- Accept: running `royalty report` twice on the same inputs and period gives
  byte-identical files, with no timestamps other than the period.

## ROYL-007: A file with only `[project]` stays valid for ever

- Priority: must
- Why: Every project in this monorepo already has one, and producers must
  never have to rewrite a file to keep it working.
- Accept: a `royalty.toml` with only `[project]` `name` and `description`
  has no `royalty check` findings and means "no payment terms"; files that
  were valid under schema 1 stay valid as optional fields are added.

## ROYL-008: A royalty.toml never holds a secret

- Priority: must
- Why: The file is public and ships inside packages; a token or bank detail
  in it would leak.
- Accept: the format has no field for a secret, and `royalty check` rejects
  values that look like tokens, keys or private key blocks.

## ROYL-009: Money is whole cents, never floating point

- Priority: must
- Why: Rounding must be the same on every machine so amounts agree to the
  cent.
- Accept: every amount is an integer number of US cents, worked out with
  integer arithmetic and rounded down once per family per period as the
  reference describes.
