# Priority Issues

Build v1 in this order. Each item links the decision that explains it.

- TODO: Add a `cli` binary that downloads the data, runs the simulation and writes `rooting.json`. ([0001](docs/decisions/0001-static-site-rebuilt-weekly.md))
- TODO: Build the Dioxus `web` page that reads `rooting.json`: the week's games sorted by impact, the Bears' current odds, and the NFC North toggle. ([0002](docs/decisions/0002-optimize-super-bowl-odds.md))
- TODO: After the first deploy, check <https://jimhillr-bear-down.pages.dev> shows this week's guide and that Nightly CD redeploys it the next night (the `cloudflare/` deploy itself is in place). ([0009](docs/decisions/0009-cloudflare-pages-nightly-cd.md))

# Backlog

- TODO (agent-generated): Tune the Elo parameters (K, home field, margin multiplier, regression) against past seasons by Brier score. The decision's starting values give 0.219 on 2005 to 2026 games. ([0004](docs/decisions/0004-elo-win-probabilities.md))
- TODO: Use betting point spreads (already in `games.csv`) for the current week's win probabilities. ([0004](docs/decisions/0004-elo-win-probabilities.md))
- TODO: Rooting guide for the playoffs. ([0008](docs/decisions/0008-v1-scope.md))
- TODO: Archive of past weeks' guides. ([0008](docs/decisions/0008-v1-scope.md))
- TODO: Pick a custom domain to replace the pages.dev address. ([0001](docs/decisions/0001-static-site-rebuilt-weekly.md))
