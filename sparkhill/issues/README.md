# Epic notes

## Blue Eel parity

Jim (2026-11-27): Blue Eel, the web app in `../webapp`, is top priority:
deploy it and bring it to parity with the live app at
<https://blue.eel.education>. It deploys from `../webapp_cloudflare` to
<https://jimhillr-sparkhill-webapp.pages.dev>. The gaps were found from the
live app's page head and its public `eel3-data` S3 bucket, because cloud
sandboxes can't load blue.eel.education itself. The four gaps found
(install icons, each learner's progress, the live letter shapes, capitals)
are done; add any more Jim lists and work them by `order`. The web app's
`@smoke` Playwright tests run against the deploy after every push to `main`
and feed the `cd` evals in `../evals.toml`.
