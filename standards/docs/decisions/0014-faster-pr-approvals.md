# 0014: Faster pull request approvals

- Status: Proposed. Options 1 and 2 are built in the PR that adds this
  record, so approving it accepts them; option 3 or 4 is Jim's choice
- Date: 2026-11-27
- Related: [pull-requests.md](../pull-requests.md),
  [review dashboard plan](../../../pane/docs/plans/2026-11-review-dashboard.md),
  [decision 0010](0010-credential-second-approval.md)

## Context

Jim, 2026-11-27: "I need a faster way to review and approve PRs. It
currently takes 5-6 clicks per pr."

How approvals work today, checked on 2026-11-27:

- Agents open every PR as `jimhillr-agt` (Write role, not admin). It can't
  approve its own PRs, so every agent PR needs Jim (`jimhillr`).
- The Default ruleset on `main` requires a PR with 1 approval, dismisses
  approvals on a new push, and has an empty bypass list
  ([runbook](../runbooks/github-repo-setup.md)). The `CI result` required
  check waits on `issues/human/github-app-infra-read.md`.
- Agents turn on squash auto-merge when they mark a PR ready, so Jim's
  approval is already the last step: GitHub merges as soon as it lands.
- That day 40 agent PRs were out of draft, 27 of them green and waiting on
  a review, and 10 were one royalty stack, each needing its own approval.

Today's path for one PR, on the web or the GitHub app: find it in the PR
list (1), open it (2), open Files changed (3), Review changes (4), pick
Approve (5), Submit (6).

## Options

Each option's money cost is $0. Claude Code effort counts against the $25
a month budget.

| # | Option | Taps per PR | Security change | Claude Code effort |
| --- | --- | --- | --- | --- |
| 1 | **Approve page in pane**: every green PR waiting on review, clear ones first, with its 75-word summary, attention badges and a link straight to its review form | 4 (Approve link, Review changes, Approve, Submit) | None: it reads GitHub's public API without a token | This PR |
| 2 | **One approval per stack**: a thread with stacked PRs retargets the top one to `main`, so one approval merges the stack | 4 per stack instead of per PR (royalty: 10 approvals become 1) | None: Jim still approves the whole diff | This PR (a rule) |
| 3 | **One-tap Approve in pane**, and "Approve all clear" | 1, or 1 for a whole batch | New credential: Jim's fine-grained token (Pull requests: read and write, this repo only) as a Worker secret, behind Cloudflare Access | About 3 or 4 threads: pane's Phase 3 Worker and Access first ([PR 759](https://github.com/jimhillr/monorepo/pull/759)), then the approve route |
| 4 | **Code-owner ruleset**: 0 required approvals, but code-owner review required, with Jim as owner of the risky paths | 0 for most PRs; 4 for PRs touching owned paths or labelled red | Weakens review: agent code outside owned paths merges on green CI with no human look | About 1 thread: `CODEOWNERS`, a required check that fails on red labels, a runbook; Jim edits the ruleset by hand |

Free-tier headroom: option 1 makes 2 unauthenticated GitHub API calls a
page load against a limit of 60 an hour per network. Option 3 adds a few
Worker requests a day against 100,000, and Access is free up to 50 users.

### Option 1: the Approve page

Pane gets an **Approve** page (`/approve`, built in this PR). The browser
calls GitHub's public REST API, which allows CORS, with no token: the open
PRs (drafts, stacks, bodies, labels), and a search for open PRs with
passing checks that still need a review. The rules are in
`pane_model::queue`:

- **Ready to approve**: out of draft, in the search, not labelled red.
  Clear ones first, then orange, each oldest first.
- **Attention badges** come from labels: red for `needs-human` and
  `credential-change`, orange for `security` and `cost`. A PR from anyone
  but `jimhillr`, `jimhillr-agt` or Dependabot (a fork) is orange too, with
  an "outside author" badge. Red PRs are listed apart, under "Needs a
  decision first".
- **Summary**: the first paragraph of the body after the attribution
  lines, links reduced to their text, cut at 75 words.
- **Approve on GitHub** opens the PR's Files changed tab, where GitHub's
  review form is.

It saves finding and opening each PR and lets Jim decide from the summary
which ones need the diff. It can't approve by itself, which is what keeps
it free of any secret.

### Option 2: one approval per stack

When a thread's stack of PRs is all green, it retargets the top PR to
`main`, so that PR holds the whole stack's diff, adds a `Stack: #a, #b, …`
line listing every PR in it, and moves the other PRs in the stack back to
draft. Jim approves the one PR, it squash-merges, and the thread closes the
rest as merged in it. The rule is under "Stacked pull requests" in
[pull-requests.md](../pull-requests.md). Tradeoff: `main` gets one squash
commit for the stack instead of one per step, and the one diff is bigger;
the step PRs stay readable as drafts until they are closed.

### Option 3: one-tap Approve

A pane Worker route posts an `APPROVE` review with Jim's own token, so a
tap in pane approves and auto-merge does the rest; "Approve all clear"
approves every clear PR in one tap. The token has to be Jim's because
`jimhillr-agt` can't approve its own PRs. It lives only as a Worker
secret, never in git or state, and pane sits behind Cloudflare Access for
Jim's email, with every `/api/*` call checking the Access JWT. The risk:
whatever can call that route approves as Jim, and the route is repo code,
the same reason decision 0010 kept credential approvals out of pane.
Agents still can't deploy a change to it without Jim approving that PR.
It needs a human task for the token and the Access policy, and the
credential second approval (decision 0010).

### Option 4: code-owner ruleset

GitHub can require a code owner's review without requiring any other
approval. `CODEOWNERS` would name Jim for `.github/`, `infra/`,
`standards/`, `auth_sdk/`, `tofu_state/`, `threat/`, `AGENTS.md`, every
`wrangler.toml` and `CODEOWNERS` itself. An "Attention" required check
would fail on red labels, so those still wait for Jim. Everything else
merges by auto-merge once `CI result` is green. Tradeoff: most PRs then
ship without a human reading them, and a risky change in a path nobody
listed (auth code inside an app, say) gets through. It relaxes the main
review control, so only Jim can choose it, and he makes the ruleset change
himself.

## Decision

- Options 1 and 2 need no security decision, so the PR that adds this
  record builds them.
- Recommended next: **option 4**. It removes the most clicks, needs no new
  credential, and the code-owner list keeps Jim on the risky paths. Option
  3 is the choice if Jim wants to keep reading every PR. Jim chooses; until
  then every PR still needs his approval.

## Consequences

- Pane's CSP allows `connect-src https://api.github.com`. The page shows
  GitHub's refusal when the 60-an-hour limit is used up.
- Threads with stacks follow "Stacked pull requests" in
  [pull-requests.md](../pull-requests.md), and PR triage counts the step
  PRs as part of the stack PR.
- If Jim picks option 3 or 4, a follow-up PR adds its human task and
  runbook, and this record's status is updated.
