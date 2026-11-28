# Resources

Every live resource we run (a Cloudflare Pages project, Worker, database,
DNS record, Auth0 sign-in URL, Git branch, GitHub release) must be one we
still need, and nothing may be left behind when a resource is renamed or
removed (Jim, 2026-11-26). This page has the rules. The reasoning and the
phases that build the checks are in the
[resource cleanup plan](plans/2026-11-resource-cleanup.md).

Words used here:

- **Claimed:** a file on `main` names the resource, so we want it.
- **Kept:** live but not claimed by any project file, and listed in
  [`standards/resources.toml`](../resources.toml) under `[[keep]]` with a
  reason and a review date.
- **Retired:** given up on purpose, and listed under `[[retired]]` in the
  same file, so it can be deleted safely and its old name never comes back.
- **Orphan:** live, but not claimed, kept or retired. The weekly sweep
  reports every orphan.
- **Temporary:** rebuilt from git on demand (preview deployments, PR caches,
  branches of closed PRs), so automation deletes it without asking.

## Rules

1. **Every live resource is claimed.** Claims come from the files projects
   already have, so there is no inventory to keep in sync:

   | Kind | Claimed by |
   | --- | --- |
   | Pages project, Worker | `name` in a `wrangler.toml` |
   | D1, KV, Queue, R2 bucket, Cron Trigger | A binding or `[triggers]` entry in a `wrangler.toml` |
   | Public URL, Auth0 callback, origin, logout URL | `threat.toml` `[[surface]] url` and `[auth] callbacks` |
   | DNS record, custom domain | `threat.toml` `[[surface]] url` |
   | GitHub release | The game matrix in `.github/workflows/nightly-downloads.yml` |
   | Git branch | `main`, or the head of an open PR |
   | Anything else that must exist | A `[[keep]]` entry in `standards/resources.toml` |

2. **Renaming or removing a resource retires the old one in the same PR.**
   Add a `[[retired]]` entry to `standards/resources.toml` for every
   resource the PR stops claiming, or a `[[keep]]` entry if it stays on
   purpose. CI fails a PR that drops a claim without one (RES-001).
3. **Retiring or keeping is Jim's call.** PR CI labels any PR that adds a
   `[[retired]]` or `[[keep]]` entry `needs-human`. Jim approving that PR is
   the approval to delete the retired resource; nothing else is.
4. **Old names stay out of the code.** No tracked file outside
   `standards/resources.toml` and dated history (plans, decisions, reviews,
   changelogs) may mention a retired `name` or `url` (RES-002).
5. **Nothing is deleted just because it is unclaimed.** An orphan gets a
   retirement PR for Jim, never a deletion. Only the cleanup workflow
   deletes, only `[[retired]]` entries past their `delete_after` date, in
   the safe order below. Infrastructure as code never deletes
   ([decision 0007](decisions/0007-infrastructure-as-code.md)).
6. **A kept resource is looked at again.** Every `[[keep]]` entry has a
   `review` date, and the sweep fails once it has passed, so "keep" never
   means "forever without a look".
7. **Retired entries stay after deletion,** with `deleted` set to the date
   the sweep confirmed it was gone, so the repo remembers every name we
   gave up.

## `standards/resources.toml`

```toml
[[keep]]
kind = "pages"
name = "example-landing"
why = "Jim's hand-made landing page, not in the repo."
review = "2027-05-26"              # the sweep fails after this date

[[retired]]
kind = "pages"                     # pages | worker | d1 | kv | queue | cron | dns | auth0-url | branch | release
name = "cahokia-web"
url = "https://cahokia-web.pages.dev"   # every address it answered on
replaced_by = "jimhillr-cahokia"   # empty when nothing replaces it
match_name = true                  # false: RES-002 looks only for the url
why = "Renamed to the jimhillr-{project} scheme."
retired = "2026-11-27"             # the date of the PR that adds the entry
delete_after = "2026-12-04"        # at least 7 days later; remove the entry before then to undo
deleted = ""                       # set once the sweep confirms it is gone
```

The file holds no secrets: names, addresses and dates only.

## Safe deletion order

A retired resource is deleted from the outside in, so nothing ever points
at a name someone else could claim. A freed `*.pages.dev` name that an
Auth0 callback still trusts would let a stranger receive our users'
sign-in codes.

1. **Stop new deploys.** The rename PR has merged and the new resource is
   serving.
2. **Remove inbound references.** Auth0 callback, origin and logout URLs;
   DNS records and custom domains; PostHog allowed hosts; links in the repo
   (RES-002 blocks those). While no workflow can read Auth0, the retirement
   PR adds a human issue in `docs/issues/human/` for Jim to remove the Auth0 URLs,
   and deletion waits until that file is gone.
