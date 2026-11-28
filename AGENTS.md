# Agent instructions

Repo-wide rules for any AI coding agent working in this monorepo. Task-specific
procedures live as [Agent Skills](https://agentskills.io) in `.agents/skills/`
(`.claude/skills` is a symlink to the same folder).

## Ground rules

- **No local clone required.** The repo is meant to be fully automated. Do
  maintenance through a GitHub workflow in `.github/workflows/` or an agent skill
  in `.agents/skills/`, never through a script someone has to run locally.
- **Standards are the source of truth.** Coding, testing, documentation, naming
  and workflow rules live in `standards/docs/`. If this file disagrees with
  them, the standards win; fix this file.
- **Work is tracked as one file per issue** in each project's
  `docs/issues/` folder (`<project>/docs/issues/`; repo-wide work goes in
  the platform project that owns it, `standards/docs/issues/` when no other
  fits), not in GitHub issues and not in `ISSUES.md` files, which are
  retired so parallel PRs stop conflicting (decision 0006). There is no
  root `issues/` folder (Jim, 2026-11-28: "Issues should live in the
  docs/issues directory in a given project"). Add an issue by adding a
  file, close it by deleting the file, and see the list with
  `mise run //:issues`; never commit a generated index. The format is in
  `standards/docs/issue-management.md`.
- **Tasks only a person can do** (dashboards, accounts, secrets, billing,
  sign-offs) go in the owning project's `docs/issues/human/`, one file
  each, never among its agent issues and never only in a chat thread.
  Agents don't work them. Pane's Human issues page gathers them from every
  project (Jim, 2026-11-28: "Pane should aggregate human issues from all
  the projects"), and a weekly Claude scheduled task, "Weekly human issues
  digest", reports them to Jim with the `human-issues-digest` skill
  (decision 0002).
- **Every human task has a runbook (Jim, 2026-11-27).** "Whenever asking
  for human help, document the requirement, why it can't be automated and
  create a run book. Link the run book in the task." Each
  `docs/issues/human/` file has `## Requirement`, `## Why it can't be automated`
  and `## Runbook` sections and a `runbook:` front matter field, and its
  runbook (steps, permissions, where secrets go, how to verify, cost) is
  in `standards/docs/runbooks/`, added in the same PR. Runbooks outlive
  their tasks, so every human task in the architecture, one-time ones
  included, stays documented. The audit enforces it (ISS-004); the format
  is under "Human issues" in `standards/docs/issue-management.md`.
- **Settings made by hand** (connectors, secrets, Auth0, Cloudflare tokens,
  billing) are listed in `standards/settings.toml`, never with a secret's
  value. A change that makes or needs one adds its entry in the same PR; the
  weekly digest asks Jim to re-check each one on its schedule ("Human
  checks" in `standards/docs/evals.md`).
- **Pull requests always open as drafts.** Mark a PR ready for review only once
  you judge it ready for human review. The `open-pull-request` skill has the
  full checklist. Don't hand a PR to a human (or present it as awaiting their
  review) until its checks are green and it is out of draft.
- **PR description sections (Jim, 2026-11-27).** "Every PR should have
  these sections: 1) What changed summary (50 words) 2) Why changed
  summary (50 words) 3) Cost changes (<25 words) 4) Risk changes (<25
  words) 5) Details (anything else agents want to include)." Above them,
  "the top most section should be 1) Attention (list of strongly enum'ed
  tags)", one of which is cost changed. This replaces the 75-word
  executive summary. So after any required attribution lines, every PR
  description has `## Attention` (the tags from `chores badges`: one
  requirements, one cost, one threat/risk and one docs tag from a closed
  set; Jim, 2026-11-28: "state if it is only docs being updated which
  includes plans", so a PR whose every file is docs says `docs-only` and goes to the
  `docs-reviewer`, Jim for now, decision 0019), `##
  What changed` (at most 50 words), `## Why` (at most 50), `## Cost
  changes` and `## Risk changes` (under 25 each, `None.` when nothing
  changes) and `## Details` (testing, review notes, links to committed
  plans, decisions and issues). Write it with `chores pr-body`. The format
  and tags are under "Pull request descriptions" in
  `standards/docs/pull-requests.md` and in
  `.github/pull_request_template.md`; the PR summary check (PR-001) warns
  on a missing section, an unknown tag or a summary over its limit.
