# 0019. Review roles: who may approve what, as relationships in one file

- Status: Accepted
- Decided by: Jim (answers in this thread and one decision card, 2026-11-27, logged under "Jim's answers")
- Date: 2026-11-27
- Asked by: Jim
- Builds on: [0010](0010-credential-second-approval.md) (the second
  approval for credentials) and [0014](0014-faster-pr-approvals.md) (the
  reviewer account and the `Human review` check)

## Context

Jim, 2026-11-27: "This conversation makes me think we need a more formal
ReBAC solution for our processes. We should have three roles.
Credential-reviewer, critical-reviewer, standard-reviewer or something
like that and have the ability to include different users in those groups.
Right now I would be the only credential reviewer, I would be the only
critical-reviewer, and any collaborator on the repo is a
standard-reviewer. It might make sense to do the roles based on domains..
or the things in the red flags such as cost, risk/threats, requirements."

Who may approve what is decided today in six places that each spell it out
their own way:

| Rule | Who may approve | Where it is written |
| --- | --- | --- |
| A green, low-risk agent PR | `jimhillr-reviewer` | Lists in `standards/scripts/auto-review.sh` |
| A red attention tag or a cost change | Any human collaborator | `is_human` and `is_collaborator` in `chores/src/gate.rs` |
| A `credential-change` PR | Jim | Prose in AGENTS.md and the triage prompt |
| A credential apply and Infra Mint | Jim | The `infra-credentials` environment's reviewer list, set by hand |
| A safe Dependabot or human PR | PR triage (`jimhillr-agt`) | The triage prompt |
| A new vendor, money, a raised budget, a retired resource | Jim | `needs-human` label and prose |

The word "Jim" is hard-coded in most of them. Adding a second person, or
letting someone own only cost, would mean editing code, prose, a prompt and
a GitHub setting, and nothing checks that they agree.

## What ReBAC means here

Relationship-based access control (Google's Zanzibar is the reference
design) answers "may this subject do this to this object?" from stored
relationships, not from rules written into code. It has three parts:

- **Tuples**: facts of the form *subject* has *relation* to *object*, such
  as "`jimhillr` is `cost-reviewer` of the repo".
- **Rewrites**: rules that derive one relation from another, such as "a
  `cost-reviewer` of the repo is `cost-reviewer` of every project in it".
- **A check**: a pure function over the tuples and rewrites.

We need the model, not a server. Our tuples number in the tens, change a
few times a year, and must be reviewed like code. So they belong in a file
in the repo (files before databases), read by one small Rust function.

## The model: a matrix

Jim, 2026-11-27: "I am considering if this is actually a matrix. We have
lots of domains such as cost, requirements, quality, credential, threat,
etc... and then we have project tiers as an orthogonal concern." It is.
A change has two independent properties:

- its **domain**: what kind of thing changed (cost, a requirement, a
  credential);
- its **tier**: where it changed (which layer of the repo the project
  belongs to).

