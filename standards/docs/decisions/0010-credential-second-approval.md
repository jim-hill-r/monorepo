# 0010. A second approval for credential changes: a GitHub environment only Jim can approve

- Status: Proposed
- Date: 2026-11-26
- Asked by: Jim
- Builds on: [0007](0007-infrastructure-as-code.md) (IaC, and its
  "no secrets in state" amendment), PR #701 (`infra guard`), and the coming
  decision 0009 (state in R2)

## Context

Jim (2026-11-26):

> Now that we have IaC minting credentials we need much better controls
> around them. Lets design a process where I have to explicitly approve any
> changes to credentials outside of the typical PR workflow. So new
> credentials or edits would require a pr approval + a second approval
> (probably in pane?)

Since the "no secrets in state" amendment to 0007, IaC never mints a token
or writes a secret's value. It still changes what credentials *can do*: an
Auth0 application's grants and authentication method, a token's scopes, who
an Access policy lets in, which GitHub secrets and environments exist. Those
are the changes this record gates.

### What counts as a credential change

A plan has a credential change when it creates or updates any of these
(the exact resource list lives in `infra/src/guard.rs`, next to the IAC-004
allowlist):

| Vendor | Resources | Why |
| --- | --- | --- |
| Auth0 | creating an application (`auth0_client`), its credentials and authentication method (`auth0_client_credentials`), its API grants and scopes (`auth0_client_grant`), an API's scopes (`auth0_resource_server*`), connections and roles | Each one makes a new credential or changes what one can reach. Editing an existing app's callback, origin and logout URLs is not a credential change. |
| Cloudflare | API tokens and their policies (`cloudflare_api_token`, `cloudflare_account_token`), Access applications, policies and service tokens (`cloudflare_zero_trust_access_*`) | Token scopes, and who can sign in to pane. |
| GitHub | Actions and environment secrets and variables, environments and their protection, rulesets and branch protection, deploy keys, collaborators and team access | Where credentials live and who can use them. Repo settings such as "delete head branches" sit here too, because GitHub only lets them be changed with the same Administration permission. |

### Who must not be able to get round it

Agents act as `jimhillr-agt`, which has push (Write) on the repo and no
admin (checked 2026-11-26: `admin: false, maintain: false, push: true`). An
agent can push any branch, open PRs that change workflows or the guard,
approve other people's PRs in triage, and merge any mergeable PR. It cannot
approve its own PR, change repo settings, or sign in as Jim. So the gate
must hold even if an agent:

1. edits `infra/src/guard.rs` so a credential change isn't classed as one;
2. edits the workflow to drop the approval step, or adds a new workflow;
3. pushes a branch whose workflow asks for the credential tokens;
4. merges a PR (agents may merge), so the apply run starts as `jimhillr-agt`;
5. ships pane code (pane deploys on every push to `main`).

A gate written in our own code (a step in a workflow, a check in pane)
can be removed by a PR, and then only the PR approval is left, which is
exactly what Jim wants to go beyond. The second approval has to be enforced
by something agents can't edit, and it has to be attached to the
credentials themselves, so that removing it means losing the credentials,
not skipping a check.

## Options

All three are free in money. Claude build effort is a rough count of agent
PRs, which spends the $25 a month Claude Code budget.

