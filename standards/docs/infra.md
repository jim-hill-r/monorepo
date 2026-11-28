# Infrastructure as code

Settings we used to click through vendor dashboards (Auth0 applications,
GitHub repository settings and rulesets, Cloudflare Web Analytics) live as
Rust in the [`infra`](../../infra) project, which writes OpenTofu
configuration that CI plans on every PR and applies on merge. The design is
[decision 0007](decisions/0007-infrastructure-as-code.md); state is
[decision 0009](decisions/0009-opentofu-state-http-backend.md) and the
credential rules are [decision 0010](decisions/0010-credential-second-approval.md)
and [decision 0011](decisions/0011-mint-secrets-by-workflow.md). This page is
the rules; [infra/README.md](../../infra/README.md) has the details of each
command, guard and workflow.

## The rule

**A setting made in a dashboard goes into `infra` when a provider covers
it.** Before asking Jim to click something, or writing a human task for
it, check whether the OpenTofu provider for that vendor has the resource.
If it does, add it to `infra` instead. Only what no provider can express
(or what would put a secret in state, below) stays a human task or a
settings check in [`standards/settings.toml`](../settings.toml).

## How it fits together

1. **Constructs** in `infra/src/constructs.rs` say what we mean, such as
   `WebApp::new("cookbook").sign_in(Auth0App::import(...))`.
2. **Stacks** in `infra/src/stacks/` (`auth0`, `cloudflare`, `github`)
   expand constructs into provider resources, using typed bindings that
   [terrars](https://github.com/andrewbaxter/terrars) generates into
   `infra/src/tfschema/`.
3. **Synth** (`mise run synth`) writes one OpenTofu root module per stack
   to `infra/out/<stack>/main.tf.json`. It is committed (the one exception
   to "no build artifacts in git" in [mise.md](mise.md)), and `infra`'s CI
   fails when it differs from a fresh synth.
4. **OpenTofu** plans every PR that changes `infra/` (Infra Plan), and on
   `main` plans again and applies exactly that saved plan (Infra Apply).
   Merging the PR is the approval to apply, except for credential changes
   (below).

## Changing a setting

Edit the construct or stack, run `mise run synth` in `infra`, and commit
`src/` and `out/` together. `mise run ci` checks they match and that
`tofu validate` accepts the output. Read the plan in the PR's Infra Plan
job summary before marking the PR ready: it should change only what you
meant to.

## Adding a resource

1. **Check it can't bill and isn't a secret.** A resource that can cost
   money, or whose attributes the provider marks sensitive, doesn't belong
   in `infra` (IAC-002 and IAC-003 below). Secrets are minted by the Infra
   Mint workflow instead ([decision 0011](decisions/0011-mint-secrets-by-workflow.md)).
2. **Generate its binding.** Add the type to the `include` list in
   `infra/terrars_<provider>.json` and run `mise run bindings`. Generate
   only what is used; a whole provider is hundreds of thousands of lines.
   `terrars` is pinned with `=`, so bump it only together with
   regenerated bindings.
3. **Allowlist it** in `infra/src/guard.rs` (IAC-004) in the same PR. A
   test fails if a stack uses a type that isn't listed. If it can change a
   credential (a token, a grant, an Access policy, anything in the `github`
   stack), it must also be in the credential lists there.
4. **Import what already exists.** Every live resource comes in with an
   `import` block, so the first plan changes only what you intend and
   creates nothing that is already there. Claim it in
   [`standards/resources.toml`](../resources.toml) if it isn't claimed.
5. **Add it to a construct** when it is part of how a project is set up
   (a web app's sign-in or analytics), so the next project gets it by
   writing one line. Test the construct's expansion with unit tests, as in
   `constructs.rs`.
6. **Add a provider** only as a new stack with its own read and apply
   credentials, minted by `infra/scripts/mint.sh`, and a row in the stack
   table in `infra/README.md`.

## The guards

`cargo run -- guard plan.json` reads every plan before it is shown or
applied, and fails it when:

| ID | Fails when |
| --- | --- |
| IAC-001 | it deletes or replaces anything |
| IAC-002 | any sensitive attribute would hold a value in state |
| IAC-003 | it creates anything that can bill, or goes past a free-tier budget in `standards/radar/limits.toml` |
| IAC-004 | it uses a provider or resource type that isn't allowlisted |
| IAC-005 | it changes the `infra-credentials` environment or a ruleset's bypass list |

Never weaken a guard to get a plan through. If a guard is wrong, fix it in
its own PR with a test on a fixture plan in `infra/tests/fixtures/plans/`.

## IaC never destroys

Every resource has `prevent_destroy`, and IAC-001 fails any delete or
replace. To stop managing a resource, remove it from the code and add a
`removed` block (OpenTofu forgets it and leaves it running), and add a
`[[retired]]` entry to `standards/resources.toml` if it should actually be
deleted. Deletion then goes through the cleanup plan's safe order, with
Jim's approval ([resources.md](resources.md)). Removing a URL or setting
inside a resource is an update, not a delete, and applies normally.

The reason: a wrong plan that deletes an Auth0 application or a ruleset
can lock users or agents out, and nothing in a PR review makes that easy
to spot. Forgetting is always safe; deleting is a separate, deliberate
step.

## Credentials

- **Secrets are never in git or state** (Jim, 2026-11-26), not even
  encrypted. OpenTofu never generates or copies one; IAC-002 enforces it.
- **Credential changes wait for Jim.** A plan that changes a credential
  (listed in the job summary, and labelled `credential-change` on the PR)
  applies only in the `infra-credentials` environment, after Jim approves
  that run. The whole `github` stack counts as credential changes.
- **New credentials are minted, not requested.** Add a credential to
  `infra/scripts/mint.sh` and a row to [secret-rotation.md](secret-rotation.md),
  rather than writing a human task. A person makes only the bootstrap
  credentials ([runbook](runbooks/infra-credentials-environment.md)).

## How state works

State is kept by the `jimhillr-tofu-state` Worker with D1, through
OpenTofu's `http` backend, one URL per stack with locking
([decision 0009](decisions/0009-opentofu-state-http-backend.md)). It is
encrypted by OpenTofu with a passphrase in the `TOFU_STATE_PASSPHRASE`
secret, and it is never committed. PR plans and drift checks read it with
`-lock=false` and a read-only password; only the `main` jobs, in the
`infra` concurrency group, write it. With IAC-002 it holds only IDs and
settings already public in the committed JSON, so encryption is defence in
depth. To use it locally, `infra backend-config <stack> <origin>` prints
the `tofu init` arguments, but routine work never needs local state: CI
plans every PR.

## Fixing drift

Infra Drift runs a refresh-only plan of every stack each Monday. Drift
means someone changed a managed setting in a dashboard. Fix it with a PR,
one of two ways:

- **The dashboard change was deliberate:** change the construct or stack to
  match, synth, and check the PR's plan shows no changes for that setting.
- **It wasn't:** re-run Infra Apply on `main` (or merge any `infra`
  change), which sets it back to the code.

Ask Jim which it was when you can't tell; don't overwrite a dashboard
change he may have made on purpose. A stack whose secrets aren't set up
yet is skipped with a warning, not failed.
