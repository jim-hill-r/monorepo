# 0019. Review roles: who may approve what, as relationships in one file

- Status: Proposed (Jim answered the first four questions on 2026-11-27; three follow-ups are at the end)
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
  as "`jimhillr` is a member of `critical-reviewer`".
- **Rewrites**: rules that derive one relation from another, such as "a
  collaborator on the repo is a `product-reviewer`".
- **A check**: a pure function over the tuples and rewrites.

We need the model, not a server. Our tuples number in the tens, change a
few times a year, and must be reviewed like code. So they belong in a file
in the repo (files before databases), read by one small Rust function.

## The model

Four kinds of thing, and the relations between them.

```text
subject ──member──▶ role ──reviews──▶ domain ◀──touches── pull request
(account)          (group)           (what changed)      (diff, labels, tags)
```

1. **Subjects** are GitHub accounts. Each has a kind the policy can't
   change: `human`, `agent` (login ends in `-agt`), `auto-reviewer`
   (`jimhillr-reviewer`) or `bot`. The kind comes from the login and the
   API, never from the file.
2. **Roles** are named groups of subjects. A member is a login, or a
   relationship GitHub already holds: `github:owner` and
   `github:collaborators` (read from `author_association`, as the `Human
   review` check does today). Roles don't include each other
   (Jim: "No but one review from me should satisfy both roles"): each role
   lists its own members, and one approval counts for every role its
   author holds.
3. **Domains** are what a PR can touch: `requirements`, `cost`,
   `threat-risk`, `credentials`, `platform` (the rules and the shared
   tooling) and `product` (the things users use). Each domain has **levels**, and each level
   names the role that must approve it. The attention tags already are
   domain levels: `must-requirement-changed` is `requirements` at level
   `critical`.
4. **A pull request touches a domain at a level.** This is worked out
   from the diff, the description's tags and the labels, as `chores
   badges`, the Cost summary, the Security summary and IAC-005 do today.

The check: a PR may merge when, for every domain level it touches, a
member of that level's role has approved its head commit (or opened the
PR), and no member of a required role has asked for changes.

### Roles follow domains

Jim asked whether roles should follow tiers (credential, critical,
standard) or domains (cost, risk, requirements). His answers on 2026-11-27
settled it as domains, with one role each:

| Role | Approves | Members today |
| --- | --- | --- |
| `credential-reviewer` | Anything that makes or changes a secret, token, grant or ruleset | Jim |
| `critical-reviewer` | A change to a `must` requirement | Jim |
| `cost-reviewer` | Every cost change ("Cost-reviewer.. again me") | Jim |
| `threat-reviewer` | Every threat or risk change ("Always. We should have a threat-reviewer and have it be just me") | Jim |
| `technical-platform-reviewer` | The rules, workflows and platform projects ("A technical-platform-reviewer should be just me for now") | Jim |
| `product-reviewer` | Product projects (the old standard reviewer) | Any human collaborator |
| `auto` | Product changes that touch nothing above, by written rules | `jimhillr-reviewer` |

A domain says *what* changed, a role says *who* is trusted, and the policy
file joins them. One approval from Jim satisfies every role he holds, so a
PR that touches cost and a threat model still needs one click. Handing
cost to someone else later is one line in the file.

### Technical platform and products

Jim, 2026-11-27: "Lets introduce somewhere in our processes the concept of
technical platform versus products. Every project of codebase should live
in one of these two categories."

- A **product** is something an end user uses: a site, an app, a game.
- The **technical platform** is everything products are built, checked,
  deployed and governed with: standards, shared libraries, tooling,
  infrastructure, and the rules agents follow.

Every top-level folder is listed under exactly one category in
`standards/reviewers.toml`, and the audit fails a folder that is in
neither or both (REV-002). Root files and dot folders (`AGENTS.md`,
`.github/`, `.agents/`, `mise.toml`) are always platform. The category
decides the reviewer, and other processes can read it too: the deep review
rotation, cost attribution, the tech radar and pane's project list.

Proposed split (follow-up question 2 asks about the unsure ones):

| Category | Projects |
| --- | --- |
| Technical platform | `standards`, `chores`, `infra`, `tofu_state`, `evals`, `cost`, `threat`, `telemetry`, `jimhillr`, `auth_sdk`, `design_system`, `pane`, `athenaeum`, `issues`, `docs` |
| Product | `bear_down`, `blueeel`, `bogan`, `cahokia`, `controlled_chaos`, `cookbook`, `homer`, `luggage`, `my_lang`, `profiles`, `psy_ents`, `royalty`, `sparkhill`, `starcraft` |

### Fixed rules the file can't change

Some rules must hold however the file is edited, so they are in code and
tested, not in the policy:

- An `agent`, `bot` or `auto-reviewer` is never a member of a human role,
  even if a line in the file names it.
- `auto-reviewer` can only hold the `auto` role, and `auto` can only be
  required by a `product` level.
- The repo owner is a member of every role, so a bad edit can't lock
  everyone out.
- A change to the policy file needs every role it edits, on top of the
  `technical-platform-reviewer`. Changing who is a `credential-reviewer`
  needs a `credential-reviewer`.
- The check reads the policy from the base branch, so a PR can't grant
  itself an approver.

## The policy file

`standards/reviewers.toml`, a protected path. This is the whole proposed
starting state.

```toml
# Who may approve what (decision 0019). Read by `chores gate`.
# Roles don't include each other. One approval counts for every role its
# author holds.