| | Option | Money | Free-tier impact | Claude build effort | Can an agent get round it? |
| --- | --- | --- | --- | --- | --- |
| **A** | **Approve in pane.** The apply job waits for an approval record that Jim makes on a pane page behind Cloudflare Access. | $0 | Workers (a few requests per change, of 100,000 a day), Access (1 user of 50), KV (a few writes, of 1,000 a day) | About 4 PRs, and it needs pane Phase 3 (the Worker and Access) built first | **Yes.** The record is checked by workflow code a PR can delete (2), and the approving code is pane, which every merge to `main` redeploys with our Cloudflare token (5). |
| **B** | **A GitHub environment with Jim as the only required reviewer.** The tokens that can change credentials are secrets of an `infra-credentials` environment. A job using it waits until Jim presses "Approve and deploy" on the run, in GitHub or GitHub Mobile. | $0: environments with required reviewers are free on public repos. If the repo ever went private they'd need GitHub Pro, $4 a month. | Actions: one extra job per credential change, a minute or less. Nothing else. | About 2 PRs, which fit into IaC phase 1's workflow PR, plus about 20 minutes of settings for Jim | **No**, with the settings below. GitHub enforces it server-side; only Jim can approve; only an admin can change the rules, and `jimhillr-agt` isn't one; and without the environment the tokens don't exist. |
| C | **Signed approvals.** Jim signs the plan's hash (an SSH key, or a passkey through pane), and the apply job checks the signature against Jim's public key. | $0 | None, or as A when signing through pane | About 4 PRs (WebAuthn in a Worker, or a signing command Jim runs on his laptop, which breaks "no local clone" and is awkward on a phone) | **Yes, on its own.** The check is workflow code, so (2) removes it. It only holds on top of B. |

### Why not A or C

Both put the gate in code that agents can change. Hardening A with
signatures turns it into C, and hardening C means keeping the credentials
in a place only Jim can release, which is B. B is also the only one that
works today without building pane Phase 3.

Pane still has a part to play: it is where Jim looks first. Option B sends
Jim GitHub's own notification for each waiting run, and from pane Phase 3
pane lists them too (below).

## Decision

Option B. Each item is a default, so Jim can answer "yes to all".

1. **Two GitHub environments for apply.**
   - `infra`: the write tokens for everyday changes, scoped so they
     **cannot** change credentials (item 3). Deployment branches: `main`
     only. No reviewer, so ordinary IaC still applies on merge (0007,
     item 5).
   - `infra-credentials`: the tokens that can change credentials.
     Deployment branches: `main` only. **Required reviewer: `jimhillr`
     only.** "Allow administrators to bypass configured protection rules"
     **off**. A wait timer of 0.
   No credential-capable token is ever a repository secret, since every
   workflow on every branch can read those.
