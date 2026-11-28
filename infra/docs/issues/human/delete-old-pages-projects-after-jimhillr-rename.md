---
added: 2026-11-26
unblocks: "frees three of the 25 budgeted Pages projects and stops the old sites serving stale builds, see the Cloudflare Pages projects section of standards/docs/naming.md"
runbook: standards/docs/runbooks/delete-old-pages-projects.md
---

## Requirement

Once `jimhillr-cahokia.pages.dev`, `jimhillr-pane.pages.dev` and `jimhillr-controlled-chaos.pages.dev` are live and the `auth0` stack in `infra` has been applied once, delete the old Pages projects `cahokia-web`, `pane` (pane-dbc.pages.dev) and `controlled-chaos` (controlled-chaos-5v4.pages.dev) in the Cloudflare dashboard (Workers & Pages).

Wait for that apply: it sets the Cahokia Auth0 application's URLs to exactly the `jimhillr-cahokia` ones, which removes every `cahokia-web.pages.dev` URL. Once `cahokia-web` is deleted, anyone can claim `cahokia-web.pages.dev`, and Auth0 must not send sign-in tokens there. This stays a person's task because IaC never deletes anything (decision 0007, item 8).

## Why it can't be automated

It deletes something, and IaC never deletes (decision 0007, item 8); agents also have no Cloudflare write token.

## Runbook

[Delete old Pages projects](../../../../standards/docs/runbooks/delete-old-pages-projects.md)