- **Back to draft before working on a PR (Jim, 2026-11-26).** Before an agent
  starts any work on an existing PR (a fix, a review answer, a merge from
  `main`, any push), it first converts the PR to draft, and marks it ready for
  review again only once its checks are green and it meets the
  `open-pull-request` checklist. This applies to every PR an agent works on,
  including in PR triage.
- **Auto-merge on ready (Jim, 2026-11-26).** When an agent marks its PR
  ready for review (checklist met, checks green, nothing for a human to
  decide), it turns on GitHub auto-merge with squash, and turns it off
  whenever it moves the PR back to draft. Agents still never approve their
  own PRs, so the PR merges when a reviewer (normally Jim) approves it.
  Details are under "Auto-merge" in `standards/docs/pull-requests.md`.
- **Human review for red flags and cost (Jim, 2026-11-27).** "Lets make sure
  no prs with red attention flags get auto merged. Cost changes must be
  reviewed by jim", then "Lets make it needs human review". A PR with a red attention tag
  (`must-requirement-changed`) or `cost-changed`, in its diff or its
  description, or the `cost` label, is labelled `human-review` (PR-004,
  `chores gate`) until a human collaborator approves its head commit
  (Jim: "Any human collaborator is fine"); agent accounts (`-agt`),
  `jimhillr-reviewer`, bots and non-collaborators never count. It is not a
  CI check and posts no status, so CI shows only real results (Jim,
  2026-11-28). Agents and `jimhillr-reviewer` never approve it; an agent
  marks it ready and turns auto-merge on once its checks are green, like
  any agent PR, so the human's approval merges it (Jim, 2026-11-28: "Yes,
  merge on my approval"). Details are under
  "Human review" in `standards/docs/pull-requests.md`.
- **Plans, decisions and retros live in the repo**, not in chat tools or
  hosted docs. Write them as Markdown next to what they are about (a project's
  own folder, or `standards/docs/` for repo-wide work) and commit them with the
  related PR.

## Project rules and notes

This file is the home for project rules and memory. Keep them here, in the
repo, rather than in any one agent tool's settings.

- **Branch cleanup** is done with the `all-delete-branches.yml` workflow (dry
  run by default), not by deleting branches by hand or with a local script.
- **Pinned dependencies — do not bump:**
  - `dioxus` in `cahokia` is deliberately pinned to an older version.
  - `sycamore` in `sparkhill/website` and `profiles/jimhillr` must stay on
    0.8, which `perseus` 0.4.2 requires.
  - `terrars` in `infra` is pinned with `=`: its generated bindings in
    `infra/src/tfschema/` must be regenerated with the same version, so bump
    it only together with them (see `infra/README.md`).