[roles.auto]
description = "Approves by written rules, no judgment (decision 0014)."
members = ["jimhillr-reviewer"]

[roles.product-reviewer]
description = "Any human who can push to the repo."
members = ["github:collaborators"]

[roles.technical-platform-reviewer]
description = "The rules agents follow and the tooling every product is built with."
members = ["jimhillr"]

[roles.critical-reviewer]
description = "Changes to what a product must do."
members = ["jimhillr"]

[roles.cost-reviewer]
description = "Anything that moves money, free-tier usage or a budget."
members = ["jimhillr"]

[roles.threat-reviewer]
description = "Anything that changes what is exposed or trusted."
members = ["jimhillr"]

[roles.credential-reviewer]
description = "Anything that makes or changes a secret, token, grant or ruleset."
members = ["jimhillr"]

# Every top-level folder is in exactly one list (REV-002).
[projects]
platform = ["standards", "chores", "infra", "tofu_state", "evals", "cost",
  "threat", "telemetry", "jimhillr", "auth_sdk", "design_system", "pane",
  "athenaeum", "issues", "docs"]
product = ["bear_down", "blueeel", "bogan", "cahokia", "controlled_chaos",
  "cookbook", "homer", "luggage", "my_lang", "profiles", "psy_ents",
  "royalty", "sparkhill", "starcraft"]

# Each domain: how a PR touches it, and who must approve at each level.
[domains.requirements]
levels = { must-requirement-changed = "critical-reviewer", requirement-changed = "product-reviewer" }

[domains.cost]
levels = { cost-changed = "cost-reviewer" }   # the tag or the `cost` label

[domains.threat-risk]
levels = { threat-risk-changed = "threat-reviewer" }   # the tag or the `security` label

[domains.credentials]
levels = { credential-change = "credential-reviewer" }
environments = ["infra-credentials"]   # its required reviewers must equal this role

[domains.platform]
levels = { changed = "technical-platform-reviewer" }   # any file in a platform project or at the root

