# Delete old Pages projects

- **Kind:** on demand (after a Pages project is renamed or retired)
- **Status:** open (`issues/human/delete-jimhillr-pane-pages-project-after-worker.md`)
  for `jimhillr-pane`, which pane left for a Worker. The three projects
  left over from the `jimhillr-<project>` rename were deleted by Jim on
  2026-11-28.
- **Time:** about 5 minutes
- **Cost:** $0. Frees one of the 25 budgeted Pages projects per deletion.

## What it's for

Old Pages projects keep serving stale builds and count against the Pages
project budget. Every project is now named `jimhillr-<project>`
([naming.md](../naming.md#cloudflare-pages-projects)).

## Why a person

It deletes something, and IaC never deletes
([decision 0007](../decisions/0007-infrastructure-as-code.md), item 8).
Agents also have no Cloudflare write token for it.

## Before you start

- The replacement is live and serving.
- No Auth0 application lists a URL on the old domain. Once a project is
  deleted anyone can claim its `*.pages.dev` name, and Auth0 must not send
  sign-in tokens there.
- Signed in to dash.cloudflare.com.

## Steps

General case, for any retired Pages project:

1. Auth0 > Applications: search each application's Allowed Callback URLs,
   Allowed Web Origins and Allowed Logout URLs for the old `*.pages.dev`
   address. Stop if any is there.
2. dash.cloudflare.com > Workers & Pages. Open the project > Settings >
   Delete project, and type its name to confirm.
3. Set `deleted` on its `[[retired]]` entry in `standards/resources.toml`.

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

Workers & Pages lists only the Pages projects the repo deploys, and the
old `*.pages.dev` URLs no longer load.

## Undo or rotate

A deleted project can't be restored; deploy again under a new name if
ever needed.