A reviewer role belongs to exactly one domain ("All reviewer roles should
map directly to some specific domain"), and a person holds it **over a
scope**: the whole repo, one tier, or one project. That scope is what
makes this relationship-based and not a flat list of roles.

```text
subject ──<domain>-reviewer──▶ scope            pull request ──touches──▶ (domain, project)
(account)                      repo ⊃ tier ⊃ project
```

The check: a PR may merge when, for every (domain, project) cell it
touches, someone who holds that domain's role over a scope containing the
project has approved its head commit (or opened the PR), and none of them
has asked for changes. One approval counts for every role its author
holds, so today one click from Jim covers every cell.

Roles don't include each other (Jim: "No but one review from me should
satisfy both roles"). Scopes do contain each other: a grant over the repo
covers every tier, and a grant over a tier covers its projects. So "Jim is
`cost-reviewer` everywhere" is one line, and so is "Dana is
`cost-reviewer` for products" or "Sam is `change-reviewer` for
`cookbook`".

### Tiers

Jim, 2026-11-27: "Lets introduce somewhere in our processes the concept of
technical platform versus products. Every project of codebase should live
in one of these two categories", then: "we need multiple layers.
Software-development-platform for all the things related to how we make
software such as pane, athanaeum, standards. Then technical-platform for
reusable components such as auth-sdk, design-system. These have lower
risk. We want strict protections on the software-development-platform as
changes to that can result in changing how we make code and how we protect
and guard things."

| Tier | What it is | Protection | Projects |
| --- | --- | --- | --- |
| `software-development-platform` | How we make, check, deploy and guard software, and the rules agents follow | Strict | `standards`, `chores`, `pane`, `athenaeum`, `evals`, `cost`, `threat`, `infra`, `tofu_state`, `jimhillr`, `issues`, `docs`, and the root files and dot folders (`AGENTS.md`, `.github/`, `.agents/`, `mise.toml`) |
| `technical-platform` | Reusable components products are built from | Reviewed | `auth_sdk`, `design_system`, `telemetry`, `royalty` (Jim: "even though we may someday release it") |
| `product` | What an end user uses | Fast lane | `bear_down`, `blueeel`, `bogan`, `cahokia`, `controlled_chaos`, `cookbook`, `homer`, `luggage`, `my_lang`, `profiles`, `psy_ents`, `sparkhill`, `starcraft` |

Every top-level folder is in exactly one tier, listed in
`standards/reviewers.toml`; the audit fails a folder in none or in two
(REV-002). Other processes read the same list: the deep review rotation,
cost attribution, the tech radar and pane's project list.

**Strict** means: never auto-approved, never auto-merged, and the policy
file, the check and its workflow are in this tier, so changing the guard
needs the strictest review.

### Domains

Each domain has one role, a trigger the tooling can detect, and no
judgment about severity hidden in the role's name (Jim: "critical-reviewer
is too generic and not meaningful").

| Domain | Role | A PR touches it when |
| --- | --- | --- |
| `change` | `change-reviewer`, named per tier: `software-development-platform-reviewer`, `technical-platform-reviewer`, `product-reviewer` | It changes any file in a project |
| `requirements` | `requirements-reviewer` | Any `docs/requirements.md` changes (the `requirement-changed` and `must-requirement-changed` tags) |
| `cost` | `cost-reviewer` | The `cost-changed` tag or the `cost` label |
| `threat` | `threat-reviewer` | The `threat-risk-changed` tag or the `security` label |
| `quality` | `quality-reviewer` | A quality baseline goes up or a lint or complexity limit is loosened (decision 0015). Jim: "fine for now"; other threads are working out what quality means, and this trigger follows them |
| `credential` | `credential-reviewer` | IaC adds or changes something that creates a credential: the `credential-change` label from the plan guard (IAC-005) |
| `minting` | `minting-reviewer` | It changes something that can mint other credentials: `infra/scripts/mint.sh`, `infra-mint.yml`, the bootstrap credentials or the environment that holds them |

The two credential roles are separate on purpose (Jim: "clearly delineate
between the two types of credential reviewers"). A `credential-reviewer`
approves one new credential. A `minting-reviewer` approves the machinery
that can make any credential, which is the higher power. They also guard
two different runs: credential applies, and Infra Mint (see "Where it is
enforced").

The three tier-named roles are the `change` domain's role at each tier.
Jim named them, so the file keeps those names as shorthand for
"`change-reviewer` over that tier".

### The matrix today

Who must approve each cell. "Jim" means the role's only member is Jim;
every domain role is held over the whole repo for now.

| Domain | `software-development-platform` | `technical-platform` | `product` |
| --- | --- | --- | --- |
| `change` | Jim (strict) | Jim | `auto` when green and small, else any human collaborator |
| `requirements` | Jim | Jim | Jim |
| `cost` | Jim | Jim | Jim |
| `threat` | Jim | Jim | Jim |
| `quality` | Jim | Jim | Jim |
| `credential` | Jim | Jim | Jim |
| `minting` | Jim | not applicable | not applicable |

The matrix is uniform across tiers today except for the `change` row. It
earns its keep when a second person arrives: they can be given one cell
(`cost` for products), one row (`requirements` everywhere) or one column
(`change` for `technical-platform`) with one line each.

### Fixed rules the file can't change

Some rules must hold however the file is edited, so they are in code and
tested, not in the policy:

- An agent (`-agt`), a bot or `jimhillr-reviewer` never holds a human
  role, even if a line in the file names it.
- `auto` can only satisfy the `change` domain, and only in a tier whose
  protection allows it. No other domain is ever auto-approved.
- The repo owner holds every role over the whole repo, so a bad edit
  can't lock everyone out.
- A change to the policy file needs every role it edits, on top of the
  `software-development-platform-reviewer`. Changing who is a
  `minting-reviewer` needs a `minting-reviewer`.
- The check reads the policy from the base branch, so a PR can't grant
  itself an approver.

## The policy file

`standards/reviewers.toml`, in the strict tier. This is the whole
starting state.

```toml
# Who may approve what (decision 0019). Read by `chores gate`.
# One approval counts for every role its author holds.

# Every top-level folder is in exactly one tier (REV-002).
[tiers.software-development-platform]
protection = "strict"          # never auto-approved or auto-merged
projects = ["standards", "chores", "pane", "athenaeum", "evals", "cost",
  "threat", "infra", "tofu_state", "jimhillr", "issues", "docs"]
root = true                    # root files and dot folders belong here

[tiers.technical-platform]
protection = "reviewed"        # a human always approves
projects = ["auth_sdk", "design_system", "telemetry", "royalty"]

[tiers.product]
protection = "fast"            # `auto` may approve a plain change
projects = ["bear_down", "blueeel", "bogan", "cahokia", "controlled_chaos",
  "cookbook", "homer", "luggage", "my_lang", "profiles", "psy_ents",
  "sparkhill", "starcraft"]

# Grants: who holds each domain's role, and over what.
# A scope is "repo", "tier:<name>" or "project:<name>".
[[grants]]
role = "software-development-platform-reviewer"   # change-reviewer over that tier
members = ["jimhillr"]
scope = "tier:software-development-platform"

[[grants]]
role = "technical-platform-reviewer"
members = ["jimhillr"]
scope = "tier:technical-platform"

[[grants]]
role = "product-reviewer"
members = ["github:collaborators"]
scope = "tier:product"

[[grants]]
role = "requirements-reviewer"
members = ["jimhillr"]
scope = "repo"

[[grants]]
role = "cost-reviewer"
members = ["jimhillr"]
scope = "repo"

[[grants]]
role = "threat-reviewer"
members = ["jimhillr"]
scope = "repo"

[[grants]]
role = "quality-reviewer"
members = ["jimhillr"]
scope = "repo"

[[grants]]
role = "credential-reviewer"
members = ["jimhillr"]
scope = "repo"
environments = ["infra-credentials"]   # its required reviewers must equal these members

[[grants]]
role = "minting-reviewer"
members = ["jimhillr"]
scope = "repo"
environments = ["infra-mint"]

[auto]
members = ["jimhillr-reviewer"]   # decision 0014; size limit and green checks still apply
```

What changes from today's behavior:

- Cost changes go back to needing Jim (they were "any human collaborator"
  for a few hours).
- Threat and risk changes and every requirement change need Jim. Today
  threat changes are only kept away from `jimhillr-reviewer`, and only
  `must` requirement changes are held.
- Every change outside the product tier needs Jim (his decision card,
  2026-11-27: "Every platform change"). Today `jimhillr-reviewer` may
  approve small green changes in `chores`, `evals`, `cost`, `pane`,
  `telemetry`, `design_system` and `athenaeum`.
- Infra Mint moves to its own `infra-mint` environment, so minting and
  credential applies can have different reviewers.

## Where it is enforced

One decision function, several readers. Nothing else keeps its own list.

| Reader | Today | With this decision |
| --- | --- | --- |
| `Human review` check (`chores gate`) | Hard-coded tags and "any human collaborator" | Reads the file; reports each touched cell (domain and project), the role it needs and who satisfied it. Renamed `Review roles` |
| `jimhillr-reviewer` (`auto-review.sh`) | Its own label, tag and path lists | Approves only when `Review roles` says `auto` satisfies every touched cell, which only a plain product change can. Size limit and green checks stay |
| `main` ruleset | Requires `CI result` and `Human review` | Requires `CI result` and `Review roles` |
| `infra-credentials` and `infra-mint` environments | One environment, Jim, set by hand | Two environments, still set by hand (agents must not hold that power): credential applies wait for a `credential-reviewer`, Infra Mint for a `minting-reviewer`. The settings check and infra drift compare each environment's reviewers with its role and report a mismatch |
| PR triage prompt | Prose rules naming Jim | "Never approve what `Review roles` holds; merge when it is green" |
| Pane's Approve page | Labels and tags | Shows "needs cost-reviewer, threat-reviewer" and who can give it |
| Labels (`needs-human`, `cost`, `security`, `credential-change`) | Each set by its own job | Kept as inputs and as what people filter by. `needs-human` is set whenever a cell needs a human and has no approval yet |

GitHub's own approval count stays at one. The `Review roles` check is what
says whether the right person gave it.

## Options

Money is $0 for every option: all of it runs in GitHub Actions on a public
repo, with no new vendor. Build effort is Claude Code threads, which count
against the $25 a month budget.

| # | Option | Money | Free-tier use | Build effort | Verdict |
| --- | --- | --- | --- | --- | --- |
| 1 | Keep rules hard-coded, add cases as they come | $0 | None | None now, a code and prose change per new person or rule | Rejected: six places already disagree in wording |
| 2 | **Policy file in the repo, checked by `chores gate`** | $0 | One short Actions job per PR event (already running) | About 3 threads | **Recommended** |
| 3 | GitHub native: CODEOWNERS, teams and "require code owner review" | $0 | None | About 1 thread | Rejected: teams need an organization and this repo is under a user account; CODEOWNERS matches paths only, so it can't tell a `must` requirement from a `could`, or read a label |
| 4 | A ReBAC engine (OpenFGA, SpiceDB) | $0 self-hosted, but it needs somewhere to run | A Worker or container plus a database | About 6 threads and a standing service | Rejected: a service and a database for tens of tuples, against "files before databases" and "fewest vendors". The file keeps the same model, so moving later is a translation, not a redesign |

## Plan

Each phase is one PR, and each leaves behavior the same or stricter.

1. **Model, file and tiers.** `standards/reviewers.toml` with the tiers
   and grants, a `review_roles` module in `chores` (parse, fixed rules,
   check), and `chores gate` reading it for `requirements` and `cost`.
   Audit rules REV-001 (a file that breaks a fixed rule or names an
   unknown role or scope) and REV-002 (a folder in no tier or in two).
   AGENTS.md and `REPOSITORY_STRUCTURE.md` gain the tiers.
2. **All domains.** Add `change`, `threat`, `quality`, `credential` and
   `minting` to the check. Rename it `Review roles`. `auto-review.sh`
   drops its own lists and follows the check.
3. **Outside GitHub's reach.** The `infra-mint` environment (a human task
   with a runbook, since only Jim can make it), and a settings check and
   drift report for both environments' reviewers. Pane shows the needed role. The triage
   prompt, AGENTS.md and `pull-requests.md` point at the file and drop
   the names.
4. **When a second person joins.** Add a grant with their login and a
   scope. No code.

## Risks

- **A wrong file locks PRs or opens them.** The fixed rules, REV-001, the
  base-branch read and the owner fallback bound both failures.
- **The check is repo code.** A PR could change `chores` or the workflow.
  Those paths are in the strict tier, the check builds from the base branch, and
  credentials keep their second gate (the environment) outside the repo,
  as decision 0010 wanted.
- **`github:collaborators` is whoever GitHub says.** Adding a collaborator
  with push access makes them a `product-reviewer`. That is the intent,
  and the settings check lists collaborators for Jim on its schedule.

## Jim's answers (2026-11-27)

First round:

1. *Does a credential reviewer count as a critical reviewer, and so on?*
   "No but one review from me should satisfy both roles."
2. *What does a cost change need?* "Cost-reviewer.. again me."
3. *What do rule changes need?* "Lets make this a
   technical-platform-reviewer and make the standard one a
   product-reviewer. A technical-platform-reviewer should be just me for
   now. Lets introduce somewhere in our processes the concept of technical
   platform versus products. Every project of codebase should live in one
   of these two categories."
4. *Should a threat or risk change need a human?* "Always. We should have
   a threat-reviewer and have it be just me."

Second round:

1. *Does every platform change need its reviewer, or only rule changes?*
   "Every platform change."
2. *Is the platform and product split right?* "I actually think we need
   multiple layers. Software-development-platform for all the things
   related to how we make software such as pane, athanaeum, standards.
   Then technical-platform for reusable components such as auth-sdk,
   design-system. These have lower risk. We want strict protections on the
   software-development-platform as changes to that can result in changing
   how we make code and how we protect and guard things."
3. *Does `critical-reviewer` stay?* "I think requirements should just be a
   requirments-reviewer and should be strict. The concept of
   critical-reviewer is too generic and not meaningful. All reviewer roles
   should make directly to some specific domain."
4. "After answering I am considering if this is actually a matrix. We have
   lots of domains such as cost, requirements, quality, credential,
   threat, etc... and then we have project tiers as an orthogonal
   concern."
5. "We should clearly delininate between the two types of credential
   reviewers. 1) Some credentials can mint other credentials
   (minting-reviewer maybe?) and a crednential-reviewer when some IaC has
   been added that creates credentials."

Third round:

1. *Are the tier placements right?* "royalty should be technical platform
   even though we may someday release it." The rest stand.
2. *Should Infra Mint get its own `infra-mint` environment?* "Yes clear
   lines here are probably good."
3. *What counts as a quality change (a raised baseline or a loosened
   limit)?* "That's fine for now. There are other threads working through
   what software quality means."

## Consequences

- The build is the plan above, one issue file per phase in the root
  `issues/` folder (epic "Review roles").
- Until phase 1 merges, the `Human review` check (PR-004) stays as it is:
  any human collaborator's approval passes a red or cost PR. Today that is
  only Jim, so nothing is looser in the meantime.
- Jim makes the `infra-mint` environment by hand in phase 3, from a human
  task with a runbook.
