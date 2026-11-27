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
settings panel (phase 6). The techniques themselves are done (#901);
moving a learner from Tracing to Pattern after a level is still open. The
live app's source is the public repo jimhillr/BlueSteel (Quasar/Vue, in
`frontend/src/client`). The live app has no capitals level, and its
Upload Practice switch sends tracings to S3, which SPRK-006 rules out, so
neither is copied. Never switch Upload Practice on or read the bucket's
`practice/` files: they hold real learners' data. Add any more gaps Jim
lists and work them by `order`. The web app's
`@smoke` Playwright tests run against the deploy after every push to `main`
and feed the `cd` evals in `../evals.toml`. They test our app, not the live
one: run against blue.eel.education on 2026-11-27, 1 of 10 passed (Begin
opens the letter level), because the live app asks for the learner's name
in its settings panel rather than on the home page.
