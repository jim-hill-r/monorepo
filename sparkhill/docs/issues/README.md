# Epic notes

## Blue Eel parity

Jim (2026-11-27): Blue Eel, the web app in `../webapp`, is top priority:
deploy it and bring it to parity with the live app at
<https://blue.eel.education>. It deploys from `../webapp_cloudflare` to
<https://jimhillr-sparkhill-webapp.pages.dev>. The first gaps were found from the
live app's page head and its public `eel3-data` S3 bucket, before cloud
sandboxes could load blue.eel.education. Those four (install icons, each
learner's progress, the live letter shapes, capitals) are done. Since
2026-11-27 the sandbox network allows blue.eel.education, so agents can
open the live app in Playwright to compare. Doing so found two more gaps:
the Pattern and Freeform techniques (phase 5) and the practice menu and
settings panel (phase 6). The techniques (#901), the menu and settings
panel, and moving a learner from Tracing to Pattern to Freeform after
each level are done. The
live app's source is the public repo jimhillr/BlueSteel (Quasar/Vue, in
`frontend/src/client`). The live app has no capitals level, and its
Upload Practice switch sends tracings to S3, which SPRK-006 rules out, so
neither is copied. Never switch Upload Practice on or read the bucket's
`practice/` files: they hold real learners' data. Add any more gaps Jim
lists and work them by `order`. The web app's
`@smoke` Playwright tests run against the deploy after every push to `main`
and feed the `cd` evals in `../evals.toml`. They test our app, not the live
one: run against blue.eel.education on 2026-11-27, 1 of 10 passed (Begin
opens the letter level), because the live app asked for the learner's
name in its settings panel and the app then asked on the home page. The
app now has the same settings panel, Student, Sequence and Technique,
though the live app's Sequence is a code for a few hard-coded learners'
lists, where ours takes the letters and words themselves.

## User feedback

See [standards/docs/plans/2026-11-user-feedback.md](../../../standards/docs/plans/2026-11-user-feedback.md)
and decisions 0020 and 0021 (accepted, 2026-11-28): in-app help from each
project's user guide, support requests and feedback, built as libraries
each product runs on its own origin, with product data in tenant-scoped
D1 tables. One issue per phase, in this folder and in `sparkhill/` and
`bear_down/` `docs/issues/`. Phase 1 replaces the eval engine's phase 6 model
issue.
