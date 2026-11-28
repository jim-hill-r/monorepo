# Plan: only live resources exist, and orphans are cleaned up

Started 2026-11-26 from Jim's request:

> Whenever we rename we need to ensure we don't leave behind orphaned
> resources. Can you add a new thread to work through a plan on how we can
> ensure our live resources are the only ones needed and orphans or
> temporary resources are cleaned up regularly?

And later the same day:

> The IaC solution might also be a part of our orphaned resources solution.
> But we still need a separate orphaned resources solution to make sure
> every live resource gets on boarded to IaC including the GitHub repo
> settings and Claude settings etc....

Status: **approved** by Jim (PR #669, 2026-11-26), with the recommended
answer to every question (see "Decisions" near the end). The work items
are issue files in the root [issues/](../issues/) under the epic
"Resource cleanup", one group per phase, and the steps only Jim can do are
in [issues/human/](../issue-management.md#human-issues). Part 6 (onboarding every live
resource to infrastructure as code) was added after approval and is
**proposed** until Jim approves the PR that adds it.

This plan builds on the [threat check](2026-11-threat-check.md) rather than
duplicating it. The threat check's phase 3 already plans a weekly live check
that flags Pages projects and Workers no `threat.toml` claims. This plan
widens that one check to every kind of resource we create, adds a way to
retire a resource on purpose (the rename case), and adds the step the
threat check leaves open: actually deleting what nobody needs, safely and
only with Jim's approval.

## Goal

- **Every live resource is claimed by a file in the repo.** A Pages
  project, Worker, database, branch, release or sign-in URL that no file
  claims is an orphan, and shows up within a week.
- **A rename or removal can't leave anything behind.** The PR that renames
  or removes a resource records the old one as retired, or CI fails.
- **Temporary resources clean themselves up.** Preview deployments, caches
  and branches of closed PRs are deleted on a schedule with no one asking.
- **Deleting anything that was live needs Jim's approval** and happens in
  an order that can't open a hole (for example, sign-in URLs are removed
  before the site they point at, so nobody can claim the freed subdomain and
  receive our users' sign-in codes).
- **Every live resource is onboarded to infrastructure as code (IaC)**,
  including GitHub repo settings and Claude settings. A resource or setting
  that is live but not defined in IaC is a finding of its own,
  **unmanaged**, until it is onboarded or exempted with a reason. The IaC
  tool is OpenTofu driven from Rust
  ([decision 0007](../decisions/0007-infrastructure-as-code.md)); this
  plan only finds the gaps, and the IaC work fills them.
- **No local clone, $0.** Everything runs as GitHub workflows and agent
  skills on free tiers, per [AGENTS.md](../../../AGENTS.md).

## Where we start

Live inventory read from the Cloudflare API and GitHub on 2026-11-26
(read-only):

| Resource | What exists | Claimed by |
| --- | --- | --- |
| Pages projects | 5: `jimhillr-cookbook` (37 deployments), `cahokia-web` (20), `pane` at `pane-dbc.pages.dev` (17), `jimhillr-bear-down` (10), `controlled-chaos` at `controlled-chaos-5v4.pages.dev` (2) | Each `wrangler.toml` `name` on `main`. All 5 are claimed today. |
| Workers, D1, KV | None | - |
| R2 | Unknown: the token can't read R2. R2 bills once enabled, so it must stay off (free-tiers.md). | - |
| DNS zones, custom domains | None; every site is on `*.pages.dev` | - |
| Auth0 | Applications for cahokia and cookbook; callback URLs, origins and logout URLs live only in the Auth0 dashboard | Nothing in the repo yet (the threat check's `[auth] callbacks` will) |
| Git branches | 13: `main`, 11 `claude/*`, 1 `dependabot/*` | Open PRs; nothing checks the rest |
| GitHub releases | One nightly prerelease per game (`nightly-downloads.yml`), assets replaced in place | The workflow's game matrix |
| Actions caches and artifacts | Counted weekly by `free-tiers.sh usage`; artifacts use `retention-days` 1 or 90 | Nothing deletes caches of closed PRs |

What is about to go wrong: the `jimhillr-{project}` rename moves
`cahokia-web`, `pane` and `controlled-chaos` to new Pages projects. The
`free-tiers.sh pages-project` guard creates the new project on the first
deploy, and nothing deletes the old one. Each old project would keep its
last deployment public at its old `*.pages.dev` address (pane's holds
Jim-only data until Cloudflare Access is in front of it), keep using one of
our 25 budgeted Pages projects, and, once deleted later by hand, free a
subdomain that any Auth0 callback still pointing at it would trust.

Existing pieces to reuse:

- `standards/scripts/free-tiers.sh` already lists Pages projects and
  Workers with the read-only Cloudflare token, and runs weekly from
  `all-free-tier-usage.yml`.
- `all-delete-branches.yml` deletes named branches, skips the default branch
  and open PR heads, and is a dry run by default. (The
  `delete-stale-branches.sh` local script from 2026-11-25 is superseded by
  it and is not reused.)
- The threat check's `threat.toml` `[[surface]] url` and `[auth] callbacks`,
  and the cost check's `cost.toml`, already say what each project runs.
- The eval engine's `human-check` entries in `standards/settings.toml` ask
  Jim to confirm dashboard-only settings on a schedule.

## Part 1: claims (what should exist)

A resource is **claimed** when a file on `main` names it. Claims come from
files projects already have, so there is nothing new to keep in sync:

| Kind | Claimed by |
| --- | --- |
| Pages project, Worker | `name` in a `wrangler.toml` |
| D1, KV, Queue, R2 bucket, Cron Trigger | A binding or `[triggers]` entry in a `wrangler.toml` |
| Public URL, Auth0 callback, origin, logout URL | `threat.toml` `[[surface]] url` and `[auth] callbacks` (once threat check phase 2 writes them; until then the Auth0 part waits, see phase 4) |
| DNS record, custom domain | `threat.toml` `[[surface]] url` |
| GitHub release | The game matrix in `nightly-downloads.yml` |
| Git branch | `main`, or the head of an open PR |

Anything else that must exist goes in one small repo-wide file,
**`standards/resources.toml`**, which also holds the retired list:

```toml
# standards/resources.toml

# Live but not claimed by any project file: each needs a reason and a date
# to look at it again, so "keep" never means "forever without a look".
[[keep]]
kind = "pages"
name = "example-landing"
why = "Jim's hand-made landing page, not in the repo."
review = "2027-05-26"

# Retired on purpose. The PR that adds an entry is Jim's approval to delete
# it; the cleanup workflow deletes it after delete_after.
[[retired]]
kind = "pages"                        # pages | worker | d1 | kv | queue | cron | dns | auth0-url | branch | release
name = "cahokia-web"
url = "https://cahokia-web.pages.dev" # every address it answers on
replaced_by = "jimhillr-cahokia"
why = "Renamed to the jimhillr-{project} scheme."
retired = "2026-11-27"
delete_after = "2026-12-04"           # 7 days of grace to undo by removing the entry
deleted = ""                          # set once the sweep confirms it is gone
```

Entries in `[[retired]]` stay after deletion (with `deleted` set), so the
repo remembers every name we gave up and the audit can keep old addresses
out of the code (RES-002 below).

## Part 2: guards on every PR (prevent orphans)

Two new rules, in the `free-tiers.sh check` style, run by the hooks and PR
CI. They read only files, so they cost milliseconds:

- **RES-001, a rename needs a retirement (PR CI and pre-push).** Compare the
  claims on the PR's base with its head. Any claimed resource that
  disappears (a `wrangler.toml` `name` changes, a binding or project is
  removed, a game leaves the release matrix) must appear as a `[[retired]]`
  entry in the same PR, or as a `[[keep]]` entry if it is staying on
  purpose. Fails from day one: it has no false positives, and missing one
  is exactly the bug Jim described.
- **RES-002, no references to retired addresses (`//:audit`).** No tracked
  file outside `standards/resources.toml` and dated history (plans,
  decisions, reviews, changelogs) mentions a retired `url` or `name`. This
  catches the README link, the hard-coded API base URL and the test fixture
  that a rename forgets.
- **Retirement is Jim's call.** PR CI labels any PR that adds a
  `[[retired]]` or `[[keep]]` entry `needs-human`, so it can't merge until
  Jim approves it. That approval is the approval to delete.

The `open-pull-request` skill gains one checklist line: a PR that renames or
removes a resource adds its retirement entry, and lists the inbound
references to remove first (Part 4).

## Part 3: the weekly sweep (find orphans)

`standards/scripts/resources.sh sweep`, a bash and `jq` script next to
`free-tiers.sh` that reuses its Cloudflare helpers, runs as a new job in
the existing weekly `all-free-tier-usage.yml` (Mondays, 7 AM Pacific).
Same read-only token, no new secret. It:

1. Reads every claim, `[[keep]]` and `[[retired]]` entry from the repo.
2. Lists what is live: Pages projects and their deployments, Workers, D1,
   KV, Queues, Cron Triggers, DNS records once a zone exists, branches,
   releases, Actions caches.
3. Sorts each live resource into **claimed**, **kept**, **retired, pending
   deletion**, **retired but still live after `delete_after`** (the cleanup
   failed), **kept past its review date** or **orphan**.
4. Writes the table to the run summary and a JSON report as a workflow
   artifact (`retention-days: 7`), and goes red when there is an orphan, a
   failed deletion or an overdue keep.

This is the threat check's weekly live check, widened. When threat check
phase 3 lands, its URL and access probes run in the same job and read the
same report, so there is one sweep, not two.

**Who acts on it.** A workflow can't open PRs that need judgement, so the
weekly `human-issues-digest` skill gains a step: read the latest sweep
report, and for each new orphan open one draft PR that adds a `[[retired]]`
entry (or a `[[keep]]` entry when the resource is clearly Jim's), labelled
`needs-human`, and list it in the digest. It also sets `deleted` on entries
the sweep confirms are gone. No new scheduled task, so no new Claude spend.

## Part 4: deleting safely (approved resources)

A new workflow, **`all-retire-resources.yml`**, runs weekly after the sweep
and on `workflow_dispatch` (`dry_run` defaults to true, like
`all-delete-branches.yml`). It deletes only resources listed in
`[[retired]]` whose `delete_after` has passed. It never deletes something
just because it is unclaimed. Guards, each a hard stop for that entry:

- The name is still claimed by any file on `main`.
- A Pages project has had a production deployment in the last 7 days
  (something still deploys to it).
- The entry's inbound references (below) aren't all gone yet.

**Safe order.** Work from the outside in, so nothing ever points at a name
someone else could claim:

1. **Stop new deploys.** The rename PR is merged and the new resource is
   serving (the sweep sees it claimed and live).
2. **Remove inbound references.** Auth0 callback, origin and logout URLs;
   DNS records and custom domains; PostHog allowed hosts; links in the
   repo (RES-002 already blocks those). Auth0 and DNS are checked by the
   workflow, which refuses step 3 while any reference remains; until the
   Auth0 read in phase 4 exists, the retirement PR adds a human issue in
   `issues/human/` for Jim to remove the Auth0 URLs, and the workflow waits
   until that file has been deleted (files are deleted once done).
3. **Delete the resource.** For Pages, delete the project, which removes
   every deployment and preview alias with it. For data stores (D1, KV),
   export a dump to a workflow artifact first, kept 90 days.
4. **Remove what protected it.** Cloudflare Access applications for the old
   hostname go last, so private content is never public in between.
5. **Verify.** The next sweep confirms it is gone, and the digest sets
   `deleted`.

The deploy token already has Pages edit, which is all step 3 needs today.
A Workers, D1 or DNS deletion needs that permission on the token; that is a
human issue added when the first such resource exists, not now.

**Old `*.pages.dev` names: delete, don't stub.** An alternative is to keep
the old project with a redirect page, which holds the subdomain so nobody
can claim it. It costs a Pages project slot and keeps a live resource
forever. Our old names are days old with no outside users, and removing the
Auth0 URLs first takes away the reason to claim them, so the plan deletes.
A future rename of a site with real outside users can choose a redirect
stub in its retirement PR (`[[keep]]` with a review date).

## Part 5: temporary resources (no approval needed)

These are temporary by design and rebuilt from git, so automation deletes
them without asking, and each run lists what it deleted in its summary:

| Resource | Rule | Where |
| --- | --- | --- |
| Pages preview deployments | Delete non-production deployments older than 14 days, keeping the newest one per branch that still has an open PR. Production deployments other than the current one older than 30 days too (rollback is a redeploy from git). | `all-retire-resources.yml` weekly |
| Actions caches of a PR | Delete the caches for `refs/pull/N/merge` when PR N closes (GitHub's documented clean-up pattern). | New `pull-request-closed-cleanup.yml` on `pull_request: closed` |
| Branches of merged or closed PRs | Delete 7 days after the PR closes. A closed PR keeps its commits and GitHub's "Restore branch" button, so nothing is lost. | Sweep, through the `all-delete-branches.yml` logic (moved into `resources.sh` so both share it) |
| Branches with no PR | Listed after 14 days; the digest asks Jim before deleting, since one could hold unfinished work. | Sweep and digest |
| Workflow artifacts | `retention-days` of 7 or less on every `upload-artifact`, except review decks (90) with a comment saying why. | New audit rule RES-003 |
| Dependabot entries for missing folders | Already planned as SEC-006 in the threat check. | - |

GitHub's own **"Automatically delete head branches"** setting removes most
merged branches at merge time for free. IaC phase 2's `github` stack turns
it on ([decision 0007](../decisions/0007-infrastructure-as-code.md)), so
it needs no human issue of its own.

## Part 6: every live resource is in IaC

Cleaning up orphans keeps only what we need; IaC makes sure what we need is
written down and reproducible. The sweep checks both, because a resource
can be claimed (we want it) and still be hand-made (only the dashboard
knows how it is set up).

**Findings.** The sweep adds two kinds of finding to Part 3's list:

- **Unmanaged:** live and wanted, but not defined in IaC. Every resource
  kind starts here, apart from what `wrangler.toml` already declares.
- **Drift:** defined in IaC, but the live settings differ from the
  definition (someone changed the dashboard, or an apply failed). For
  kinds IaC manages, decision 0007's weekly `tofu plan -refresh-only`
  already finds drift in the same workflow run; the sweep reads its
  result into the report instead of checking again.

**What counts.** Every kind of resource and setting we have, not only what
costs money or serves traffic:

| Area | Resources and settings |
| --- | --- |
| Cloudflare | Pages projects, Workers, D1, KV, Queues, Cron Triggers, DNS and custom domains, Access applications and policies, API tokens (names and permissions, never values) |
| GitHub repo | General settings (merge methods, auto-merge, delete head branches), rulesets and branch protection, labels, Actions permissions and allowed actions, environments, secret and variable **names**, code security settings, Dependabot settings, webhooks, deploy keys, installed GitHub Apps |
| Auth0 | Applications (callbacks, origins, grant types), APIs, connections, tenant settings |
| PostHog | Project settings and allowed hosts |
| Claude | Project instructions, scheduled tasks (triggers), cloud environments (network policy, setup script, environment variable names), enabled connectors and plugins, and the repo's own `.claude/` settings, which are already files |

**Coverage, kind by kind.** `standards/resources.toml` gains an `[iac]`
table that says which kinds are managed and which are exempt:

```toml
[iac]
managed = ["pages"]                 # kinds whose claims must come from IaC
[[iac.exempt]]
kind = "squarespace"
why = "Jim's own account, no API automation may touch."
review = "2027-05-26"
```

For a kind in `managed`, a resource counts as claimed only when the IaC
definitions define it, and **unmanaged** is an error. For every other kind
it is a warning, so the sweep reports the whole gap from day one without
going red over work nobody has done yet. Onboarding a kind means moving it
into IaC and adding it to `managed` in the same PR. RES-001 then applies to
IaC definitions as well: removing one needs a retirement entry.

**How each area is read.** All reads are read-only and free:

- **Cloudflare:** the existing read-only token, in the sweep.
- **GitHub:** the workflow's `GITHUB_TOKEN` reads most repo settings.
  Settings it can't read (rulesets details, secret names, webhooks and
  some code security settings need admin read) are read by the agent step
  with its own GitHub access, so no new token is needed.
- **Auth0:** the read-only `infra-read` application (`AUTH0_INFRA_READ_*`
  secrets), which the Infra Mint workflow creates
  ([decision 0011](../decisions/0011-mint-secrets-by-workflow.md)).
- **Claude:** no workflow can reach Claude's settings, so the weekly
  `human-issues-digest` agent step reads scheduled tasks and cloud
  environments with its Claude tools, compares them with the IaC
  definitions (or, until those exist, a plain snapshot), and reports
  unmanaged or drifted ones in the digest. Project instructions and
  connectors it can't read are `human-check` entries in
  `standards/settings.toml` (the eval engine's manifest), so Jim confirms
  them on a schedule.

**Snapshots as the first step.** Until a kind is onboarded to IaC, the sweep
writes what it read to `standards/live/<area>.json` (sorted, secrets never
included), and the agent step commits changes to it in a PR. That gives a
reviewable diff of every hand change, and the snapshot is the starting
input when a kind is onboarded to IaC.

## Phases

### Phase 1: claims, guards and a read-only sweep

- `standards/docs/resources.md` with the rules above (claims, retiring,
  safe order, temporary resources) and a short "renaming a resource"
  checklist; link it from `standards/docs/README.md` and `free-tiers.md`.
- `standards/resources.toml` (empty `[[keep]]` and `[[retired]]`), and
  `resources.sh check` with RES-001 to RES-003 in the hooks, `//:audit`
  and PR CI, tested in `jimhillr/workflow_tests`.
- PR CI step that labels PRs adding `[[retired]]` or `[[keep]]`
  `needs-human`.
- `resources.sh sweep` as a read-only job in `all-free-tier-usage.yml`.
- The `human-issues-digest` and `open-pull-request` skill steps above.
- A rule in AGENTS.md: "Renaming or removing a resource retires the old
  one in `standards/resources.toml` in the same PR."

### Phase 2: temporary resources

- `pull-request-closed-cleanup.yml` for PR caches.
- Preview deployment pruning and closed-PR branch deletion in a new
  `all-retire-resources.yml` (temporary resources only at this point).
- Move the branch-deletion logic from `all-delete-branches.yml` into
  `resources.sh` so the manual workflow and the sweep share it.

### Phase 3: approved deletion

- `all-retire-resources.yml` deletes `[[retired]]` entries in the safe
  order, with the guards and a dry run by default.
- First use: the `jimhillr-{project}` rename has merged, so
  `cahokia-web`, `pane` and `controlled-chaos` get retirement entries in
  a `needs-human` PR as soon as `standards/resources.toml` exists. Their
  deletion is already a human issue with a runbook
  (`delete-old-pages-projects.md`); if Jim follows it before this phase
  ships, the entries are added with `deleted` already set.
- Decision 0007 keeps deletes out of IaC. This workflow is the one
  approved delete path outside it: it deletes only entries Jim approved in
  a PR, with the deploy token's Pages edit permission, inside a workflow.
  Until it exists, deletions follow the human runbook.

### Phase 4: Auth0 and DNS

- A weekly read of Auth0 applications' callback, origin and logout URLs
  with the IaC work's read-only `infra-read` application (`read:clients`
  only; about 4 of the free plan's 1,000 machine tokens a month). Any URL not claimed by a
  `threat.toml` is an orphan; step 2 of the safe order checks it
  automatically instead of waiting on a human issue. The Infra Mint
  workflow (decision 0011) creates that application, so this plan adds no
  human issue for it. Once the `auth0` IaC stack manages the
  callback URLs, removing one is an IaC change instead of a dashboard step.
- Dangling DNS: once a zone exists, any record pointing at a `*.pages.dev`
  or `*.workers.dev` name we don't own is an error (the classic subdomain
  takeover).

### Phase 5: everything onboarded to IaC

- Add the `[iac]` table and the **unmanaged** and **drift** findings to the
  sweep, as warnings for every kind. Cover the GitHub repo settings in the
  table above, and add the Claude read to the digest's agent step.
- Write the first snapshots to `standards/live/`, and add `human-check`
  entries for the Claude and Auth0 settings no tool can read.
- As the IaC work onboards each kind, add it to `managed`, so unmanaged
  becomes an error for that kind. Done when every kind is managed or
  exempt with a reason.

The snapshot and warnings part doesn't wait for the IaC choice and can run
alongside phase 1; making a kind `managed` waits for it.

## Cost of this plan

Nothing. Everything runs on standard GitHub-hosted runners on a public repo
and reads the Cloudflare API with the existing token. Phase 4 uses a few
Auth0 machine tokens a month out of 1,000 free. No new scheduled Claude
task: the digest that already runs weekly does the agent step. Deleting old
projects frees Pages project slots rather than using them.
Part 6 adds read-only GitHub API calls inside the existing jobs, small
snapshot files, and a few minutes of the weekly digest's Claude run.

## Decisions

Jim, 2026-11-26: approved PR #669 and chose "yes to all" for the
recommended answers.

1. **Claims come from existing files** (`wrangler.toml`, `threat.toml`,
   the release matrix, open PRs), plus `standards/resources.toml` only for
   kept and retired resources. *Alternative:* one hand-kept inventory of
   everything, which drifts from the code.
2. **Approval to delete is approving the PR that retires the resource**,
   with 7 days of grace before the workflow deletes it. *Alternative:*
   also require a click on a protected GitHub environment for each
   deletion run (free, but one more thing to remember).
3. **A rename without a retirement fails CI from day one** (RES-001), and
   old addresses can't stay in the code (RES-002).
4. **Temporary resources are deleted automatically** (previews after 14
   days, closed-PR caches at close, closed-PR branches after 7 days);
   branches with no PR are only listed.
5. **Old `*.pages.dev` projects are deleted, not kept as redirect stubs,**
   after their Auth0 URLs are removed.
6. **One sweep:** this job is the threat check's weekly live check,
   widened, in `all-free-tier-usage.yml`.
7. **The weekly digest does the agent work**, rather than a new scheduled
   task.
8. **Jim turns on "Automatically delete head branches"** in the repo
   settings, and later creates the read-only Auth0 token (phase 4).
   (Update, 2026-11-27: both are now covered by the IaC work, decision
   0007. The `github` stack manages the branch setting, and the Infra Mint
   workflow (decision 0011) creates the read-only Auth0 application, so
   neither needs its own human issue.)

Proposed with Part 6, for Jim's approval:

9. **Unmanaged is a finding from day one**, as a warning for every kind,
   and becomes an error kind by kind as each one is onboarded.
   *Alternative:* wait for each IaC stack before reporting anything.
10. **Snapshots of live settings in `standards/live/`** until IaC exists,
    so every hand change shows up as a diff. *Alternative:* report only,
    with no committed record.
11. **Claude settings are read by the weekly digest's agent step**, and
    what it can't read becomes a `human-check` for Jim.

Cost of each: all three are $0 and use no new free-tier allowance (the
GitHub and Cloudflare reads are a few dozen API calls a week, far inside
their rate limits; snapshots are small JSON files). The Claude cost is
build effort, about four to six agent sessions for Part 6, plus a few
minutes a week inside the digest run that already happens. The
alternatives cost the same or less: waiting for IaC (9) or skipping
snapshots (10) saves one or two sessions now, but leaves hand changes
unseen until then.

## Not doing (yet)

- Cleaning Claude project files from a workflow: no API reaches them. The
  coordinator keeps them tidy. (Claude scheduled tasks and environments are
  covered by Part 6's agent step.)
- Building the IaC stacks, or applying changes with them: that is the
  IaC work in decision 0007. This plan only detects what isn't in IaC
  yet, and reports drift.
- PostHog: one project on the free plan, nothing to orphan. Its allowed
  hosts are listed as an inbound reference in the safe order.
- Squarespace: Jim's account, not touched by automation.
- Deleting Actions caches by age: GitHub already evicts caches unused for 7
  days, and the usage check warns at 8 GB.
