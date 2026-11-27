# tofu_state requirements

What the OpenTofu state Worker promises `infra`, the only client it
serves. The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the README,
[decision 0009](../../standards/docs/decisions/0009-opentofu-state-http-backend.md),
`docs/threat.toml`, `scripts/smoke.sh` and the existing tests. tofu_state
has no deep review yet, so these are a first guess at Jim's intent. Its
production checks can't hold a password, so they check only what anyone
on the internet sees; `scripts/smoke.sh` checks the read password after
each deploy.

## TOFU-001: infra's state is kept and read back

- Priority: must
- Why: Every `infra` plan and apply starts from the state this Worker
  stores; losing or mixing it up would make OpenTofu recreate or destroy
  real resources.
- Accept: `GET /states/<stack>` returns the latest state stored with
  `POST`, or 204 when there is none; each stack is kept apart; a body
  that isn't UTF-8 JSON is refused with 400; any other path answers 404
  and any other method 405.

## TOFU-002: Only infra can read or change state

- Priority: must
- Why: State holds every resource's IDs and settings, and a forged write
  could make an apply delete anything.
- Accept: a request without a password, or with a wrong one, gets 401;
  the read password can only `GET` (403 for `POST`, `LOCK` and
  `UNLOCK`); a Worker missing its password secrets refuses every request;
  passwords are compared in constant time.

## TOFU-003: Two applies never overwrite each other

- Priority: must
- Why: Two runs writing the same stack at once would lose one run's
  changes to real resources.
- Accept: `LOCK` takes a stack's lock or answers 409 with the holder's
  lock info; while locked, only a `POST` naming the holder's lock ID is
  stored; `UNLOCK` releases the lock only for the same ID; a write that
  names a lock it no longer holds fails.

## TOFU-004: A bad apply can be rolled back

- Priority: should
- Why: An apply that writes a broken state must be undoable without a
  backup.
- Accept: the last 10 versions of each stack are kept and older ones are
  deleted, so deleting the latest version (the query in the README) goes
  back one apply.

## TOFU-005: Errors and logs never show state

- Priority: should
- Why: State is encrypted by OpenTofu, but even its ciphertext and the
  stack layout shouldn't leak through an error or a log.
- Accept: a store failure answers 500 with `state store error` and
  nothing about the state; the Worker never logs a request body.
