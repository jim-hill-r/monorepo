# Plan: reviewers sign off on review decks in pane

> For the executive review slide decks feature in pane we should provide a
> way to mark a deck as reviewed and by who. This likely means we need auth
> to login to pane (which we probably should have anyway). When we check it
> as reviewed we should be able to provide feedback which then somehow gets
> fed back into the repo to be actioned by our agents.
>
> Jim, 2026-11-28

Part of the review dashboard epic
([2026-11-review-dashboard.md](2026-11-review-dashboard.md)), Phase 2
(executive reviews). Issues are in `pane/issues/` under the epic
"Deck sign-off".

## What a reviewer does

1. Opens a deck on pane's Review page and reads it as slides.
2. Signs in to pane (Auth0, through `auth_sdk`).
3. Presses **Sign off**, picks a verdict (**Reviewed** or **Reviewed, needs
   changes**), and adds feedback: one or more items, each with an optional
   slide number.
4. Pane commits a sign-off record and one issue file per feedback item to
   the open `pane-edit` pull request. Once that merges, the Reviews page shows
   "Reviewed by jim on 2026-11-28", and agents pick the feedback up through
   `start-a-new-task` like any other issue.

## Where it lives: files, not a database

Following "files before databases", a sign-off is a committed file next to
the deck, and feedback becomes ordinary issue files.

- **Sign-off record**: `docs/reviews/<folder>/signoffs/<date>-<reviewer>.md`.
  It is permanent: the deck stays reviewed after its feedback issues are
  closed (deleted).

  ```markdown
  ---
  reviewer: jim
  date: 2026-11-28
  verdict: changes
  ---

  - [slide 4] The cost slide should say which free tier is nearest its limit.
    (cookbook/issues/review-2026-11-25-cookbook-1.md)
  ```

- **Feedback**: one issue file per item in the reviewed project's own
  `issues/` folder (the root `issues/` when the project has none),
  `review-<folder>-<n>.md`, `kind: todo`, `priority: priority`, epic
  `Review feedback`, with a link back to the deck and the sign-off. The
  reviewer wrote it, so it carries no `agent-generated` tag. Priority is
  `priority` because a reviewer asked for it; the reviewer can pick backlog.

The record lists the issue paths it filed, so pane can show each item as
open (file still there) or done (deleted).

Pure functions in `pane_model::signoff` parse and render both files, so the
snapshot, the browser and the Worker share one tested implementation.

## Who the reviewer is

- **Sign-in**: pane signs in with Auth0 through `auth_sdk`'s
  `WebAuthProvider`, like cahokia and cookbook. Reading pane stays open (it
  shows only what is already public in the repo); only signing off needs a
  sign-in.
- **Who may sign off**: an Auth0 API for pane with one permission,
  `review:decks`, carried by a `deck-reviewer` role (Auth0 role-based access
  control, on the free plan), the same shape as the flags Worker's
  `manage:{app}` ([decision 0018](../../../standards/docs/decisions/0018-feature-flags.md)).
  Jim assigns the role in the Auth0 dashboard, never in git.
- **What name is recorded**: the repo is public, so no Auth0 `sub` or email
  goes in it. When Jim assigns the role he also sets the reviewer's public
  handle (for example `jim`) in their Auth0 `app_metadata`, and an Auth0
  Action adds it to the access token as a `https://jimhillr-pane/reviewer`
  claim. The Worker writes that handle and nothing else about the person.
- **Check on the server**: the Worker's `POST /api/reviews/<folder>/signoff`
  verifies the access token against Auth0's public keys (issuer, audience,
  expiry, `review:decks`) and fails closed.

This replaces the plan's earlier "Cloudflare Access, only Jim" for pane's
write path: Access would guard the site but can't say which of several
reviewers signed. The `/api/*` routes that only read public data stay open.

## How the write reaches the repo

The Worker commits with the token from the review dashboard plan's write path
(a fine-grained token on Jim's account, Contents and Pull requests write on
`jimhillr/monorepo`, as a Worker secret) to the `pane-edit` PR
(`pane/issues/api-edits-route-pane-worker-inserts.md`, order 190). The commit
message names the reviewer handle. PR triage merges `pane-edit` PRs like any
other safe PR, and a reviewer can't push anything but the two file shapes
above, which the Worker validates.

## Phases

| Phase | What | Needs |
| --- | --- | --- |
| 1 | **Format and display** (built with this plan): `pane_model::signoff`, the snapshot reads `signoffs/` into each review, and the Reviews and Review pages show who signed off, the verdict, and each feedback item as open or done | Nothing |
| 2 | **Auth0 for pane** (IaC): the pane client and API with `review:decks`, the `deck-reviewer` role and the reviewer-handle Action in the `infra` auth0 stack. A credential change: Jim's second approval (decision 0010) | Nothing |
| 3 | **Sign-in and form**: `auth_sdk` sign-in in `pane/web`, the sign-off form on the Review page | 2 |
| 4 | **Sign-off route**: the Worker route, the token check, and the commit to the `pane-edit` PR | 3, the pane Worker (PR #759), the `/api/edits` write path (order 190) and its token |

Until phase 4 lands, a reviewer can sign off by committing the sign-off
and issue files by hand (or asking an agent to), in exactly the format above;
pane shows them the same way.

## Cost

$0 a month. Auth0 free plan: reviewers are a handful of the 25,000 monthly
active users. Workers: a few sign-offs a week against 100,000 requests a day.
GitHub: one commit per sign-off. No new vendor (Auth0 is an approved
exception). Building it is about four PRs of Claude Code effort.

## Risks

- A stolen reviewer session could file junk issues. Contained by the role
  check, short-lived tokens, the Worker accepting only the two file shapes,
  and every write going through a PR that the checks and Jim's post-merge
  acknowledgement see.
- An agent could weaken the token check. Code review on the Worker and the
  credential-change rule for anything in the auth0 stack.

## Decisions for Jim

1. Sign-in is Auth0 through `auth_sdk` with a `deck-reviewer` role, instead
   of Cloudflare Access, for the sign-off write path.
2. Sign-offs are files next to the deck, and each feedback item becomes its
   own issue file in the reviewed project, worked by agents as usual.
3. The recorded name is a public handle Jim sets in Auth0, never an email.
