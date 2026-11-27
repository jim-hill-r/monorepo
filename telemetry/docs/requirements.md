# telemetry requirements

What the telemetry Worker promises the apps that send it events and the
people whose visits those events describe. The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the README, `docs/cost.toml`,
`docs/threat.toml`, the
[deploy-feedback plan](../../standards/docs/plans/2026-11-deploy-feedback.md)
and the existing tests. telemetry has no deep review yet, so these are a
first guess at Jim's intent. Every eval request carries the
`jimhillr-evals` user agent, so the production checks never store a data
point; they check the live routes, not the stored data.

## TELE-001: Apps can send events from any page

- Priority: must
- Why: bear_down's fan votes and every app's errors, game and CLI events
  reach Analytics Engine only through this Worker.
- Accept: `POST /event` with one JSON object of at most 4 KB, sent as
  `application/json` or `text/plain` from any origin, answers 202 and
  writes one data point indexed by `project`; each kind (`exposure`,
  `feedback`, `error`, `game`, `cli`, `deploy`) keeps the fields the
  README lists; a CORS preflight answers 204 and allows any origin.

## TELE-002: Bad events are refused and nothing unexpected is stored

- Priority: must
- Why: The stored data feeds the weekly export and A/B summaries, so a
  malformed or oversized event must never get in.
- Accept: an unknown kind or field, a bad project name, an out-of-range
  value (such as week 40 or arm `c`), a commit that isn't hex, a body that
  isn't JSON or UTF-8 answers 400 saying why; a body over 4 KB answers 413;
  any method but `POST` or `OPTIONS` on `/event` answers 405.

## TELE-003: Opted-out and test traffic is never stored

- Priority: must
- Why: Fans who ask not to be tracked must not be, and test runs must
  never count as fans in the A/B results.
- Accept: a request with `DNT: 1`, `Sec-GPC: 1`,
  `x-jimhillr-synthetic: 1` or a user agent containing `jimhillr-evals`
  answers 202 and stores nothing; other visitors are stored as usual.

## TELE-004: No personal details are stored

- Priority: must
- Why: Events come from the public, so emails, phone numbers and IP
  addresses must never reach the dataset.
- Accept: anything that looks like an email address or phone number in a
  note, error message or CLI error is replaced with `[email]` or
  `[number]` (short numbers such as scores stay); notes are cut to 200
  characters and messages to 500; the IP address is used only as the rate
  limit key and is never written.

## TELE-005: Every response carries the security headers

- Priority: should
- Why: The repo's security rule 3 asks every surface for them, and the
  Worker only ever answers with a status and short text.
- Accept: every response, including 404s, sends
  `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`,
  `X-Frame-Options: DENY` and `Strict-Transport-Security`.
