# Plan: a threat check on every PR

Started 2026-11-26 from Jim's request:

> Similar to cost checks we should be doing threat checks to verify our
> technical design is secure and abides by standards. The threat check
> should also verify no secrets or keys are committed and any other risk
> related things we think are important.

Status: **approved** by Jim (PR #654, 2026-11-26), with the recommended
answer to every question (see "Decisions" near the end). The work items are
issue files in the root [issues/](../issues/) with `epic: Threat
check`, one group per phase, and the GitHub settings only Jim can switch are
in [issues/human/](../issue-management.md#human-issues).

This plan is the security twin of the
[cost check](2026-11-cost-check.md): the same shape (a structured file per
project, a fast Rust check in the hooks and PR CI, affected projects only,
warnings first), and it shares the cost check's detectors so a project's
files are read once.

## Goal

Every PR that could weaken security is caught before it merges, by the git
hooks and again by PR CI, and each project's security design is written
down where people and tools can read it.

- **No secrets in the repo, ever.** Every commit and every PR is scanned for
  keys, tokens and private key files, and the whole history is scanned
  weekly. GitHub push protection is the server-side net for anything a hook
  missed.
- **No known-vulnerable dependencies** ship without a recorded, dated reason.
- Each project keeps a **structured threat model** in its `docs/` folder:
  what it exposes, who can reach it, what data it holds, which secrets it
  uses, how sign-in works, and the main threats with their mitigations.
- A **check** proves the threat model follows the standards (nothing private
  served publicly, sign-in set up the approved way, security headers on
  every site, least-privilege workflows) and is **true to the code**: a new
  Worker, binding, secret or auth library the model doesn't list fails it.
- A **weekly live check** compares what is actually deployed on Cloudflare
  with what the threat models claim, so a forgotten or unprotected site
  (a stray Pages project such as `pane-dbc.pages.dev`, or preview URLs that
  skip Cloudflare Access) shows up within a week.
- A PR that **weakens** security (makes something public, adds a secret,
  changes sign-in, widens workflow permissions, ignores an advisory) is
  labelled `needs-human`, because those are Jim's calls.

## Where we start

| Piece | State on 2026-11-26 |
| --- | --- |
| Static analysis | CodeQL for JavaScript/TypeScript and GitHub Actions on PRs, `main` and weekly (`all-codeql.yml`). Rust is excluded (26 minutes a scan); clippy covers lints only. |
| Secret scanning | None in the repo or hooks. The repo is **public**, so GitHub secret scanning and push protection are free, but nothing records whether they are on. |
| Ignored secret files | `.gitignore` has `**/.env` only. Not covered: `.dev.vars` (wrangler's local secrets file), `*.pem`, `*.key`, `*.p12`, `*.jks`/`*.keystore` (the Android keystore is a secret today). |
| Dependency updates | `.github/dependabot.yml` has 19 Cargo entries and GitHub Actions. Five Cargo directories no longer exist (`ciam`, `dioxus_ssg`, `dioxus_static_website`, `example/binary`, `example/example_rust_library`); most of the 16 Cargo lockfiles and all three `package-lock.json` files (cahokia/web, cookbook/web, pane/web) have no entry. |
| Vulnerability audit | No `cargo audit`/`cargo deny` or `npm audit` anywhere. Whether Dependabot alerts are on is unrecorded. |
| Workflow hardening | Every workflow sets `permissions:`; `workflow-conventions.md` says to pass secrets only to the steps that need them. No check enforces either, and actions are pinned by tag, not commit. |
| Public sites | Pages projects at `*.pages.dev` for bear_down, cahokia, controlled_chaos, cookbook and pane, all public. pane's plan puts Cloudflare Access in front of pane from its Phase 3; nothing checks that it stays there, or that preview deployments are covered. |
| Sign-in | Auth0 for auth_sdk, cahokia and cookbook. Callback URLs, allowed origins and grant types live in the Auth0 dashboard only. |
| Threat models | The `deep-project-review` skill writes a STRIDE threat model into each review deck, so it is a slide, not a file a check can read. |
| Hand-made settings | The eval engine plan adds `standards/settings.toml` and `human-check` evals for dashboard settings (Access policies, token scopes, Auth0 settings). Not built yet. |
| Security policy | No `SECURITY.md`; private vulnerability reporting status unknown. |
| Hooks and PR CI | `hooks.sh` and PR CI run `//:audit` and `mise run ci` in affected projects only (PRs #627, #642). |

So the building blocks exist (CodeQL, Dependabot, hooks, the cost check's
detectors, the eval engine's settings manifest); what is missing is secret
scanning, a vulnerability audit, a per-project record of the security
design, and a check that ties them to each PR.

## Part 1: repo-wide guards (no per-project file needed)

These run on every commit and PR from phase 1 and catch the worst risks
straight away. They reuse existing open-source tools rather than new code.

| Code | Tool | Fails when | Where and how fast |
| --- | --- | --- | --- |
| SEC-001 | [gitleaks](https://github.com/gitleaks/gitleaks) (MIT) | A staged change, pushed commit or PR commit contains a secret (its default rules cover Cloudflare, GitHub, Auth0, AWS, private keys, JWTs and generic high-entropy tokens), plus our own rules for `RAPIDAPI_`, the PostHog personal key prefix `phx_` and the Auth0 management token. | Pre-commit: staged diff only (`gitleaks git --staged`), tens of milliseconds. Pre-push and PR CI: the commits being pushed or in the PR (`--log-opts=<base>..HEAD`). Weekly: the whole history. |
| SEC-002 | `.gitignore` check in `//:audit` | `.gitignore` doesn't ignore `.env`, `.dev.vars`, `*.pem`, `*.key`, `*.p12`, `*.jks`, `*.keystore`, or a tracked file matches one of them (the public `.env.example` style is allowed). | Milliseconds. |
| SEC-003 | [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) (MIT/Apache) | An affected project's `Cargo.lock` has a RustSec advisory (vulnerable, unsound or yanked), a crate from a source other than crates.io or this repo, or a licence outside our allowlist. | Only for projects whose `Cargo.lock` or `deny.toml` changed, plus a nightly run over all of them. About a second once the advisory database is cached (it refreshes at most daily). |
| SEC-004 | `npm audit --omit=dev --audit-level=high` | An affected project's `package-lock.json` has a high or critical advisory in a production dependency. | Only when that lockfile changed, plus nightly. One registry call, a second or two. |
| SEC-005 | [zizmor](https://github.com/zizmorcore/zizmor) (MIT) | A changed workflow has a high-severity finding: `pull_request_target` or `workflow_run` checking out PR code, template injection of untrusted input into `run:`, secrets passed to a job that runs PR code, `permissions: write-all`, credentials persisted by `actions/checkout`, or an unpinned third-party action. | Only when `.github/workflows/` changed. Offline, well under a second. |
| SEC-006 | `dependabot.yml` check in `//:audit` | A `Cargo.lock` or `package-lock.json` has no Dependabot entry for its directory, or an entry points at a directory that doesn't exist. | Milliseconds. |

Notes:

- **False positives.** gitleaks findings that aren't secrets (a test fixture,
  a public Auth0 client ID) go in `.gitleaksignore` by fingerprint, one line
  each with a comment saying why. An advisory we accept (for example in the
  pinned `dioxus` or `sycamore` 0.8 trees) goes in that project's
  `deny.toml` `ignore` list with a reason and a review date. Both kinds of
  exception get `needs-human` on the PR that adds them.
- **A secret that was committed is rotated, not hidden.** History isn't
  rewritten (the repo is public, so the secret is already out). The fix is a
  human task in `issues/human/` to revoke and replace it, and the
  fingerprint goes in `.gitleaksignore` once it is dead.
- **CodeQL stays as it is.** It is already a PR check. The threat check
  doesn't re-run it; it reads its result only to show it in the PR summary.
  Rust stays out of CodeQL; SEC-003 and clippy cover it.
- **Tool installs.** cargo-deny and zizmor install from crates.io, which the
  cloud sandbox can reach, so agents get the full check. gitleaks is a Go
  binary from GitHub releases, which the sandbox can't download today; there
  the hook warns that SEC-001 didn't run (the existing fallback in
  [git-hooks.md](../git-hooks.md)), PR CI runs it, and GitHub push
  protection blocks known provider tokens at push time regardless. Each tool
  is pinned to an exact version in the root `mise.toml`.

## Part 2: the threat model file

One file per project: **`<project>/docs/threat.toml`**, next to
`docs/cost.toml` and `docs/requirements.md`. TOML because the check, pane,
the eval engine and the deep review all read it; prose and diagrams go in an
optional `docs/threat.md`, which the check ignores.

```toml
# cookbook/docs/threat.toml (illustrative; the real file is written in phase 2)
reviewed = "2026-11-26"
summary = "Public recipe site; Auth0 sign-in only to edit. No personal data beyond the Auth0 user id."

# Everything someone outside the repo can reach.
[[surface]]
name = "site"
kind = "pages"                    # pages | worker | api | desktop | mobile | cli | library
url = "https://jimhillr-cookbook.pages.dev"
access = "public"                 # public | auth0 | cloudflare-access | none (not deployed)
data = "public"                   # public | personal | private (Jim-only) | secret
previews = "public"               # what Pages preview URLs expose; must match access
headers = "cookbook/web/public/_headers"

[[surface]]
name = "edit API"
kind = "worker"
url = "https://jimhillr-cookbook.pages.dev/api/*"
access = "auth0"
data = "personal"

# Names, never values. Each one also has an entry in standards/settings.toml.
[[secret]]
name = "CLOUDFLARE_API_TOKEN"
where = "github"                  # github | worker | auth0 | local
used_by = ["push-main.yml deploy step"]
scope = "Pages edit on one account; no Billing, no DNS"
setting = "cloudflare-api-token"  # id in standards/settings.toml

[auth]
provider = "Auth0"
flow = "authorization-code-pkce"  # the only flow allowed for a browser app
audience = "https://cookbook/api"
callbacks = ["https://jimhillr-cookbook.pages.dev/callback", "http://localhost:8080/callback"]
setting = "auth0-cookbook-app"

# The main threats, STRIDE-tagged. Short: the deep review keeps the long form.
[[threat]]
id = "COOK-T1"
stride = "spoofing"
what = "Someone edits recipes without signing in."
mitigation = "The Worker verifies the Auth0 JWT (issuer, audience, expiry, RS256) on every write."
eval = "COOK-E12"                 # optional: an eval in evals.toml that proves it
```

A library or a local-only CLI still has the file, with `summary` and
`surface` entries of `access = "none"`, so "no file" always means "not
assessed yet", never "safe".

### Standards the file is checked against

These go in a new **`standards/docs/security.md`**, linked from the
standards README, so the check enforces written rules rather than its own:

- Nothing with `data` other than `public` is served with `access = "public"`.
  Jim-only data (pane's dashboards, money, repo state) sits behind Cloudflare
  Access, previews included.
- Browser sign-in uses Auth0 authorization code with PKCE; no client secret
  in a web, desktop or mobile build; tokens verified server-side for issuer,
  audience, expiry and algorithm; callback URLs only on our own hosts plus
  `localhost`.
- Every public web surface sends a CSP, `X-Content-Type-Options: nosniff`,
  `Referrer-Policy`, `frame-ancestors` (or `X-Frame-Options`) and HSTS,
  from a committed `_headers` file or the Worker.
- Secrets live in GitHub repo secrets or Worker secrets, never in files; each
  has the narrowest scope that works and an entry in `standards/settings.toml`
  with how often to check it.
- Workflows grant `permissions` per job, read-only by default; secrets only
  in jobs that never run PR code; third-party actions pinned to a commit.

## Part 3: the threat check

A new command, **`threat check`**, in Rust like `cost check`. It reads only
repo files, needs no credentials or network, and runs in well under a
second once built.

| Code | Fails when |
| --- | --- |
| THR-001 | A project has no `docs/threat.toml`. |
| THR-002 | The file doesn't parse, or has an unknown field or value. |
| THR-003 | A surface serves non-public data publicly, or its `previews` don't match its `access`. |
| THR-004 | **Drift:** the project's files show a surface, secret or auth library its `threat.toml` doesn't list (detectors below). |
| THR-005 | A secret has no `scope` or no `setting` in `standards/settings.toml`, or a workflow passes `secrets.X` to a job whose project doesn't list `X`. |
| THR-006 | `[auth]` breaks the sign-in rules: a flow other than PKCE for a browser app, a callback off our hosts, or a client secret referenced in a client build. |
| THR-007 | A public web surface has no `headers` file, or the file lacks a required header. |
| THR-008 | A surface that isn't `access = "none"` has no `[[threat]]`, or a threat has no `mitigation`. |
| THR-009 | `reviewed` is more than 180 days old (a warning, never a failure). |

### Detectors: is the model true to the code?

THR-004 reuses the cost check's **`standards/radar/detectors.toml`** and its
matching code: each detector gains an optional `threat` field saying what it
implies for the threat model, so one scan of the project's files feeds both
checks.

| Pattern in the project's files | Implies |
| --- | --- |
| `wrangler.toml` with `pages_build_output_dir`, or a `DEPLOY_URL` in `mise.toml` | a `pages` surface with that URL |
| `wrangler.toml` with `main =`, `routes` or a `functions/` folder | a `worker` or `api` surface |
| `d1_databases`, `kv_namespaces`, `r2_buckets` bindings | stored data: the surface's `data` can't be `public` without a note saying why |
| `secrets.X` in a workflow the project owns, `wrangler secret put X`, or `env.X` read in a Worker | a `[[secret]]` named `X` |
| `auth0` crates or packages, `AUTH0_` variables, an `/authorize` URL | an `[auth]` table |
| `Access-Control-Allow-Origin: *` or `allow_origin("*")` | a warning on that surface |
| `dangerouslySetInnerHTML`, `inner_html`, `eval(` in web code | a warning pointing at the threat list |

### Flagging PRs that might change security

PR CI writes a short **security summary** to the job summary: which guards
ran, CodeQL's result, surfaces, secrets and auth before and after, and any
new exceptions. The same job sets labels with its `GITHUB_TOKEN`:

- `security` when a `threat.toml`, `_headers`, `wrangler.toml`, a workflow,
  `.gitleaksignore`, a `deny.toml` or a detector-matched line changed.
- `needs-human` when the PR **weakens** something: a surface becomes more
  public or its data more sensitive, a secret or an auth provider is added,
  `[auth]` changes, a workflow's permissions widen or it starts using
  `pull_request_target`, or a gitleaks or advisory exception is added.
  PR triage then waits for Jim instead of merging on green checks.

### Where it runs: fast and affected-only

The same split as the cost check (PRs #627 and #642):

- `hooks.sh` calls `standards/scripts/threat-check.sh --projects <affected>`
  right after the cost check. Per-project rules run only for affected
  projects; SEC-001 and SEC-002 always run because they are diff-sized.
- SEC-003, SEC-004 and SEC-005 run only when their input files changed, so
  a normal code change pays nothing for them.
- PR CI runs the same script with the list from `affected.sh` under
  `AFFECTED_CHANGED_ONLY`.
- The `threat` binary is built once and cached in its `target/`, like
  `evals-check.sh` does.
- Anything needing the network or credentials is weekly, not in the hook.

### The weekly live check

A new job in the existing `all-free-tier-usage.yml` run (same schedule, same
read-only Cloudflare token, so no new secret):

- Lists every Pages project and Worker on the account and flags any no
  `threat.toml` claims (an orphan like `pane-dbc.pages.dev`), with a
  `FIX` issue file in the root `issues/` to either claim it or
  delete it (deleting is a human task).
- Requests each declared URL and one preview URL per Pages project: a
  `cloudflare-access` surface must redirect to Cloudflare Access, an `auth0`
  API must answer `401` without a token, and every public site must send the
  required headers.
- Runs gitleaks over the full history, and cargo-deny and npm audit over
  every project, so a new advisory for an old lockfile shows up within a
  week even if nobody touches the project.

### Where the code lives

A new top-level project, **`threat`**, a Cargo workspace like `cost`:
`threat/model` (the `threat.toml` types and rules, pure functions) and
`threat/cli` (`threat check`, `threat summary --base <ref>`). It depends on
`cost/model` by path for the detectors, so there is one detector engine.
`standards/scripts/threat-check.sh` is the thin wrapper that also runs the
SEC tools, in the `evals-check.sh` shape.

## Reuse

- **Deep project review.** The `deep-project-review` skill reads
  `docs/threat.toml` as its starting point, and when its STRIDE section finds
  a new threat it adds it to the file in the same PR, so the review and the
  check never disagree.
- **Code review.** The `code-review` skill treats THR warnings as candidate
  findings.
- **Eval engine.** `[[threat]] eval` links a threat to an eval, so a broken
  mitigation shows up in verdicts and pane's Health light. The `http` evals
  after deploy check headers and auth on the live site; the weekly live
  check writes its results in the eval results format. Secrets and the
  Auth0, Access and push protection settings are `human-check` entries in
  `standards/settings.toml`, so Jim is asked to confirm them on a schedule.
- **pane.** A Security panel per project: surfaces and their access, open
  advisories, last full-history scan, and anything the live check found.
- **Tech radar.** gitleaks, cargo-deny and zizmor are added as tool blips in
  `standards/radar/radar.json` in the PR that first uses each. They are
  open-source tools, not vendors, so no vendor sign-off is needed.

## Phases

### Phase 1: secrets and dependencies (fails from day one)

- `standards/docs/security.md` with the standards above.
- SEC-001 (gitleaks in the hooks, PR CI and a weekly full-history run) and
  SEC-002 (`.gitignore` entries). A one-off full-history scan first; any
  real secret it finds becomes a `TODO (human)` to rotate it.
- SEC-006, and fix `dependabot.yml`: drop the five missing directories, add
  every Cargo and npm lockfile, keep the pinned `dioxus` and `sycamore` bumps
  ignored there.
- SEC-003 and SEC-004 as **warnings**, with each existing advisory triaged
  into a fix TODO or a dated `deny.toml` ignore.
- `SECURITY.md` at the root (how to report, that private vulnerability
  reporting is the channel).
- `TODO (human)` lines for the GitHub settings only Jim can switch: secret
  scanning, push protection, Dependabot alerts and security updates, and
  private vulnerability reporting.

### Phase 2: the threat model as warnings

Status: the `threat` project and `threat check` (THR-001 to THR-003, THR-005
to THR-009) are built and run from `//:audit` as warnings (2026-11-27). The
format and rules are in [security.md](../security.md#threat-models). Every
project except the on-hold luggage, starcraft and profiles has a
`docs/threat.toml` (2026-11-27). The gaps they found are FIX issues: five
sites without security headers (THR-007), and the Android keystore secret
had no settings check (added).

- The `threat` project with THR-001 to THR-003 and THR-005 to THR-009 as
  warnings, unit-tested per rule, run from `//:audit`.
- `docs/threat.toml` for the projects that deploy or sign in first (pane,
  cookbook, cahokia, bear_down, controlled_chaos, homer, auth_sdk), then the
  rest. Writing them is when real gaps (missing headers, pane public before
  Access) turn into `FIX` lines.

### Phase 3: detectors, workflows, PR summary, live check

Status: SEC-005 (zizmor, warnings) runs in the hooks, PR CI's **Workflow
audit** and nightly, and every action is pinned to a commit; the findings
left in the credential-holding infra workflows are a fix issue for a
`credential-change` PR (2026-11-27). `threat summary --base` and PR CI's
**Security summary** job set the `security` and `needs-human` labels; the
hooks pass the affected projects to the threat check through the audit
(2026-11-27). The rules are in [security.md](../security.md#pr-security-summary).

Status (2026-11-27): THR-004 is built on the shared detectors (each
detector in `detectors.toml` has an optional `threat` field; threat-only
detectors have no `blip`, and a `[[threat]]` acknowledges a risky construct
with `covers`), and the weekly live check is the `threat-live` job of
`all-free-tier-usage.yml` (`threat live` and `threat-live.sh`). The live
check fails its run and leaves issue files for PR triage to commit, because
CI never pushes to `main` (SEC-007). Both are documented in
[security.md](../security.md#drift)..

- THR-004 through the shared detectors (after cost check phase 2 adds them).
- SEC-005 (zizmor) as warnings, and pin third-party actions to commits.
- `threat summary --base` and the PR CI step with the `security` and
  `needs-human` labels; `hooks.sh` integration with hook tests.
- The weekly live check.

### Phase 4: make it fail, and reuse

- Once every project has a `threat.toml` and the check has run quietly for a
  week, SEC-003 to SEC-005 and THR-001 to THR-008 become errors.
- The eval engine, pane and deep-review reuse described above.

## Cost of this plan

Nothing new. gitleaks, cargo-deny, zizmor and npm audit are free and
open-source, GitHub secret scanning, push protection and Dependabot alerts
are free on a public repo, and everything runs on standard GitHub-hosted
runners. The weekly live check reuses the existing token and schedule. The
hooks gain steps that cost milliseconds on a normal change.

## Decisions

Jim, 2026-11-26: approved PR #654 and chose "yes to all" for the
recommended answers.

1. **File and place:** `<project>/docs/threat.toml`,
   machine-checked, with an optional `docs/threat.md` for prose.
2. **Secrets fail from day one; the rest warns first.** A committed secret is never acceptable, and gitleaks' false positives are
   handled by `.gitleaksignore`. Everything else warns until phase 4.
3. **Who decides:** anything that weakens security (more
   public, new secret, auth change, wider workflow permissions, a new
   exception) gets `needs-human` and waits for you. New threat entries,
   headers and fixes merge on green checks.
4. **GitHub's free security settings are turned on:** secret
   scanning, push protection, Dependabot alerts and security updates, and
   private vulnerability reporting. They are repo settings only you can
   switch, so they are in `issues/human/`.
5. **A separate `threat` project sharing the cost detectors,** rather than adding crates to `cost`, so a security rule change
   doesn't re-run cost CI and the other way round.

## Not doing (yet)

- Penetration testing, DAST scanners or a paid SAST tool. The live check's
  header and auth probes plus CodeQL cover the likely problems at $0.
- CodeQL for Rust. Still 26 minutes a scan; cargo-deny and clippy cover the
  dependency side, and the threat models cover design.
- Signing commits or releases, and SBOMs. Worth it once something is
  distributed to other people; today the desktop and mobile builds are for
  Jim.
- Rewriting git history to remove a leaked secret. The repo is public, so
  rotation is the fix.