- **Vendors (Jim, 2026-11-26).** When designing a system, use as few vendors
  as possible, but cost always comes first: if a non-approved vendor is
  clearly cheaper (including free tiers), cost wins over vendor count.
  - **Approved vendors:** GitHub (including Actions and Dependabot),
    Cloudflare and Squarespace. Reach for these first.
  - **Any other vendor needs Jim's sign-off.** Propose it in a decision record
    in `standards/docs/decisions/` (or the PR description for a small
    change), saying what it costs, why no approved vendor fits, and what a
    switch would take. Label the PR `needs-human` and don't merge it until Jim
    approves.
  - **Approved exceptions:**
    - PostHog, for real-user session replay (decision 0001).
    - Auth0, used by `auth_sdk` and `cahokia` for sign-in.
    - RapidAPI (Zillow data), used by `homer`, **only on its free plan**. It
      must never cost money: don't add a paid plan, scheduled runs or anything
      else that could push usage past the free quota.
    - Anthropic's Claude Code, our AI coding platform (Jim, 2026-11-26).
      It is the one paid service, with a budget of $25 a month.
  - **Prefer what Claude supports (Jim, 2026-11-26).** After cost, prefer
    technologies Claude knows well and vendors with a Claude Code plugin or
    MCP server, enabled read-only for context. The tech radar notes which
    have one.
  - **Cost in every decision question (Jim, 2026-11-26).** "Always include
    cost in a question like this." Every decision put to Jim (a decision
    card, an option list, a decision record) gives each option's money cost
    (usually $0 on the free tier), how close it gets to its free-tier
    limits ([standards/docs/free-tiers.md](standards/docs/free-tiers.md)
    and the budgets in the tech radar), and the Claude Code build effort,
    which counts against the $25 a month budget.
  - **Tech radar.** [standards/docs/tech-radar.md](standards/docs/tech-radar.md)
    lists every approved, evaluated and held technology and vendor, with its
    exemplar projects. A PR that approves, rejects or drops one, or adds the
    first project to use one, updates `standards/radar/radar.json` in the
    same PR. Every vendor there says how its billing is checked (Jim,
    2026-11-27: "the billing check should check every vendor in the tech
    radar"): a read-only API check in `standards/scripts/billing-check.sh`
    where the vendor allows it, otherwise a settings check for Jim; the
    radar check fails an approved vendor with neither (RAD-007).
- **Free tiers only (Jim, 2026-11-26).** Every service is used on its free
  plan, except Claude Code (budget $25 a month). Each technology's monthly
  budget is in the tech radar; only Jim sets or raises one. Never turn on a paid plan, add a payment method or raise a spending
  or cache limit, and when a limit is near, stop rather than pay. Anything
  automation creates on demand goes through `standards/scripts/free-tiers.sh`.
  Limits, budgets and guards are in
  [standards/docs/free-tiers.md](standards/docs/free-tiers.md), with the
  numbers in `standards/radar/limits.toml`. Every project keeps its expected
  usage and money in `docs/cost.toml`, checked by `cost check`
  ([standards/docs/cost.md](standards/docs/cost.md)); update it in any PR
  that adds a service, a deploy or a cost.
- **No secrets in git or state (Jim, 2026-11-26).** "Never commit secrets
  to git or store them in tf state." No secret value (token, password,
  client secret, key) is ever committed, and none may be in OpenTofu
  state, not even encrypted. OpenTofu never generates or copies a
  secret, and the `infra` plan guard IAC-002 fails any plan that would
  put a sensitive value in state. Secrets are stored only as GitHub or
  vendor secrets. A person makes only the bootstrap credentials in
  `infra-credentials`; the Infra Mint workflow makes every other infra
  secret, piping each value straight into a GitHub secret so no one sees
  it (Jim, 2026-11-27,
  [decision 0011](standards/docs/decisions/0011-mint-secrets-by-workflow.md)).
  A new credential is added to `infra/scripts/mint.sh`, not to a human
  task. Every long-lived secret has a rotation cadence and owner in
  [standards/docs/secret-rotation.md](standards/docs/secret-rotation.md)
  (Jim, 2026-11-27): Infra Mint rotates what it makes every quarter, and
  a settings check reminds Jim of the rest. A PR that adds a long-lived
  secret adds its row there.
- **Second approval for credentials (Jim, 2026-11-26).** A credential
  change (new or edited Auth0 apps and grants, token scopes, Access
  policies, GitHub secrets, environments or rulesets) applies only after
  Jim approves its run in the `infra-credentials` GitHub environment, on
  top of the PR approval
  ([decision 0010](standards/docs/decisions/0010-credential-second-approval.md)).
  Agents never approve a `credential-change` PR, never put a
  credential-capable token anywhere but that environment, and never ask
  for admin rights for `jimhillr-agt` or the Claude GitHub App.
- **Evals are continuous improvement (Jim, 2026-11-27).** "The eval
  process is a continuous improvement process and should not be considered
  a one off project. The evals should always be green and a system of
  feedback to our todo process should be the way in which individual
  projects improve their eval coverage." So every requirement's verdict
  should be green; any requirement that is not met, at risk or unknown
  becomes a `FIX` or `TODO` issue file in its project's `docs/issues/`, and the
  normal agent loop works it. Each project grows its own eval coverage
  through those issues: requirements for anything it ships, evals that
  check real content in production, and an eval coverage figure and badge
  per project. Agents working on a project add or fix evals as part of the
  work, not as a separate phase. Details are under "Continuous
  improvement" in [standards/docs/evals.md](standards/docs/evals.md).
- **Code quality ratchet (Jim, 2026-11-27).** "We also need a process for
  ensuring maintainable code base." Each project's quality findings (clippy
  pedantic and the complexity limits) may only go down, never up, and gaps
  become issue files through weekly quality triage, the same loop as evals
  ([decision 0015](standards/docs/decisions/0015-code-quality-process.md),
  [plan](standards/docs/plans/2026-11-code-quality.md)). Leave no new
  findings in files you touch. Scripts stay Rust-only: add no new `.sh`
  files, and port existing ones into `chores` commands over time.