2. **The gate is the tokens, not our code.** The guard (PR #701) gains
   **IAC-005**, which does not fail a plan but marks each stack's plan as a
   credential change when it touches a resource from the table above. The
   apply workflow sends that stack's saved plan to the job in
   `infra-credentials`; every other stack applies in `infra` as now. If an
   agent breaks the classification, the credential change goes to `infra`,
   whose tokens are refused by the vendor, and the apply fails. It never
   applies without Jim.
3. **Split every write token in two**, so the vendor enforces the line:

   | Vendor | `infra` (everyday) | `infra-credentials` (needs Jim) |
   | --- | --- | --- |
   | Auth0 | A Management API app with `read:clients` and `update:clients` only, which can edit URLs and settings but not secrets, authentication methods or grants (`update:client_keys`, `*:client_grants`, `create:clients` are left out) | A second app with those scopes too |
   | Cloudflare | Pages: Edit, account-scoped | Also API Tokens: Edit and Access: Apps and Policies: Edit |
   | GitHub | No GitHub write token (the `github` stack has nothing that isn't credential-class, see the table) | A fine-grained token on Jim's account, this repo only: Administration, Environments, Secrets and Variables: read and write |

   Plans on PRs keep using read-only tokens (0007, item 9). Jim creates and
   rotates all of these by hand, as now; none is made by IaC.
4. **Jim approves the exact plan.** On a push to `main`, the plan job (read
   tokens) writes the saved plan as a workflow artifact and puts its SHA-256
   and a readable summary of each credential change (resource, what changes,
   before and after scopes) in the job summary. The `infra-credentials` job
   downloads that artifact, checks the hash, and runs `tofu apply` on that
   file; it never re-plans. Jim sees the summary on the run before he
   approves. An approval is for one run only, so it can't be reused.
5. **The PR says so up front.** PR plans run IAC-005 too and the plan comment
   leads with "Credential change: after merge, this needs your second
   approval", listing what changes, so the first approval is given knowing a
   second is coming. The PR gets a `credential-change` label, and PR triage
   never approves a PR with that label, whoever opened it.
6. **Rejecting or ignoring it is safe.** If Jim rejects the run, or it waits
   30 days and GitHub cancels it, nothing is applied. `main` then holds code
   that isn't live, which the weekly drift plan reports; the fix is a revert
   PR or a fresh run. All infra jobs share one `concurrency` group, so a
   later apply waits behind a pending approval rather than racing it.
7. **IaC never manages its own gate.** IAC-005 hard-fails (a finding, not a
   routing) any plan that touches the `infra-credentials` environment, its
   reviewers, its secrets, or the `main` ruleset's bypass list. Those stay
   settings Jim makes by hand, recorded in `standards/settings.toml` and
   checked by the weekly digest:
   - `infra-credentials` has `jimhillr` as its only reviewer, admin bypass
     off, and `main` as its only deployment branch;
   - `jimhillr-agt` has Write, not Maintain or Admin, and isn't on any
     ruleset bypass list;
   - the Claude GitHub App has no Administration, Environments or Secrets
     write permission on the repo;
   - the Cloudflare connector stays read-only (already a check).
8. **Pane is the inbox, never the approver.** From pane Phase 3, the Worker
   lists runs waiting on `infra-credentials` (GitHub's pending deployments,
   read with Actions: read) on the Today page, each linking to the run's
   "Review deployments" button on GitHub. Pane's GitHub token must never get
   Deployments write or any permission that can approve a deployment,
   because anyone who can merge pane code could then approve as Jim; this
   goes in pane's plan and as a settings check. Until Phase 3, GitHub's
   email and mobile notifications to the required reviewer do this job.
9. **Docs.** `standards/docs/infra.md` (or `infra/README.md` until it
   exists) explains the two tiers, IAC-005 and how to approve; `AGENTS.md`
   gets one rule: "A credential change applies only after Jim approves the
   `infra-credentials` run; agents never work round it, never approve a
   `credential-change` PR, and never ask for credential-capable tokens
   outside that environment."

### What it doesn't protect against

- **Jim approving without reading.** The summary in item 4 is there to make
  reading quick.
- **Jim's own account or session.** Use a passkey or security key on
  `jimhillr`; that is a human check, not something the repo can enforce.
- **Changes made by hand in a dashboard.** They bypass IaC entirely; the
  weekly drift plan and the cleanup sweep report them.

## Phases

1. **With IaC phase 1's workflow PR** (the IaC thread owns it): IAC-005 in
   `guard.rs` with fixture tests, the plan artifact and hash, the
   `infra-credentials` job, the PR comment line and label, docs and the
   AGENTS.md rule. About 2 PRs.
2. **Human tasks for Jim** (one-time, about 20 minutes): create the
   `infra-credentials` environment with the settings in item 1, create the
   second Auth0 Management app and split the Cloudflare token, put the
   credential tokens only in `infra-credentials`, and confirm the checks in
   item 7. Filed as `issues/human/` files when this is accepted, with their
   `standards/settings.toml` entries.
3. **PR triage** skips approving `credential-change` PRs (a line in
   `pull-requests.md`'s safe-to-approve rules).
4. **Pane Phase 3**: the pending-approvals list on Today (item 8).

## Cost

$0 in money on the current public repo: environments, required reviewers
and environment secrets are free there, and the extra job is a minute or
less of Actions per credential change. If the repo ever goes private,
required reviewers need GitHub Pro at $4 a month, which Jim would have to
approve. No new vendor. Claude build effort is about 2 PRs, inside IaC
phase 1, plus a line in pane Phase 3. One more Auth0 machine-to-machine app
adds no cost (its tokens come out of the same 1,000 a month, and credential
applies are rare).

## Consequences

- Everyday IaC (Auth0 URLs, Pages projects) still applies on merge with one
  approval. Credential changes take two, both from Jim, and the second is
  on the exact plan that runs.
- The `github` stack only ever applies with Jim's second approval, because
  GitHub can't separate repo settings from rulesets and secrets.
- The gate is a GitHub setting, not code, so a PR can't remove it; changing
  it is a dashboard change only Jim can make, and the weekly digest checks
  it.
- Agents keep their current GitHub role. Giving `jimhillr-agt` admin, or
  the Claude App Administration, would undo this decision, so both are
  checked.
