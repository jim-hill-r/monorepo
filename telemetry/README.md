# telemetry

A small Rust Cloudflare Worker with one route, `POST /event`. It checks each
event, rate-limits it per IP address, and writes it to the `telemetry`
Analytics Engine dataset. It is phase 5 of the
[deploy-feedback plan](../standards/docs/plans/2026-11-deploy-feedback.md)
and step 2 of bear_down's
[fan feedback plan](../bear_down/docs/plans/2026-11-fan-feedback.md).

Production: <https://jimhillr-telemetry.jim-hill-r.workers.dev/event>

## Sending an event

Post one JSON object, at most 4 KB, as `application/json` or `text/plain`
(what `navigator.sendBeacon` sends, which needs no CORS preflight). Any
origin may post; events carry no credentials.

| Response | Meaning |
| --- | --- |
| 202 | Accepted, or dropped on purpose (below) |
| 400 | Not a valid event; the body says why |
| 413 | Body over 4 KB |
| 429 | Over about one event a second from this IP address |

Every event has `kind`, `project` (letters, digits, `_`, `-`, `.`; at most
32) and an optional `build` (at most 40, such as a commit). Unknown fields
are refused, so nothing unexpected is ever stored.

| `kind` | Fields |
| --- | --- |
| `exposure` | `arm` (`a` or `b`), `season`, `week` (1 to 22), `tab` (`super_bowl`, `playoffs` or `nfc_north`) |
| `feedback` | the `exposure` fields, plus `game` (a game id), `verdict` (what the page showed, such as `CHI` or `doesnt_matter`), `vote` (`yes`, `no` or `confusing`; or `note` for the follow-up event that carries the optional note on a vote just cast, so each vote counts once) and an optional `note` |
| `error` | `page` (at most 200 characters) and `message` |
| `game` | `name` and an optional number `value` |
| `cli` | `command`, `ms` and an optional `error` |
| `deploy` | `commit` (hex SHA) and an optional `pr` number |

```json
{"kind":"feedback","project":"bear_down","arm":"a","season":2026,"week":12,
 "tab":"super_bowl","game":"2026_12_PHI_DAL","verdict":"doesnt_matter","vote":"no",
 "note":"Felt like it mattered"}
```

## Privacy

- Requests with `DNT: 1` or `Sec-GPC: 1` get a 202 and are not stored.
- So do requests with `x-monorepo-synthetic: 1` or a user agent containing
  `monorepo-evals` (UAT, evals and smoke tests; eval engine phase 3), so
  test traffic never counts as fans. Browsers in tests only need the user
  agent, since `navigator.sendBeacon` can't add a header.
- The IP address is used only as the rate-limit key. It is never stored.
- Notes, error messages and CLI errors have anything that looks like an
  email address or a phone number replaced with `[email]` or `[number]`,
  and are cut to 200 (notes) or 500 characters.

## What is stored

Each event is one Analytics Engine data point, indexed by `project`:

| Column | Every kind | `exposure` | `feedback` | `error` | `game` | `cli` | `deploy` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `blob1`..`blob3` | kind, project, build | | | | | | |
| `blob4` | | arm | arm | page | name | command | commit |
| `blob5` | | tab | tab | message | | error | |
| `blob6`..`blob9` | | | game, verdict, vote, note | | | | |
| `double1` | | season | season | | value | ms | pr |
| `double2` | | week | week | | | | |

Analytics Engine is only the inbox. Per the repo's "files before databases"
rule, a weekly workflow (fan feedback plan, step 5) exports the votes into
the repo as JSON Lines files.

## Free tier

Workers Free: 100,000 requests a day, and Analytics Engine: 100,000 data
points a day, both far above our traffic. The rate limit (10 events per 10
seconds per IP address, per Cloudflare location) keeps one client from
using much of either. `mise run deploy` runs `free-tiers.sh worker` first,
so the Worker is only created within the account's Workers budget
([free-tiers.md](../standards/docs/free-tiers.md)).

## Tasks

`mise run ci` checks formatting, lints for the native and wasm targets,
builds for wasm and runs the event tests natively. `mise run dev` runs it
with `wrangler dev`; `mise run deploy` builds it with `worker-build` and
deploys it with `wrangler deploy`, which needs `CLOUDFLARE_API_TOKEN` (the
`jimhillr-deploy` token that Infra Mint makes) and `CLOUDFLARE_ACCOUNT_ID`.
