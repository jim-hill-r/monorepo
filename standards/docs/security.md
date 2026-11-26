# Security

The security rules every project follows, and the checks that enforce them.
Why each rule exists, and the later phases (per-project threat models in
`docs/threat.toml`, the `threat check`, the weekly live check), are in the
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
   Third-party actions are pinned to a commit (being rolled out in phase 3 of
   the plan).

## Checks

| Code | What fails | Where it runs | Since |
| --- | --- | --- | --- |
| SEC-001 | A secret in a commit: gitleaks' default rules plus ours in [`.gitleaks.toml`](../../.gitleaks.toml) (PostHog personal keys, Auth0 client secrets). | Pre-commit (the staged diff), pre-push (the pushed commits), PR CI (the **Secret scan** job over the PR's commits, part of **CI result**), and weekly over the whole history (`all-free-tier-usage.yml`). All through [`standards/scripts/secrets-check.sh`](../scripts/secrets-check.sh). | Phase 1, as an error |
| SEC-002 | `.gitignore` doesn't ignore `.env`, `.dev.vars`, `*.pem`, `*.key`, `*.p12`, `*.jks` or `*.keystore`, or a tracked file is one of them (`*.example` files are allowed). | `//:audit`, through [`standards/scripts/security-check.sh`](../scripts/security-check.sh). | Phase 1, as an error |
| SEC-006 | A tracked `Cargo.lock` or `package-lock.json` has no entry in `.github/dependabot.yml`, or an entry's directory doesn't exist or has no lockfile (a Cargo workspace has one entry, at its root). | `//:audit`, through `security-check.sh`. | Phase 1, as an error |
| SEC-003 | A `Cargo.lock` has a RustSec advisory (vulnerable, unsound, yanked, or an unmaintained direct dependency) or a crate from a source other than crates.io or this repo. Config: [`standards/deny.toml`](../deny.toml), or a project's own `deny.toml`. Licences are not checked yet. | The hooks and PR CI (**Dependency audit**) for the lockfiles a change touches, and nightly over every lockfile (`all-dependency-audit.yml`). All through [`standards/scripts/deps-check.sh`](../scripts/deps-check.sh). | Phase 1, as a warning |
| SEC-004 | A `package-lock.json` has a high or critical advisory in a production dependency (`npm audit --omit=dev --audit-level=high`). | Same as SEC-003. | Phase 1, as a warning |

SEC-005 (zizmor) comes next, and SEC-003 to SEC-005 become errors in phase
4; see the plan.

All of these read only repo files, except that gitleaks and cargo-deny have
to be installed (cargo-deny also fetches the RustSec advisory database).
`secrets-check.sh` takes it from `PATH`, or runs the pinned version through
`mise exec`. Where neither works (cloud sandboxes can't download GitHub
releases) the hooks print `warning[SEC-001]` and carry on, and PR CI runs the
scan. Say in the PR that secrets weren't scanned locally. To install it in a
sandbox anyway, `go install github.com/zricethezav/gitleaks/v8@v8.30.1` works
through the Go module proxy.

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
2. Add a human task in `issues/human/` to revoke and replace
   it, naming the service and the secret (never its value).
3. Once it is revoked, add its fingerprint to `.gitleaksignore` with the date
   it was revoked, so the weekly full-history scan passes again.

A secret caught by the pre-commit hook never left your machine: just remove
it from the change.

## Dependencies

Dependabot opens weekly version bumps for every lockfile.

**An advisory (SEC-003, SEC-004).** Update the crate or package if you can
(`cargo update -p <crate>`, `npm audit fix`). If you can't now, add a
fix issue file in `issues/` naming the advisory IDs. If it doesn't affect
us, or no fix exists, add it to the `ignore` list of the project's
`deny.toml` (copy `standards/deny.toml` to start one) with a reason and a
review date. A PR that adds an ignore gets the `needs-human` label. The
advisories found on 2026-11-26 are `issues/threat-check-advisories-*.md`. When a project
adds a `Cargo.lock` or `package-lock.json`, add its entry to
`.github/dependabot.yml` in the same PR (SEC-006 fails until you do). Pinned
dependencies from `AGENTS.md` are listed under that entry's `ignore`, so
Dependabot never opens a bump for them.
