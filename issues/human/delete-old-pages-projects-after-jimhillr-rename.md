---
added: 2026-11-26
unblocks: "frees three of the 25 budgeted Pages projects and stops the old sites serving stale builds, see the Cloudflare Pages projects section of standards/docs/naming.md"
---

Once `jimhillr-cahokia.pages.dev`, `jimhillr-pane.pages.dev` and `jimhillr-controlled-chaos.pages.dev` are live, first remove every `cahokia-web.pages.dev` URL from the Cahokia Auth0 application (callbacks, web origins, logout URLs), then in the Cloudflare dashboard (Workers & Pages) delete the old Pages projects `cahokia-web`, `pane` (pane-dbc.pages.dev) and `controlled-chaos` (controlled-chaos-5v4.pages.dev). Remove the Auth0 URLs first: once `cahokia-web` is deleted, anyone can claim `cahokia-web.pages.dev`, and Auth0 must not send sign-in tokens there.
