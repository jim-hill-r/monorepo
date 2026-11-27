# 0011. Mint infra secrets with a workflow, outside OpenTofu

- Status: Accepted
- Decided by: Jim (decision card, 2026-11-27: "Mint by workflow, not tofu")
- Date: 2026-11-27
- Asked by: Jim
- Builds on: [0007](0007-infrastructure-as-code.md) (and its "no secrets in
  state" amendment), [0009](0009-opentofu-state-http-backend.md) and
  [0010](0010-credential-second-approval.md)

## Context

Jim (2026-11-27), reviewing `issues/human/`:

> I think the only one I need to do after the IaC design is introduce the
> infra_env and secrets for minting tokens.

His rule since 2026-11-26 is "Never commit secrets to git or store them in
tf state", and AGENTS.md added "IaC never generates or copies a secret".
OpenTofu can't mint a token under that rule: an Auth0 client secret or a
Cloudflare token value comes back from the vendor API as a computed
attribute, and OpenTofu saves it in state. IAC-002 fails that plan. Without
minting, Jim had about eight token and password tasks, around 45 minutes,
and every rotation by hand.

## Decision

A workflow outside OpenTofu mints them: **Infra Mint**
(`.github/workflows/infra-mint.yml`, steps in `infra/scripts/mint.sh`).

1. It runs only by hand, from `main`, in `infra-credentials`, so Jim approves
   every run (decision 0010).
2. Each value comes from the vendor's API response, or from `openssl rand`
   for passwords, and goes through a pipe into `gh secret set`. It is masked
   in the log, never passed on a command line, never written to git, state
   or a file, and no one sees it. The one exception is the Android keystore,
   which `keytool` writes to a temporary file in the job and the job deletes.
3. It makes only what is missing. The `rotate` input remakes named secrets.
   The state passphrase is never rotated, since it encrypts state.
4. Read credentials are repository secrets, because PR plans use them.
   Anything that can change a vendor is only in `infra`.
5. A person makes only the bootstrap credentials, stored only in
   `infra-credentials`:
   - an Auth0 Machine to Machine app that can create apps and grants
   - a Cloudflare account token with only Account API Tokens: Edit
   - the infra GitHub App (Jim asked for an App instead of a PAT,
     2026-11-27), which turns its key into an installation token that
     expires within an hour

   The runbook is `standards/docs/runbooks/infra-credentials-environment.md`.
6. OpenTofu still never generates or copies a secret, and IAC-002 stays.

## Options considered

| Option | Money | Jim's time | Build effort |
| --- | --- | --- | --- |
| Mint by workflow (chosen) | $0 | about 30 minutes, once | about $3 of the $25 Claude Code budget |
| Mint with OpenTofu | $0 | none | small, but it breaks the no-secrets-in-state rule |
| Keep making tokens by hand | $0 | about 45 minutes, plus every rotation | none |

Free tiers: each run uses one Auth0 Machine to Machine token (1,000 a month
free). GitHub Apps, Cloudflare tokens and GitHub secrets are free.

## Consequences

- `issues/human/` keeps one infra setup task,
  `infra-credentials-environment`, plus `github-app-infra-credentials`
  for the app it uses. Seven older tasks are closed.
- A new credential is added to `mint.sh`, never to a human task.
- The GitHub settings tasks stay until the `github` stack
  (`issues/iac-github-stack.md`) applies them with the same app.
- AGENTS.md's no-secrets rule now names this workflow as the only
  automation that makes secrets.