[domains.product]
levels = { needs-human = "product-reviewer", changed = "auto" }
# needs-human: the label, a failed size limit, or anything the auto rules refuse.
```

What changes from today's behavior:

- Cost changes go back to needing Jim (they were "any human collaborator"
  for a few hours).
- Threat and risk changes need Jim. Today they are only kept away from
  `jimhillr-reviewer`.
- Every change to a platform project needs Jim. Today `jimhillr-reviewer`
  may approve small green changes in `chores`, `evals`, `cost`, `pane`,
  `telemetry`, `design_system` and `athenaeum` (follow-up question 1).

## Where it is enforced

One decision function, several readers. Nothing else keeps its own list.

| Reader | Today | With this decision |
| --- | --- | --- |
| `Human review` check (`chores gate`) | Hard-coded tags and "any human collaborator" | Reads the file; reports each touched domain, the role it needs and who satisfied it. Renamed `Review roles` |
| `jimhillr-reviewer` (`auto-review.sh`) | Its own label, tag and path lists | Approves only when `Review roles` says every touched level needs `auto`, which only product changes can. Size limit and green checks stay |
| `main` ruleset | Requires `CI result` and `Human review` | Requires `CI result` and `Review roles` |
| `infra-credentials` environment | Jim, set by hand | Still set by hand (agents must not hold that power). The settings check and infra drift compare its reviewers with `credential-reviewer` and report a mismatch |
| PR triage prompt | Prose rules naming Jim | "Never approve what `Review roles` holds; merge when it is green" |
| Pane's Approve page | Labels and tags | Shows "needs cost-reviewer, threat-reviewer" and who can give it |
| Labels (`needs-human`, `cost`, `security`, `credential-change`) | Each set by its own job | Kept as inputs and as what people filter by. `needs-human` is set whenever a level above `auto` is unmet |

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

1. **Model, file and categories.** `standards/reviewers.toml` with the
   roles and the platform and product lists, a `review_roles` module in
   `chores` (parse, fixed rules, check), and `chores gate` reading it for
   requirements and cost. Audit rules REV-001 (a file that breaks a fixed
   rule or names an unknown role) and REV-002 (a folder in neither or both
   categories). AGENTS.md and `REPOSITORY_STRUCTURE.md` gain the platform
   and product rule.
2. **All domains.** Add `threat-risk`, `credentials`, `platform` and
   `product` to the check. Rename it `Review roles`. `auto-review.sh`
   drops its own lists and follows the check.
3. **Outside GitHub's reach.** Settings check and drift report for the
   `infra-credentials` reviewers. Pane shows the needed role. The triage
   prompt, AGENTS.md and `pull-requests.md` point at the file and drop
   the names.
4. **When a second person joins.** Add their login to a role. No code.

## Risks

- **A wrong file locks PRs or opens them.** The fixed rules, REV-001, the
  base-branch read and the owner fallback bound both failures.
- **The check is repo code.** A PR could change `chores` or the workflow.
  Those paths are `platform`, the check builds from the base branch, and
  credentials keep their second gate (the environment) outside the repo,
  as decision 0010 wanted.
- **`github:collaborators` is whoever GitHub says.** Adding a collaborator
  with push access makes them a `product-reviewer`. That is the intent,
  and the settings check lists collaborators for Jim on its schedule.

## Jim's answers (2026-11-27)

1. *Does a credential reviewer count as a critical reviewer, and so on?*
   "No but one review from me should satisfy both roles." Roles don't
   include each other; one approval counts for every role its author holds.
2. *What does a cost change need?* "Cost-reviewer.. again me."
3. *What do rule changes need?* "Lets make this a
   technical-platform-reviewer and make the standard one a
   product-reviewer. A technical-platform-reviewer should be just me for
   now. Lets introduce somewhere in our processes the concept of technical
   platform versus products. Every project of codebase should live in one
   of these two categories."
4. *Should a threat or risk change need a human?* "Always. We should have
   a threat-reviewer and have it be just me."

## Follow-up questions for Jim

Each costs $0 either way; the difference is how many PRs wait for you.

1. **Does every change to a platform project need the
   technical-platform-reviewer, or only changes to the rules?**
   Recommended: every change. It is the simple reading of "platform versus
   products", and a platform bug reaches every product. It means
   `jimhillr-reviewer` approves product changes only, so the small green
   PRs in `chores`, `evals`, `cost`, `pane`, `telemetry`, `design_system`
   and `athenaeum` start waiting for you. The other choice keeps the fast
   lane for those and holds only the rules (`AGENTS.md`, `standards/`,
   `.github/`, `.agents/`, `infra/`).
2. **Is the split right?** The ones I was least sure of: `pane` (your
   review dashboard, listed as platform), `royalty` (listed as a product,
   since others would use it), `auth_sdk` and `design_system` (platform,
   as shared libraries), `my_lang` (product) and `athenaeum` (platform,
   as reference material).
3. **Does `critical-reviewer` stay for `must` requirement changes?** You
   renamed the other two original roles. Recommended: keep it, with you as
   the only member.
