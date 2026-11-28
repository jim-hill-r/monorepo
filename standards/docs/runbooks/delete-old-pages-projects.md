# Delete old Pages projects

- **Kind:** on demand (after a Pages project is renamed or retired)
- **Status:** open (`infra/docs/issues/human/delete-old-pages-projects-after-jimhillr-rename.md`)
  for `cahokia-web`, `pane` and `controlled-chaos`, and
  (`pane/docs/issues/human/delete-jimhillr-pane-pages-project-after-worker.md`) for
  `jimhillr-pane`, which pane left for a Worker
- **Time:** about 5 minutes
- **Cost:** $0. Frees three of the 25 budgeted Pages projects.

## What it's for

Old Pages projects keep serving stale builds and count against the Pages
project budget. Every project is now named `jimhillr-<project>`
([naming.md](../naming.md#cloudflare-pages-projects)).

## Why a person

It deletes something, and IaC never deletes
([decision 0007](../decisions/0007-infrastructure-as-code.md), item 8).
Agents also have no Cloudflare write token for it.

## Before you start

- The new sites are live: `jimhillr-cahokia.pages.dev`,
  pane's Worker (below), `jimhillr-controlled-chaos.pages.dev`.
- No Auth0 application lists a URL on the old domain. For
  `cahokia-web.pages.dev` that means the `auth0` stack has been applied
  once (it sets Cahokia's URLs to exactly the `jimhillr-cahokia` ones), or
  you've removed those URLs by hand in Auth0 > Applications > Cahokia.
  Once a project is deleted anyone can claim its `*.pages.dev` name, and
  Auth0 must not send sign-in tokens there.
- Signed in to dash.cloudflare.com.

## Steps

1. Auth0 > Applications > Cahokia > Settings: search Allowed Callback
   URLs, Allowed Web Origins and Allowed Logout URLs for `cahokia-web`.
   Stop if any is there.
2. dash.cloudflare.com > Workers & Pages. For each of `cahokia-web`,
   `pane` (pane-dbc.pages.dev) and `controlled-chaos`
   (controlled-chaos-5v4.pages.dev): open it > Settings > Delete project,
   and type its name to confirm.

## Pane's Pages project, after the move to a Worker

Pane now runs as the Worker `jimhillr-pane`, which serves its build as
Workers static assets, so its Pages project `jimhillr-pane` is retired.
A Worker and a Pages project can share a name, so take care to delete the
right one.

1. Open `https://jimhillr-pane.jim-hill-r.workers.dev` and
   `https://jimhillr-pane.jim-hill-r.workers.dev/api/health`. Stop unless
   the first shows pane and the second shows `{"status":"ok"}`.
2. dash.cloudflare.com > Workers & Pages. Open the `jimhillr-pane` entry
   marked **Pages** (its URL ends in `.pages.dev`; the full address is in
   `standards/resources.toml`), not the Worker.
   Settings > Delete project, and type its name to confirm.
3. Verify: the Worker `jimhillr-pane` is still listed and its URL still
   loads; the Pages address no longer does. Then set `deleted` on its
   `[[retired]]` entry in `standards/resources.toml`.

## Where secrets go

None.

## Verify

Workers & Pages lists only `jimhillr-*` Pages projects, and the old
`*.pages.dev` URLs no longer load.

## Undo or rotate

A deleted project can't be restored; deploy again under a new name if
ever needed.
