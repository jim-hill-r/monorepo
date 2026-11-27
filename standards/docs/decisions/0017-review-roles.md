# 0017. Review roles: who may approve what, as relationships in one file

- Status: Proposed (waiting for Jim's answers to the questions at the end)
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
  `credential-reviewer` is also a `critical-reviewer`".
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
   review` check does today). A role can include another role, which is
   the rewrite: `credential-reviewer` ⊂ `critical-reviewer` ⊂
   `standard-reviewer`.
3. **Domains** are what a PR can touch: `requirements`, `cost`,
   `threat-risk`, `credentials`, `governance` (the rules themselves) and
   `code` (everything else). Each domain has **levels**, and each level
   names the role that must approve it. The attention tags already are
   domain levels: `must-requirement-changed` is `requirements` at level
   `critical`.
4. **A pull request touches a domain at a level.** This is worked out
   from the diff, the description's tags and the labels, as `chores
   badges`, the Cost summary, the Security summary and IAC-005 do today.

The check: a PR may merge when, for every domain level it touches, a
member of that level's role has approved its head commit (or opened the
PR), and no member of a required role has asked for changes.

### Roles or domains? Both, on different axes

Jim asked whether roles should follow tiers (credential, critical,
standard) or domains (cost, risk, requirements). They answer different
questions, so the model keeps both:

- A **domain** says *what* changed. It is a property of the PR.
- A **role** says *who* is trusted. It is a property of people.
- The **policy** is the table that joins them: this domain at this level
  needs that role.

Three tier roles are enough today because one person holds every role. The
day someone should own only cost, add a role `cost-reviewer`, point the
`cost` domain at it, and nothing else changes. No code changes either
way. Starting with a role per domain would mean six roles that all hold
the same one person.

### Fixed rules the file can't change

Some rules must hold however the file is edited, so they are in code and
tested, not in the policy:

- An `agent`, `bot` or `auto-reviewer` is never a member of a human role,
  even if a line in the file names it.
- `auto-reviewer` can only hold the `auto` role, and `auto` can only be
  required by level `none`.
- The repo owner is a member of every role, so a bad edit can't lock
  everyone out.
- A change to the policy file touches `governance` at the level of the
  highest role it edits. Changing who is a `credential-reviewer` needs a
  `credential-reviewer`.
- The check reads the policy from the base branch, so a PR can't grant
  itself an approver.

## The policy file

`standards/reviewers.toml`, a protected path. This is the whole proposed
starting state, and it matches today's behavior except where the questions
below say otherwise.

```toml
# Who may approve what (decision 0017). Read by `chores gate`.

[roles.auto]
description = "Approves by written rules, no judgment (decision 0014)."
members = ["jimhillr-reviewer"]

[roles.standard-reviewer]
description = "Any human who can push to the repo."
members = ["github:collaborators", "role:critical-reviewer"]

[roles.critical-reviewer]
description = "Changes to what the products promise and the rules agents follow."
members = ["role:credential-reviewer"]

[roles.credential-reviewer]
description = "Anything that makes or changes a secret, token, grant or ruleset."
members = ["github:owner"]

# Each domain: how a PR touches it, and who must approve at each level.
[domains.requirements]
levels = { must-requirement-changed = "critical-reviewer", requirement-changed = "standard-reviewer" }

[domains.cost]
levels = { needs-human = "critical-reviewer", cost-changed = "standard-reviewer" }
# needs-human: a new vendor, money, or a raised budget (cost summary).

[domains.threat-risk]
levels = { weakened = "critical-reviewer", threat-risk-changed = "standard-reviewer" }
# weakened: the Security summary says the change weakens something.

[domains.credentials]
levels = { credential-change = "credential-reviewer" }
environments = ["infra-credentials"]   # its required reviewers must equal this role

[domains.governance]
levels = { rules-changed = "critical-reviewer" }
paths = ["AGENTS.md", "standards/", ".github/", ".agents/"]

[domains.code]
levels = { none = "auto" }             # only when nothing above is touched
```

## Where it is enforced

One decision function, several readers. Nothing else keeps its own list.

| Reader | Today | With this decision |
| --- | --- | --- |
| `Human review` check (`chores gate`) | Hard-coded tags and "any human collaborator" | Reads the file; reports each touched domain, the role it needs and who satisfied it. Renamed `Review roles` |
| `jimhillr-reviewer` (`auto-review.sh`) | Its own label, tag and path lists | Approves only when `Review roles` says every touched level needs `auto`. Size limit and green checks stay |
| `main` ruleset | Requires `CI result` and `Human review` | Requires `CI result` and `Review roles` |
| `infra-credentials` environment | Jim, set by hand | Still set by hand (agents must not hold that power). The settings check and infra drift compare its reviewers with `credential-reviewer` and report a mismatch |
| PR triage prompt | Prose rules naming Jim | "Never approve what `Review roles` holds; merge when it is green" |
| Pane's Approve page | Labels and tags | Shows "needs critical-reviewer (requirements)" and who can give it |
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

1. **Model and file.** `standards/reviewers.toml`, a `review_roles` module
   in `chores` (parse, fixed rules, check), and `chores gate` reading it.
   Same verdicts as today. Audit rule REV-001 fails a file that breaks a
   fixed rule or names an unknown role.
2. **All domains.** Add `credentials`, `governance` and the `needs-human`
   and `weakened` levels to the check. Rename it `Review roles`.
   `auto-review.sh` drops its own lists and follows the check.
3. **Outside GitHub's reach.** Settings check and drift report for the
   `infra-credentials` reviewers. Pane shows the needed role. The triage
   prompt, AGENTS.md and `pull-requests.md` point at the file and drop
   the names.
4. **When a second person joins.** Add their login to a role. No code.

## Risks

- **A wrong file locks PRs or opens them.** The fixed rules, REV-001, the
  base-branch read and the owner fallback bound both failures.
- **The check is repo code.** A PR could change `chores` or the workflow.
  Those paths are `governance`, the check builds from the base branch, and
  credentials keep their second gate (the environment) outside the repo,
  as decision 0010 wanted.
- **`github:collaborators` is whoever GitHub says.** Adding a collaborator
  with push access makes them a `standard-reviewer`. That is the intent,
  and the settings check lists collaborators for Jim on its schedule.

## Questions for Jim

Today every human role resolves to Jim alone, so these answers change
nothing until a second person is added. Each costs $0 either way.

1. **Does a credential reviewer count as a critical reviewer, and a
   critical one as standard?** Recommended: yes. Otherwise a person in the
   top role must be added to all three.
2. **What does a cost change need?** You first said cost changes must be
   reviewed by you, then that any human collaborator is fine. Recommended:
   `standard-reviewer` for a cost file change, `critical-reviewer` when the
   Cost summary says it adds a vendor or money or raises a budget.
3. **What do rule changes need** (AGENTS.md, `standards/`, workflows, this
   file)? Recommended: `critical-reviewer`.
4. **Should a `threat-risk-changed` PR need a human at all?** Today it is
   only kept away from `jimhillr-reviewer`. Recommended: yes,
   `standard-reviewer`, and `critical-reviewer` when the Security summary
   says the change weakens something.
