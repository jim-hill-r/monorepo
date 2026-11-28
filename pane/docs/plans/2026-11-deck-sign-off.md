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
2. Signs in to pane (Cloudflare Access, with their GitHub account).
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

Internal people (Jim, reviewers, engineers) and product users sign in
through separate systems ([decision 0024](../../../standards/docs/decisions/0024-internal-sign-in.md)), so nobody internal has an account in a product's
user pool (Jim, 2026-11-28: "The risk is that our end users would be in the
same auth domain as our internal use cases. That is not my favorite
idea."). Products keep Auth0; internal tools such as pane use Cloudflare
Access.

- **Sign-in**: pane's Worker URL sits behind **Cloudflare Access**, with
  **GitHub** as its login method (Phase 3 of the review dashboard plan,
  `pane/issues/put-pane-behind-cloudflare-access-allowing.md`, order 110).
  Access does the whole sign-in, so pane has no sign-in code. Free for up to
  50 users; Cloudflare and GitHub are approved vendors.
- **Who may sign off**: the Access policy allows an Access group of internal
  people that Jim keeps by hand in the Cloudflare dashboard, since their
  emails can't be in the public repo. Only people in that group reach pane
  at all.
- **What name is recorded**: no email goes in the repo. A Worker secret,
  `PANE_REVIEWERS`, maps each reviewer's Access email to a public handle
  (for example `jim`), and Jim sets it by hand. The Worker writes that handle
  and nothing else about the person, and refuses a signed-in person with no
  handle.
- **Check on the server**: the Worker's `POST /api/reviews/<folder>/signoff`
  verifies the `Cf-Access-Jwt-Assertion` token against the Access team's
  public keys (issuer, the pane application's audience, expiry) and fails
  closed, so a misconfigured Access policy can't let anyone through.

Considered and not chosen: Auth0 for internal users too (shares a user pool
with products); a second Auth0 tenant for internal users (the free plan
allows one tenant per account); our own GitHub OAuth in the Worker (more code
and a client secret for what Access does free).

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
| 2 | **Access for pane**: the review dashboard plan's order 110, with GitHub as the login method and the internal Access group. A credential change (an Access policy): Jim's second approval (decision 0010) | Nothing |
| 3 | **Sign-off form** on the Review page, validated in the browser with `pane_model::signoff::files` | Nothing (it posts to phase 4's route) |
| 4 | **Sign-off route**: the Worker route, the Access token check, the reviewer handle, and the commit to the `pane-edit` PR | 2, the pane Worker (merged in #759), the `/api/edits` write path (order 190) and its token |

Until phase 4 lands, a reviewer can sign off by committing the sign-off
and issue files by hand (or asking an agent to), in exactly the format above;
pane shows them the same way.

## Cost

$0 a month. Cloudflare Access free plan: a few internal people of its 50
users. Workers: a few sign-offs a week against 100,000 requests a day.
GitHub: one commit per sign-off. No new vendor. Building it is about three
PRs of Claude Code effort.

## Risks

- A stolen reviewer session could file junk issues. Contained by the Access
  group, Access's session length, the Worker accepting only the two file shapes,
  and every write going through a PR that the checks and Jim's post-merge
  acknowledgement see.
- An agent could weaken the token check. Code review on the Worker and the
  credential-change rule for Access policies.

## Decisions for Jim

1. Internal tools (pane) sign in with Cloudflare Access and GitHub login;
   products keep Auth0 (Jim chose "Access with GitHub login" on the decision
   card, 2026-11-28). Internal people never share a user pool with a
   product's users.
2. Sign-offs are files next to the deck, and each feedback item becomes its
   own issue file in the reviewed project, worked by agents as usual.
3. The recorded name is a public handle from the `PANE_REVIEWERS` Worker
   secret, never an email.