3. **Delete the resource.** For Pages, delete the project, which removes
   every deployment and preview alias with it. For a data store (D1, KV),
   first export a dump to a workflow artifact kept 90 days.
4. **Remove what protected it.** Cloudflare Access applications for the old
   hostname go last, so private content is never public in between.
5. **Verify.** The next sweep confirms it is gone, and `deleted` is set.

Each deletion is skipped while the name is still claimed on `main`, while a
Pages project has had a production deployment in the last 7 days, or while
any inbound reference remains. Until the cleanup workflow exists, a person
deletes by following a runbook, such as
[Delete old Pages projects](runbooks/delete-old-pages-projects.md).

## Temporary resources

These are rebuilt from git, so automation deletes them without asking and
lists what it deleted in the run summary:

| Resource | Rule |
| --- | --- |
| Pages preview deployments | Non-production deployments older than 14 days go, keeping the newest one per branch with an open PR. Production deployments other than the current one go after 30 days. |
| Actions caches of a PR | Deleted when the PR closes. |
| Branches of merged or closed PRs | Deleted 7 days after the PR closes (GitHub keeps the commits and a "Restore branch" button). |
| Branches with no PR | Listed after 14 days; the weekly digest asks Jim before deleting, since one could hold unfinished work. |
| Workflow artifacts | Every `upload-artifact` sets `retention-days` of 7 or less, except review decks (90) with a comment saying why (RES-003). |

## Renaming a resource

A short checklist for the PR that renames (or removes) a Pages project,
Worker, binding, URL or release:

1. Make the change in the claiming file (`wrangler.toml`, `threat.toml`,
   the release matrix).
2. Add a `[[retired]]` entry for the old name with every `url` it answered
   on, `replaced_by`, `why`, `retired` (today) and `delete_after` (at least
   7 days later).
3. Remove every mention of the old name and address from the code, docs
   and tests (RES-002 lists any left).
4. List in the PR description the inbound references outside the repo that
   must go before deletion: Auth0 URLs, DNS records, PostHog allowed hosts,
   bookmarks or links Jim shared. Add a human issue with a runbook for any
   only a person can remove.
5. Let CI label the PR `needs-human`. Jim's approval of it is the approval
   to delete the old resource once its `delete_after` date has passed.

## Checks

All in [`standards/scripts/resources.sh`](../scripts/resources.sh), tested
in `jimhillr/workflow_tests/tests/resources_tests.rs`.

| Check | Where it runs | What it does |
| --- | --- | --- |
| RES-001 a dropped claim needs an entry | `mise run //:audit` when it has a base: PR CI and pushes to `main` (`AFFECTED_BASE`), and the pre-push hook (`RESOURCES_BASE`) | Lists the claims on the base and in the working tree (`resources.sh claims [REF]`); each one that disappeared needs a `[[retired]]` or `[[keep]]` entry |
| RES-002 no references to retired names or addresses | `mise run //:audit` | Searches every tracked file except `standards/resources.toml`, plans, decisions, reviews, retros, issue files and changelogs for each retired `url`, and for the `name` as a whole word unless the entry sets `match_name = false` (for a name that is also an ordinary word, such as a project name) |
| RES-003 artifact retention of 7 days or less | `mise run //:audit` | Every `actions/upload-artifact` step sets `retention-days` of 7 or less, or has a `# RES-003:` comment in the step saying why it keeps longer |
| RES-004 entries are well formed | `mise run //:audit` | Known `kind`, the required fields, dates as `YYYY-MM-DD`, `delete_after` at least 7 days after `retired` |
| `needs-human` on new entries | PR CI, **Resource entries** job | `resources.sh new-entries BASE` lists the `[[retired]]` and `[[keep]]` entries a PR adds; the job lists them in its summary and labels the PR `needs-human` |
| Weekly sweep for orphans, overdue keeps and failed deletions | [All Free Tier Usage](../../.github/workflows/all-free-tier-usage.yml) | Planned (phase 1) |

What the claims are, per kind: a `wrangler.toml` gives `pages` or `worker`
by its `name`, `d1` by `database_name`, `kv` as `<worker>/<binding>`, `r2`
by `bucket_name`, `queue` by queue name and `cron` as `<worker>/<schedule>`;
a project's `RELEASE_TAG` gives `release`. A retirement entry uses the same
`kind` and `name`.

## Cost

$0. The checks read only files in the repo, and the sweep reads the
Cloudflare and GitHub APIs with the existing read-only token on standard
GitHub-hosted runners. Deleting retired resources frees free-tier room
(Pages project slots) rather than using it.
