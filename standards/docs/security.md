# Security

The security rules every project follows, and the checks that enforce them.
Each project's own security design is its threat model,
[`docs/threat.toml`](#threat-models), checked by `threat check`. Why each
rule exists, and the later phases (drift detection, the PR security summary,
the weekly live check), are in the
[threat check plan](./plans/2026-11-threat-check.md). How to report a
vulnerability is in the root [SECURITY.md](../../SECURITY.md).

## Rules

1. **No non-public data on a public surface.** Anything that isn't public
   (personal data, Jim-only dashboards, money, repo state) is served only
   behind sign-in or Cloudflare Access. Pages preview URLs count as public
   surfaces too: a project whose production site is behind Access has its
   previews behind Access as well.
2. **Sign-in is Auth0 authorization code with PKCE.** No client secret in a
   web, desktop or mobile build. Servers verify tokens for issuer, audience,
   expiry and algorithm. Callback URLs are only on our own hosts, plus
   `localhost` for development.
3. **Every public web surface sends security headers:** a
   `Content-Security-Policy`, `X-Content-Type-Options: nosniff`, a
   `Referrer-Policy`, `frame-ancestors` in the CSP (or `X-Frame-Options`) and
   `Strict-Transport-Security`, from a committed `_headers` file or the
   Worker.
4. **Secrets live in GitHub repository secrets or Worker secrets, never in
   files.** Each has the narrowest scope that works (a read-only token where
   reading is enough). Local copies go in `.env` or `.dev.vars`, which git
   ignores; commit a public `<name>.example` instead.
5. **Workflows are least-privilege.** `permissions` are set per workflow or
   job, read-only by default. Secrets go only to jobs that never run PR code.
   Every action is pinned to a full commit SHA with its version in a
   comment (Dependabot's `github-actions` entry bumps both), and
   `actions/checkout` sets `persist-credentials: false` unless the job
   pushes. See [Workflows](#workflows).

## Checks

| Code | What fails | Where it runs | Since |
| --- | --- | --- | --- |
| SEC-001 | A secret in a commit: gitleaks' default rules plus ours in [`.gitleaks.toml`](../../.gitleaks.toml) (PostHog personal keys, Auth0 client secrets). | Pre-commit (the staged diff), pre-push (the pushed commits), PR CI (the **Secret scan** job over the PR's commits, part of **CI result**), and weekly over the whole history (`all-free-tier-usage.yml`). All through [`standards/scripts/secrets-check.sh`](../scripts/secrets-check.sh). | Phase 1, as an error |
| SEC-002 | `.gitignore` doesn't ignore `.env`, `.dev.vars`, `*.pem`, `*.key`, `*.p12`, `*.jks` or `*.keystore`, or a tracked file is one of them (`*.example` files are allowed). | `//:audit`, through [`standards/scripts/security-check.sh`](../scripts/security-check.sh). | Phase 1, as an error |
| SEC-007 | A workflow, shared script or shared task file runs `git push` other than `evals.sh store`'s push of eval results to the `evals-results` branch `coverage.sh store`'s push of coverage results to the `coverage-results` branch `feedback.sh store`'s push of fan feedback to the `feedback-data` branch, and `evals.sh triage`'s push (or delete) of the `evals-triage` branch, which PR triage opens a reviewed pull request from, so CI never writes `main`. | `//:audit`, through `security-check.sh`. | Phase 1, as an error |
| SEC-006 | A tracked `Cargo.lock` or `package-lock.json` has no entry in `.github/dependabot.yml`, or an entry's directory doesn't exist or has no lockfile (a Cargo workspace has one entry, at its root). | `//:audit`, through `security-check.sh`. | Phase 1, as an error |
| SEC-003 | A `Cargo.lock` has a RustSec advisory (vulnerable, unsound, yanked, or an unmaintained direct dependency) or a crate from a source other than crates.io or this repo. Config: [`standards/deny.toml`](../deny.toml), or a project's own `deny.toml`. Licences are not checked yet. | The hooks and PR CI (**Dependency audit**) for the lockfiles a change touches, and nightly over every lockfile (`all-dependency-audit.yml`). All through [`standards/scripts/deps-check.sh`](../scripts/deps-check.sh). | Phase 1, as a warning |
| SEC-004 | A `package-lock.json` has a high or critical advisory in a production dependency (`npm audit --omit=dev --audit-level=high`). | Same as SEC-003. | Phase 1, as a warning |
| SEC-005 | A workflow has a medium or high [zizmor](https://docs.zizmor.sh/audits/) finding: `pull_request_target` or `workflow_run` running PR code, template injection into `run:`, secrets reaching PR code, broad permissions or app tokens, credentials persisted by `actions/checkout`, or an action not pinned to a commit. Offline, with our ignores in [`.github/zizmor.yml`](../../.github/zizmor.yml). | The hooks and PR CI (**Workflow audit**, as annotations on the lines) for the workflows a change touches (every workflow when `.github/zizmor.yml` changed), and nightly over all of them (`all-dependency-audit.yml`). All through [`standards/scripts/workflows-check.sh`](../scripts/workflows-check.sh). | Phase 3, as a warning |
| THR-* | A project's [threat model](#threat-models) is missing, breaks a rule above, or has drifted from its code (THR-004). | `//:audit` (and so the hooks and PR CI, for the affected projects), through [`standards/scripts/threat-check.sh`](../scripts/threat-check.sh). | Phase 2 (THR-004 phase 3), as warnings |
| Live | What is deployed doesn't match the threat models (see [Live check](#live-check)). | Weekly, the **Live threat check** job of `all-free-tier-usage.yml`, through [`standards/scripts/threat-live.sh`](../scripts/threat-live.sh). | Phase 3, as an error |

CodeQL scans our own code (not dependencies): JavaScript/TypeScript and
GitHub Actions on every PR, and Rust weekly, in
`.github/workflows/all-codeql.yml` ([decision 0008](decisions/0008-codeql-rust-weekly.md)).

PR CI also writes a [security summary](#pr-security-summary) and labels the
PR. SEC-003 to SEC-005 and THR-001 to THR-008 become errors in phase 4; see
the plan.

All of these read only repo files, except that gitleaks, cargo-deny and
zizmor have to be installed (cargo-deny also fetches the RustSec advisory database).
`secrets-check.sh` takes it from `PATH`, or runs the pinned version through
`mise exec`. Where neither works (cloud sandboxes can't download GitHub
releases) the hooks print `warning[SEC-001]` and carry on, and PR CI runs the
scan. Say in the PR that secrets weren't scanned locally. To install it in a
sandbox anyway, `go install github.com/zricethezav/gitleaks/v8@v8.30.1` works
through the Go module proxy. zizmor comes from `PATH` or `mise exec` the same
way; in a sandbox, `cargo install zizmor --version 1.33.1 --locked` works
(a few minutes).

## Workflows

**A zizmor finding (SEC-005).** Fix it the way the finding's help link
says: pin the action (`uses: owner/repo@<40-character SHA> # vX.Y.Z`, the
SHA of that release's tag), add `persist-credentials: false` to the
checkout, pass `${{ ... }}` values to `run:` through `env:` and quote the
variable, or narrow `permissions` to the job that needs them. If it is safe
as it is (a job that has to push with the checkout's token), add an ignore
for that file and rule to [`.github/zizmor.yml`](../../.github/zizmor.yml)
with a comment saying why. A PR that adds an ignore gets `needs-human`. The
findings left in the infra workflows are
`threat/docs/issues/threat-check-zizmor-infra-findings.md`.

## PR security summary

PR CI's **Security summary** job runs `threat summary --base origin/main`
and writes what the PR does to the repo's security files to the job
summary. Security files are each project's `docs/threat.toml`, `_headers`
and `wrangler` configs, the workflows, `.gitleaksignore`, `.gitleaks.toml`,
`deny.toml` files and `.github/zizmor.yml`. It also lists which of SEC-003
to SEC-005 run for the PR. The job sets labels with its `GITHUB_TOKEN`:

- `security` when a security file changed (and takes it off again when none
  does any more).
- `needs-human` when the PR **weakens** security, so PR triage waits for Jim
  instead of merging on green checks. That is: a new reachable surface, a
  surface or its previews made more public (`none`, then
  `cloudflare-access` or `token`, then `auth0`, then `public`), its data made
  more sensitive, a surface that stops naming its headers, a new secret, any
  change to `[auth]`, a deleted or unparseable threat model, a workflow
  gaining a `write` permission (or `write-all`) or starting to use
  `pull_request_target` or `workflow_run`, or a new gitleaks, advisory or
  zizmor exception. The job never removes `needs-human`.

Permission changes are counted per scope, so moving a `write` grant from the
workflow to two jobs reads as widening; say so in the PR when it happens.

## Secrets

**Not a secret.** When gitleaks flags something that isn't a secret (a test
fixture, a public Auth0 client ID), add its fingerprint (printed with the
finding) to [`.gitleaksignore`](../../.gitleaksignore), under a comment
saying why it is safe. Build test fixtures at runtime instead where you can,
so there is nothing to ignore. A PR that adds an ignore line gets the
`needs-human` label.

**A real secret that reached GitHub.** The repo is public, so the secret is
already out: rewriting history doesn't help. Instead:

1. Remove it from the code and load it from a secret instead.
2. Add a human task in `docs/issues/human/` to revoke and replace
   it, naming the service and the secret (never its value).
3. Once it is revoked, add its fingerprint to `.gitleaksignore` with the date
   it was revoked, so the weekly full-history scan passes again.

A secret caught by the pre-commit hook never left your machine: just remove
it from the change.

## Dependencies

Dependabot opens weekly version bumps for every lockfile.

**An advisory (SEC-003, SEC-004).** Update the crate or package if you can
(`cargo update -p <crate>`, `npm audit fix`). If you can't now, add a
fix issue file in `docs/issues/` naming the advisory IDs. If it doesn't affect
us, or no fix exists, add it to the `ignore` list of the project's
`deny.toml` (copy `standards/deny.toml` to start one) with a reason and a
review date. A PR that adds an ignore gets the `needs-human` label. The
advisories found on 2026-11-26 are `threat-check-advisories-*.md` issue files in each affected project's `docs/issues/`. When a project
adds a `Cargo.lock` or `package-lock.json`, add its entry to
`.github/dependabot.yml` in the same PR (SEC-006 fails until you do). Pinned
dependencies from `AGENTS.md` are listed under that entry's `ignore`, so
Dependabot never opens a bump for them.

## Threat models

Every project keeps **`docs/threat.toml`**: what it exposes, who can reach
it, what data it holds, which secrets it uses, how sign-in works, and its
main threats with their mitigations. A library or local tool still has one,
with `access = "none"` surfaces, so a missing file always means "not assessed
yet", never "safe". Prose and diagrams can go in an optional
`docs/threat.md`, which the check ignores. Update the file in any PR that
adds or changes a site, Worker, secret or sign-in, and set `reviewed` to the
day you last checked it against the code.

```toml
reviewed = "2026-11-27"           # YYYY-MM-DD
summary = "Public recipe site; Auth0 sign-in only to edit."
notes = "optional"

# Everything someone outside the repo can reach.
[[surface]]
name = "site"                     # unique in the file
kind = "pages"                    # pages | worker | api | desktop | mobile | cli | library
url = "https://jimhillr-cookbook.pages.dev"   # optional
access = "public"                 # public | auth0 | cloudflare-access | token (a machine's shared secret) | none (not deployed)
data = "public"                   # public | personal | private (Jim-only) | secret
previews = "public"               # pages only, and required there: what preview URLs expose
headers = "cookbook/web/public/_headers"      # repo-relative; required when public and web
notes = "optional"

# Names, never values.
[[secret]]
name = "CLOUDFLARE_API_TOKEN"
where = "github"                  # github | worker | auth0 | local
used_by = ["push-main.yml deploy step"]
scope = "Pages edit on one account; no Billing, no DNS"
setting = "cloudflare-api-token-scopes"       # its id in standards/settings.toml

[auth]                            # only when people sign in
provider = "Auth0"
flow = "authorization-code-pkce"  # | authorization-code | client-credentials (servers only)
audience = "https://cookbook/api"
callbacks = ["https://jimhillr-cookbook.pages.dev/callback", "http://localhost:8080/callback"]
setting = "auth0-cookbook-app"    # optional, its id in standards/settings.toml

# The main threats, STRIDE-tagged. Short: the deep review keeps the long form.
[[threat]]
id = "COOK-T1"                    # unique in the file
stride = "spoofing"               # spoofing | tampering | repudiation | information-disclosure
                                  # | denial-of-service | elevation-of-privilege
what = "Someone edits recipes without signing in."
mitigation = "The Worker verifies the Auth0 JWT (issuer, audience, expiry, RS256) on every write."
surface = "edit API"              # optional; without it the threat covers every surface
eval = "COOK-E12"                 # optional: an eval in evals.toml that proves it
covers = ["raw-html"]             # optional: detector ids this threat accounts for (THR-004)
```

`threat check` (the [`threat`](../../threat/) project) reads only repo files
and reports:

| Code | Warns when |
| --- | --- |
| THR-001 | A project (a top-level directory with a `mise.toml`) has no `docs/threat.toml`. |
| THR-002 | The file doesn't parse, has an unknown field or value, lists a surface, secret or threat id twice, or sets `previews` on a surface that isn't on Pages. |
| THR-003 | A `public` surface holds data that isn't `public` (rule 1), or a Pages surface doesn't say what its previews expose or they are more open than production. |
| THR-004 | **Drift:** the project's files show something the file doesn't account for, found by the shared detectors (see "Drift" below): a Pages site or Worker with no surface of that kind, a `DEPLOY_URL` host no surface's `url` is on, a stored-data binding while every reachable surface says `data = "public"`, a `wrangler secret put X` with no `[[secret]]` `X`, a sign-in library with no `[auth]`, or a risky construct no `[[threat]]` `covers`. Also a `covers` naming a detector that doesn't exist. |
| THR-005 | A secret has no `scope`, no `setting`, or a `setting` that isn't in `standards/settings.toml`; or a workflow named after the project (`<project>.yml` or `<project>-*.yml`, `_` written `-`) reads a `secrets.X` the file doesn't list (rule 4). |
| THR-006 | `[auth]` breaks rule 2: a provider other than Auth0, the `implicit` or `password` flow, a flow that needs a client secret in a project with a `pages`, `desktop` or `mobile` surface, a callback off our hosts (this file's surface URLs or a `jimhillr-*` Pages project, plus `localhost`) or over plain `http`, or an unknown `setting`. A native app's own URL scheme is allowed. |
| THR-007 | A `public` `pages`, `worker` or `api` surface has no `headers`, the file doesn't exist, or it doesn't set every header in rule 3. |
| THR-008 | A surface whose `access` isn't `none` has no `[[threat]]` covering it, a threat has no `mitigation`, or names a surface the file doesn't list. |
| THR-009 | `reviewed` is more than 180 days old. Always a warning. |

Run it with `bash standards/scripts/threat-check.sh` (every project, or
`--projects a,b`); the audit passes it the affected projects in the hooks
and PR CI.

### Drift

THR-004 keeps the file true to the code. It reuses the cost check's
detectors in [`standards/radar/detectors.toml`](../radar/detectors.toml)
(one engine reads each project's tracked files for both checks, the same
way, see [cost.md](cost.md#detectors)): a detector's `threat` field says
what a match implies here.

| `threat` | Detectors | The file needs |
| --- | --- | --- |
| `pages` | a `wrangler.toml` with `pages_build_output_dir` | a `kind = "pages"` surface |
| `worker` | a `wrangler.toml` with `main` | a `worker` or `api` surface |
| `url` | `DEPLOY_URL = "https://..."` in a `mise.toml` | a surface whose `url` is on that host |
| `storage` | `d1_databases`, `kv_namespaces`, `r2_buckets` bindings | a reachable surface whose `data` isn't `public`, or a `[[threat]]` that `covers` the detector and says why the data is public |
| `secret` | `wrangler secret put X`, `wrangler pages secret put X` | a `[[secret]]` named `X` |
| `auth` | an `@auth0/` package or `auth0` crate | an `[auth]` table |
| `risk` | `cors-any-origin` (`Access-Control-Allow-Origin: *`), `raw-html` (`dangerouslySetInnerHTML`, `dangerous_inner_html`, `innerHTML =`, `new Function(`) | a `[[threat]]` with `covers = ["<detector id>"]` whose mitigation says why it is safe |

Secrets a workflow passes are THR-005's job. A false match (a test that
stubs a CORS reply, say) goes in `[[exempt]]` in `detectors.toml` with a
reason, like any other detector.

## Live check

Once a week (Mondays, with the free-tier usage run, using the same
read-only Cloudflare token), `threat-live.sh` compares what is deployed with
what the threat models say, through `threat live`:

- **Orphans.** Every Pages project and Worker on the account must be
  claimed by a surface: a Pages project when a surface's `url` is on
  `<name>.pages.dev` or one of its custom domains, a Worker when a surface's
  `url` is on `<name>.<account>.workers.dev`. An unclaimed one (a stray such
  as `pane-dbc.pages.dev`) fails the run, and the run's `threat-live`
  artifact holds a `kind: fix` issue file for it, which PR triage commits
  (see [repo-health.md](repo-health.md)). The fix is to claim it in the
  owning project's `docs/threat.toml`, or to delete it, which is a human
  task with a runbook.
- **Access.** Each reachable web surface's `url` (globs skipped) and the
  newest preview of each claimed Pages project are requested once, without
  following redirects. A `cloudflare-access` surface (or preview) must
  redirect to `*.cloudflareaccess.com`; an `auth0` or `token` API must
  answer `401` or `403`; a `public` one must answer below `400` and not
  redirect to Access.
- **Headers.** A public surface that answers `2xx` must send every header
  in rule 3.

Requests send the `jimhillr-threat-live` user agent and
`x-jimhillr-synthetic: 1`, so analytics can leave them out. A token that
can't list Workers is a warning; the Pages checks still run.

## GitHub security settings

GitHub's own code security is the server-side net behind the checks above:
push protection rejects a push with a known secret before it reaches any
branch (our hooks can be skipped, and PR CI runs only after the push),
secret scanning also reads issues and PR comments, Dependabot alerts fire
as soon as an advisory is published against a dependency that hasn't
changed, and private vulnerability reporting lets outsiders report a hole
without a public issue. All are free on this public repo, and none was on
by default.

The `github` stack in `infra` holds secret scanning, push protection,
Dependabot alerts and security updates. Private vulnerability reporting has
no OpenTofu resource, so Jim set it by hand
([runbook](runbooks/github-code-security.md)), and the weekly **GitHub
security settings** job of `all-free-tier-usage.yml` fails if it is ever
turned off.
