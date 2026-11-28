# bear_down_web

The Dioxus page of [bear_down](../README.md): it tells a Bears fan which NFL
games to root for this week. When it loads it fetches `/rooting.json`, which
the `bear_down` CLI writes at build time ([format](../docs/rooting-json.md)),
and ranks the week's games by their swing on the selected goal: the Bears'
Super Bowl odds, their playoff odds, or all four NFC North teams reaching the
divisional round. Each game also takes a fan's yes, no or confusing vote,
sent to [telemetry](../../telemetry/README.md).

The page uses the shared [design system](../../design_system/README.md) with
bear_down's own "Chicago Bears" theme (`../design/theme.css`).

- `src/main.rs`: the page and its tabs.
- `src/data.rs`: the `rooting.json` types and how numbers are shown.
- `src/feedback.rs`: the A/B arm, exposure events and votes.
- `src/teams.rs`: team names and colours.
- `tests/`: Playwright tests for the A/B arms and the votes.

It is deployed by [bear_down/cloudflare](../cloudflare/README.md).
How to build and test it is in [CONTRIBUTING.md](CONTRIBUTING.md).
