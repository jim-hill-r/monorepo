---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

Infra apply on `main` is red: the `cloudflare` stack fails to create any
`cloudflare_web_analytics_site`. Cloudflare answers `400 Bad Request` with
`web_analytics.configuration.api.autoInstallInvalid` for all seven sites
(bear_down, cahokia, controlled_chaos, cookbook, design_system, pane,
sparkhill). It has failed on every push since 2026-11-27 03:44 UTC, for
example [run 45996384873](https://github.com/jimhillr/monorepo/actions/runs/45996384873)
(job `apply (cloudflare)`).

Cause: `infra/src/stacks/cloudflare.rs` sets `auto_install = true` on each
site, whose `host` is a `jimhillr-*.pages.dev` name. Cloudflare's automatic
beacon injection works only for hostnames proxied through a zone in the
account ([Web Analytics: get started](https://developers.cloudflare.com/web-analytics/get-started/)).
A Pages project gets it from its own one-click setting, not from a site
created over the API with a bare host, so the API refuses `auto_install`
there.

Fix, choosing one:

- Set `auto_install` to `false`, add each site's beacon snippet (the site
  tag the resource exports) to the apps, and update the stack test and
  `infra/README.md`. This stays fully in code.
- Or drop the resources and turn on Web Analytics in each Pages project's
  settings, which is a hand setting: it needs a human issue with a runbook
  and an entry in `standards/settings.toml`.

Check the PR's Infra Plan, and that the next Infra apply run on `main` is
green. Never apply by hand.
