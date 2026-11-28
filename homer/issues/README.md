# homer issues

One file per issue; the format is in
[standards/docs/issue-management.md](../../standards/docs/issue-management.md).

## Web UI

Goal: make homer a free web app on `homer.jimhillr.com`, since "as a cli I
don't think it's very useful" (Jim, 2026-11-28). The why and the order are
in the [roadmap](../docs/roadmap.md); the data source and the Zillow and
RapidAPI rules are [decision 0001](../docs/decisions/0001-web-ui-data-source.md)
(proposed).

- Phase 1 (shared core) and phase 2 (score a house by hand) don't depend
  on how lookups work, so they are priority issues and can start before
  decision 0001 is accepted.
- Phase 3 (look up with your own key) waits for decision 0001; its CORS
  check decides whether the site needs a pass-through Pages Function.
- Phases 4 and 5 follow in order.