- **GAAP books (Jim, 2026-11-27).** "All of our cost software should
  follow GAP accounting so that everything is double counted on the ledger
  and also we should have attribution in such a way that if you sum up all
  service costs/revenue it equals total revenue/costs." Cost, money and
  ledger code is double-entry (every entry's postings sum to zero), books
  costs in the months they serve, and attributes every cent to exactly one
  project by a stated rule, so the projects sum to the totals with no
  remainder. The rules are under "Accounting rules" in
  [standards/docs/cost.md](standards/docs/cost.md)
  ([decision 0016](standards/docs/decisions/0016-holding-and-handling-money.md)).
- **Products are separate companies (Jim, 2026-11-28).** "Assume
  products are owned by different companies during architecture. We
  should have shared technical platform services, just libraries and
  patterns." Each product runs its own endpoints on its own origin, owns
  its data and answers its own users; the technical platform shares crates,
  patterns and infrastructure definitions, not services a product's
  requests pass through. Shared database infrastructure is allowed only
  with tenant-scoped tables that one export can move out
  ([decision 0021](standards/docs/decisions/0021-tenant-scoped-product-data.md)).
- **Files before databases (Jim, 2026-11-26).** Avoid databases for as long
  as possible. Dev work, results and data (eval results, reports, fixtures,
  scraped data) are files committed to the monorepo: Markdown, JSON, JSON
  Lines, CSV or TOML, next to the project that owns them. When a project
  really needs a database in the repo (queries, joins or indexes over the
  data), use SQLite committed to the repo: prefer a `.sql` text dump that
  builds the database (it diffs and reviews like code), and commit the
  `.sqlite` file itself only when it is small. Rust code reads it with
  `rusqlite` and its `bundled` feature. A hosted database is a last resort,
  for data a deployed service must write at runtime; which one, and when, is
  decided in [decision 0004](standards/docs/decisions/0004-database-engine.md).
- **User docs in every project (Jim, 2026-11-27).** "All projects should
  have user docs for supporting our end users." Every top-level project
  has a user guide at `<project>/docs/user/README.md` with `## Who it's
  for`, `## Getting started`, `## How to ...` and `## Getting help`
  sections ("User docs" in
  [standards/docs/documentation.md](standards/docs/documentation.md#user-docs),
  [decision 0017](standards/docs/decisions/0017-user-docs.md)). The audit
  (`chores user-docs`, DOC-005 to DOC-008) fails a project without one
  unless it is in the shrinking baseline `standards/user-docs.toml`. A PR
  that changes what users see or do updates the guide, and a new project
  ships with one.
- **Third-party content in athenaeum (Jim, 2026-11-27).** "Including
  copywrited content is fine as long as we reference the original owner
  properly and never sell or expose it as a part of our product." Copies of
  others' work may live in `athenaeum`, which is for agents and people to
  understand our coding stance, with the owner named and the original
  linked beside them; never ship them in a product. Details are in
  [athenaeum/CONTRIBUTING.md](athenaeum/CONTRIBUTING.md).
- **Code quality references in athenaeum (Jim, 2026-11-27).** "We can use
  athenaeum project as a store for important documents/blog posts/patterns
  we discover so we can reference them as we write software." Before
  leaning on a design or code quality idea, read its note in
  [athenaeum/resources/code_quality/](athenaeum/resources/code_quality/README.md),
  and when a source changes how we write code here, add a note for it
  there. The standards still hold the rules; the notes explain them.
- **Cloudflare Pages names (Jim, 2026-11-26).** Every Pages project is
  named `jimhillr-<project>` (served at `jimhillr-<project>.pages.dev`), so
  it never collides with another account's `*.pages.dev` name. The audit
  enforces it (FT-006); details in
  [standards/docs/naming.md](standards/docs/naming.md#cloudflare-pages-projects).
- **Name things `jimhillr`, not `monorepo` (Jim, 2026-11-27).** "Lets
  not use the nomenclature monorepo anywhere in our implementations. Use
  jimhillr instead." New identifiers (headers, user agents, meta tags,
  token and key labels, resource names, crates, folders) use `jimhillr`,
  such as `x-jimhillr-synthetic` and the `jimhillr-evals` user agent. The
  only exceptions are names we don't own: the GitHub repository
  `jimhillr/monorepo` itself and mise's `monorepo_root` setting and
  `[monorepo]` table. The repo-tooling project that was `monorepo/` is now
  `jimhillr/`. Prose may still call the repository a monorepo.
- **Labels.** A PR that needs a person (Jim's sign-off, a manual step, an
  unsafe change) gets the `needs-human` label (Jim, 2026-11-26; it replaces
  `needs-jim`). PR CI's **Cost summary** job adds `cost` to any PR that
  changes what the repo uses or costs, and `needs-human` when it adds a
  vendor or money or raises a budget (standards/docs/cost.md). The
  Human review workflow adds `human-review` to a PR waiting for a human's
  approval and takes it off once one approves.
- **Retire what you rename (Jim, 2026-11-26).** Renaming or removing a
  resource (a Pages project, Worker, binding, URL, release) retires the old
  one in `standards/resources.toml` in the same PR, with Jim's approval
  (`needs-human`). Nothing is deleted just because it is unclaimed; the
  rules and the safe deletion order are in
  [standards/docs/resources.md](standards/docs/resources.md).
- **No project templates.** The old `example/` templates are intentionally
  gone and must not be restored. Create new projects by hand.
- **Merge rule.** Any agent may squash-merge any PR, including one its own
  thread opened, once it is mergeable: not a draft, no merge conflict, at
  least one approving review on the current head commit and no outstanding
  "changes requested", and green checks. Green means every check has finished
  and passed, except a check that is also failing on the latest `main` commit
  for reasons the PR didn't cause. A pending check is never green. Agents
  still never approve their own PRs. Details are in
  `standards/docs/pull-requests.md`.
- **The reviewer account (Jim, 2026-11-27).** "A third GitHub account to
  do approvals so we can be explicit about what is being auto-approved and
  establish a process for that approver." `jimhillr-reviewer` approves
  only the green, unlabelled, low-risk agent PRs its written rules allow
  (the `All Auto Review` workflow, "The reviewer account" in
  `standards/docs/pull-requests.md`, decision 0014); Jim approves the
  rest, and reads every merged PR afterwards on pane's Acknowledge page.
  Agents never hold or ask for its token, never remove a red or orange
  label to get a PR approved, and never split a change to dodge its
  protected paths or size limit.
- **PR lifecycle.** Every PR, from any author, is driven to closure (merged
  or closed). The policy is in `standards/docs/pull-requests.md`. A scheduled
  agent task, "PR triage", runs three times a day as `jimhillr-agt` on
  every open PR: it approves safe dependabot and human PRs, squash-merges every
  mergeable PR (per the merge rule above), closes superseded or pinned bumps
  (and conflicting ones Dependabot can't rebase, so it recreates them), adopts
  orphaned agent PRs, turns on auto-merge for ready agent PRs, hands bumps that break the build to an agent to fix, and
  labels only real decisions for a person `needs-human`. Each run also
  checks the README's repo health badges and tries to fix what is red, at
  most one fix PR per run ([standards/docs/repo-health.md](standards/docs/repo-health.md)). Safe to approve means
  green checks, no conflict, never the pinned bumps above, no auth or secrets
  changes unless trivially safe, and for a major bump (a 0.x minor bump counts
  as major) release notes with no breaking change that CI wouldn't catch
  (Jim, 2026-11-26). It reads this file for extra pins, so list
  any new pins here.
- **Pause on Claude overage (Jim, 2026-11-27).** When any agent session
  reports overage usage (`rate_limit_info.isUsingOverage` from the session
  API), the coordinating agent starts no new threads until the usage
  resets, and PR triage reports it to Jim. Running threads finish their
  current step. This overrides "keep threads working". The tracking
  process is in
  [standards/docs/plans/2026-11-cost-tracking.md](standards/docs/plans/2026-11-cost-tracking.md).
- **Never idle without saying why (Jim, 2026-11-26).** Before an agent's
  thread goes quiet, it posts a short message saying what it is waiting on
  (a hook or CI run, a review, an answer) and roughly when it expects to
  continue. A thread with nothing running is either resolved as done or
  asks Jim a question that says why it can't close, so it shows as waiting
  on him. A checklist update alone doesn't count.
- **Credit window (Jim, 2026-11-27).** "Kick off work as our credit
  window begins to close to maximize our use of the 5 hr credits. Have
  that process run all the time." The hourly "Credit window check"
  scheduled task (Jim, 2026-11-28: a scheduled task, not a standing
  thread) starts a short run each hour that reads the
  five-hour and seven-day usage from the sessions' rate-limit events and
  asks the coordinating agent to start extra threads when the five-hour window would otherwise end under 85%
  used, aiming at 95% without running the seven-day window out early.
  Keeping at least 5 threads working is its floor, and the overage pause
  still wins. The
  rule is [decision 0013](standards/docs/decisions/0013-credit-window-scheduler.md),
  the prompt is in
  [standards/docs/scheduled-tasks/credit-window.md](standards/docs/scheduled-tasks/credit-window.md).
- **Product share (Jim, 2026-11-28).** "I'd like to ensure that at least
  50% of all of my claude credits are being used on product work, not to
  let the platform work overwhelm it." A soft heuristic: when new threads
  start, `chores next-task --working <one project per working thread>`
  prefers product issues while products (decision 0019's `product` tier)
  are under half of the working threads and last week's merged PRs. Pass
  `--working` whenever you pick for new threads. Details are under
  "Product share" in
  [decision 0013](standards/docs/decisions/0013-credit-window-scheduler.md).
- **Starting new work.** Apart from the credit window check above, there
  is no scheduled job for new work. Whenever no agent PRs or agent tasks
  are in flight, the coordinating agent starts a new
  task by running the `start-a-new-task` skill.
- **CI on `main` may be red.** Judge a PR by whether the projects it changes
  pass, not by the overall status.

## Dev commands

Each project's build, test and CI steps run through the repo's task runner.
Run the CI task for every project you changed and make it pass before
opening or readying a PR.

Git hooks do that for you: once set up, every commit and push runs
`mise run //:audit` and the CI task in each affected project, the same checks
PR CI runs. Set them up at the start of every session, before your first
commit, and never skip them (`LEFTHOOK=0`, `--no-verify`) to get a change
through. If a hook fails, fix the failure and commit again. The only
exception is a check that fails because a cloud sandbox can't install a tool;
[standards/docs/git-hooks.md](standards/docs/git-hooks.md) has the fallback,
and the PR must say which checks didn't run.

- Set up git hooks (once per clone): `mise run //:hooks`
- Run the hook checks without committing: `bash standards/scripts/hooks.sh pre-push`
- Install project tooling: `mise install` (in the project directory)
- Run CI for a project (fmt, clippy, build, tests, and npm tests where a
  `package.json` exists): `mise run ci`
- Run only tests: `mise run test`
- Fix formatting and lints: `mise run fix`
- List the project's tasks: `mise tasks`
- List the open issues: `mise run //:issues` (`-- --human` for human tasks)
- Check the repo standards: `mise run //:audit`
- Routine agent work (pick the next issue, write a PR description, write
  an issue file): `cargo run --quiet --manifest-path chores/Cargo.toml --
  <command>`; the commands are in [chores/README.md](chores/README.md).
  When you notice routine work a script could do, add a command there
  rather than repeating it by hand
  ([baked skills plan](standards/docs/plans/2026-11-baked-skills.md)).

Skills refer to this section instead of naming commands directly, so when the
task runner changes, only this section needs updating.

Every project has the same tasks (`ci`, `fix`, `build`, `test`, `dev`,
`release`, `deploy`, `evals`), from one shared file in `standards/tasks/`.
The rules and decisions (no root Rust workspace, no committed build
artifacts) are in [standards/docs/mise.md](standards/docs/mise.md). CI runs
`mise run ci` and then `mise run evals` in each changed project; pushes to
`main` also run `mise run deploy`.

## Skills

| Skill | Use it to |
| --- | --- |
| `start-a-new-task` | Pick one issue from the projects' `docs/issues/` folders and complete it end to end. |
| `code-review` | Review one project against the standards and record five TODOs. |
| `human-issues-digest` | Report every project's open `docs/issues/human/` tasks and `needs-human` PRs to Jim. |
| `open-pull-request` | Branch, commit, open a draft PR, and mark it ready when it is. |
| `write-issue` | Write an issue file or a human task with its runbook through the `chores` command. |
| `eval-triage` | On demand, for the eval calls the daily triage workflow can't make: rollback advice for a deploy that failed its evals, and an eval Jim disputes. |
| `deep-project-review` | Review the next project in the rotation in depth and deliver a Marp slide deck in `docs/reviews/` for Jim. Run weekly by the "Weekly deep project review" scheduled task. |

New skills go in `.agents/skills/<name>/SKILL.md` and must follow the
[Agent Skills specification](https://agentskills.io/specification). The
`Agent Skills` workflow validates them on every pull request.
