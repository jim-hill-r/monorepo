# tofu_state

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/tofu_state.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

The Cloudflare Worker `jimhillr-tofu-state`, which stores OpenTofu state for
`infra` ([decision 0009](../standards/docs/decisions/0009-opentofu-state-http-backend.md)).
It serves OpenTofu's [`http` backend](https://opentofu.org/docs/language/settings/backends/http/)
with the state in the D1 database `jimhillr-tofu-state`, so state never
touches git.

| Request | Who | What it does |
| --- | --- | --- |
| `GET /states/<stack>` | read or write password | The latest state, or `204` when there is none |
| `POST /states/<stack>` | write password | Stores a new version. While locked, only the lock holder (`?ID=`) may write |
| `LOCK /states/<stack>` | write password | Takes the lock, or `409` with the holder's lock info |
| `UNLOCK /states/<stack>` | write password | Releases the lock, only for the same lock ID |

- Passwords come as basic auth (`TF_HTTP_PASSWORD`). The read password
  (`TOFU_STATE_READ_PASSWORD`) is for PR plans, which run with `-lock=false`;
  the write password (`TOFU_STATE_WRITE_PASSWORD`) is only in the `infra`
  environment. Both are compared in constant time. A Worker without its
  secrets refuses every request.
- The Worker keeps the last 10 versions of each stack, so a bad apply can be
  rolled back with one D1 query, and deletes older ones.
- State is encrypted by OpenTofu before it is sent; the passphrase is only in
  GitHub, so the Worker and D1 only ever hold ciphertext. The Worker never
  logs a request body.
- `infra` never manages this Worker: IaC can't manage the store its own
  state lives in.

## Layout

- `src/backend.rs`: the protocol, with an in-memory store for the unit tests.
- `src/lib.rs`: the Worker entry point and the D1 store (wasm only).
- `migrations/`: the D1 schema (`states`, `locks`).
- `tests/e2e.sh`: real `tofu init`, `plan` and `apply` against
  `wrangler dev` with a local D1, including two concurrent applies where the
  second fails on the lock, the read password failing to apply, and the
  10-version limit.

## Deploy

`.github/workflows/tofu-state-deploy.yml` deploys on every push to `main`
that changes `tofu_state/` (or by hand), in the `infra` environment, the only
place the write password lives. It runs `mise run deploy:worker` (plain `deploy` does nothing, because
`push-main.yml` runs it in every changed project), which:

1. checks the Workers budget (`free-tiers.sh worker`),
2. finds or creates the D1 database (`free-tiers.sh d1-database`),
3. applies the migrations and deploys,
4. copies the two passwords from GitHub into Worker secrets.

Then `scripts/smoke.sh` checks the live Worker: no password gets `401`, the
read password reads every stack in `infra/out/` and can't `LOCK`. The
Worker's origin comes from the account's workers.dev subdomain
(`scripts/origin.sh`), so nothing hard-codes it. The passwords and the
deploy token are made by the Infra Mint workflow (`infra/scripts/mint.sh`),
which also queues this deploy when the passwords change.

## Rolling back a bad apply

The states table keeps versions per stack. To go back one version, delete
the latest:

```sql
DELETE FROM states WHERE stack = 'auth0'
  AND version = (SELECT MAX(version) FROM states WHERE stack = 'auth0');
```

A stuck lock is released with `tofu force-unlock <ID>`, which sends `UNLOCK`
with that ID.
