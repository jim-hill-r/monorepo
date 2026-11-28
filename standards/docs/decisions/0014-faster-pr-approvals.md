# 0014: Faster pull request approvals

- Status: Accepted (Jim chose a reviewer account and post-merge review,
  2026-11-27); the reviewer starts once its human task is done
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
  check waits on `infra/docs/issues/human/github-app-infra-read.md`.
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
| 5 | **Reviewer account** `jimhillr-reviewer`: a workflow approves, as that account, the agent PRs its written rules allow, and Jim acknowledges every merged PR in pane afterwards | 0 for PRs the rules allow; 4 for the rest; 1 tap per merged PR to acknowledge | No ruleset change; a new token (Pull requests write) that approves without Jim, kept in a main-only environment | This PR, plus Jim's human task |

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

### Option 5: the reviewer account (chosen)

Jim first picked option 4 with a post-merge check ("auto approve plus
pane acknowledgements of every pr"), then changed it, 2026-11-27: "I think
we should go with a third GitHub account to do approvals so we can be
explicit about what is being auto-approved and establish a process for
that approver. Lets call it jimhillr-reviewer".

- **Who approves.** `jimhillr-reviewer`, a GitHub account with the Write
  role, used only by the `All Auto Review` workflow. The Default ruleset
  keeps its one required approval; the reviewer's approval satisfies it
  for the PRs its rules allow, and Jim's for the rest.
- **Its rules** (PR-003, `standards/scripts/auto-review.sh`, listed under
  "The reviewer account" in [pull-requests.md](../pull-requests.md)): an
  agent PR into `main`, out of draft, every check finished and green, no
  red or orange label, no protected path (workflows, agent rules,
  standards, infra, threat models, requirements, evals, auth, costs,
  security headers, deploy settings), at most 100 files and 1,500 lines,
  no changes requested.
- **Its record.** Each approval is pinned to the head commit and its text
  names the rules version and what passed; each run's summary lists every
  open agent PR with "approve" or "skip" and why.
- **Its token.** `REVIEWER_TOKEN`, a fine-grained token on that account
  (this repo only; Pull requests read and write; Contents, Checks and
  Commit statuses read), stored only as a secret of the `reviewer`
  environment, which deploys from `main` alone. It is never in git or
  OpenTofu state, and no Claude session holds it. It can approve but not
  push or merge. Making the account and token is a human task
  ([runbook](../runbooks/jimhillr-reviewer-account.md)); Infra Mint can't
  make a token for another account.
- **Why it's safe enough.** The workflow runs `main`'s script on a
  schedule and never checks out PR code, so a PR can't change its rules,
  and a PR that changes the rules touches `standards/`, which the rules
  refuse, so only Jim can approve that.
- **Post-merge review.** Pane's Acknowledge page lists every merged PR,
  whoever approved it, until Jim acknowledges it, with a "Flag for
  follow-up" link that opens a prefilled `fix` issue file on GitHub.
  Acknowledgements are kept in that browser's storage for now; keeping them
  across devices waits for pane's write path (Phase 5).

Cost: $0. GitHub accounts, environments and Actions minutes are free for a
public repo; the workflow runs 96 times a day for under a minute each.

## Decision

- Options 1 and 2 (the Approve page and one approval per stack) are built.
- Option 5: `jimhillr-reviewer` approves what its rules allow, and Jim
  reviews every merged PR afterwards on pane's Acknowledge page. Options 3
  and 4 are not taken.

## Consequences

- Pane's CSP allows `connect-src https://api.github.com`. The Approve and
  Acknowledge pages show GitHub's refusal when the 60-an-hour limit is
  used up.
- Threads with stacks follow "Stacked pull requests" in
  [pull-requests.md](../pull-requests.md).
- Until `REVIEWER_TOKEN` exists, `All Auto Review` passes without
  approving anything, and every PR still waits for Jim.
- The rules can only get looser through a PR Jim approves himself.
- 2026-11-27, rules v2: Jim asked that "no prs with red attention flags
  get auto merged", "cost changes must be reviewed by jim" and "lets make
  it needs human review". The
  reviewer now also refuses a description whose Attention section has a
  red or orange tag (or no Attention section), and needs the new `Human
  review` check (PR-004), which holds a red or cost PR until a human (never
  an agent account or `jimhillr-reviewer`) approves it ("Human review" in [pull-requests.md](../pull-requests.md)).
