# 0007. Infrastructure as code: Rust synthesizes Terraform JSON, OpenTofu applies it

- Status: Accepted
- Decided by: Jim (approved PR #676, 2026-11-26, with every default below)
- Date: 2026-11-26
- Asked by: Jim
- Follow-up: [terrars spike](0007-terrars-spike.md) (2026-11-26): adopt `terrars`

## Context

Jim (2026-11-26):

> One of the first tasks for me is to 'In the Auth0 dashboard, add the
> Cloudflare Pages preview domains to the allowed callback URLs for cookbook
> and cahokia.' I'd like to avoid clickops as much as possible, so we should
> evaluate IaC options to determine what technology is best for our tech
> stack as defined by tech radar. We can consider a build option if nothing
> leveraging rust is out there. Terraform is ubiquitous but I don't really
> like it's DSL. Pulumi doesn't support rust. Some solution like pulumi that
> supports rust would be ideal. Lastly, I would love to have our cdk style
> solution synth the code but we apply via the synthed code to decouple
> config generation from config apply.

And, on how this relates to the orphaned resources plan:

> The IaC solution might also be a part of our orphaned resources solution.
> But we still need a separate orphaned resources solution to make sure
> every live resource gets on boarded to IaC including the GitHub repo
> settings and Claude settings etc....

### What is configured by hand today

| Vendor | Settings made in a dashboard | Where it is tracked now |
| --- | --- | --- |
| Auth0 | Applications for cookbook and cahokia: callback, web origin and logout URLs, application type | Two human issues in `issues/human/` |
| Cloudflare | 5 Pages projects (created on first deploy by `free-tiers.sh pages-project`), API token scopes, Access policies (planned for pane), later Workers, D1 and DNS | `wrangler.toml` names; token scopes nowhere |
| GitHub | Repo settings ("Automatically delete head branches", a human task in the cleanup plan), branch protection, labels such as `needs-human`, environments, Actions secrets | Nowhere |
| Claude | Project MCP connectors (read-only), cloud environment network hosts and setup script, scheduled tasks (PR triage, digest, deep review) | Prose in AGENTS.md and project memory |

The eval engine plan (decision 0003, phase 2) adds a `standards/settings.toml`
manifest so the weekly digest asks Jim to re-check these by hand. The
resource cleanup plan ([PR #669](../plans/2026-11-resource-cleanup.md))
adds claims, a weekly sweep of what is live, and a safe deletion order.
Infrastructure as code (IaC) is the third piece: it makes the settings
themselves code, so most of those checks and dashboard visits go away.

### Requirements

From Jim's ask and the repo rules:

1. **Rust, CDK style.** Code in Rust, with reusable building blocks
   ("constructs"), not a DSL. No HCL.
2. **Synth, then apply.** The Rust program writes a declarative file; a
   separate step plans and applies that file. Generation and apply are
   decoupled, and what is applied is exactly what was reviewed.
3. **$0.** Free tiers and open source only
   ([free-tiers.md](../free-tiers.md)). No new paid vendor.
4. **Fewest vendors**, and **well supported by Claude**
   ([tech radar](../tech-radar.md) selection criteria).
5. **Providers** for Cloudflare, Auth0 and GitHub at least.
6. **State** stored for free, with locking.
7. **Secrets** never in the repo, and as few as possible in state.
8. **Drift detection** that feeds the cleanup sweep and the eval engine's
   human checks.
9. **No local clone.** Plan and apply run in GitHub workflows.

## Findings

Checked 2026-11-26 (sources at the end):

- **CDK for Terraform (CDKTF) is dead.** HashiCorp archived it on
  2025-12-10. It supported TypeScript, Python, Java, C# and Go, never Rust.
  Its community fork, **CDK Terrain** (Open Constructs Foundation), is in
  Thoughtworks' Assess ring and supports TypeScript, Python and Go, still
  no Rust, and still needs Node for JSII.
- **Pulumi has no Rust SDK.** Its SDKs are TypeScript, Python, Go, .NET,
  Java and YAML. Its state service, Pulumi Cloud, is a new vendor.
- **Terraform and OpenTofu both read JSON configuration** (`*.tf.json`),
  with the same meaning as HCL. That is the declarative artifact CDKTF
  itself synthesized, so a Rust program can do what CDKTF did by writing
  `*.tf.json`. Every Terraform provider works with it.
- **OpenTofu** (Linux Foundation, MPL 2.0, latest 1.13) is the open source
  fork of Terraform. It has **built-in state encryption** (AES-GCM, with a
  key from a PBKDF2 passphrase or a KMS) that works with every backend,
  which Terraform does not. It reads the same providers from its own
  registry, with no account.
- **Terraform** itself is under the Business Source License, and its hosted
  state service, HCP Terraform, ended its legacy free plan on 2026-03-31;
  today's free tier allows 500 managed resources. It would be a new vendor
  (HashiCorp, now IBM) for something we can do without one.
- **Providers** exist and are maintained by the vendors: `cloudflare/cloudflare`
  (v5, generated from Cloudflare's OpenAPI schema; it had a rough 2025, so
  pin it and import carefully), `auth0/auth0` (Okta) and
  `integrations/github` (GitHub).
- **Rust crates that generate typed Terraform config** (checked on
  crates.io, 2026-11-26):
  - `terrars` (ISC): generates strongly typed Rust structs and builders
    from a provider's JSON schema (`terrars-generate`), and the stack
    writes `.tf.json`. It describes itself as "an alternative to the CDK".
    Version 0.4.0 was released on 2026-10-25, with about 40,000 downloads;
    one maintainer, and "usable, but may have some rough edges". It is the
    one live option, and the closest thing to "Pulumi or CDK, in Rust".
  - `tf-bindgen`: the same idea with a CDK-style builder API, but stuck at
    0.1.0 since 2023-04-16, "early in development, expect breaking changes".
  - `tfschema-bindgen`: generates Rust types from a provider schema for
    reading it; last release 0.1.5 on 2021-02-22.
  Generated bindings for a whole provider can be large (Cloudflare's v5
  schema is huge), so build time in CI and sandboxes is the thing to check.
- **Crossplane** reconciles continuously from inside a Kubernetes cluster.
  We have no cluster, and none of our approved vendors offers one free.
- **Auth0 allows one wildcard in the left-most subdomain** of a callback
  URL, matching one level. Pages previews are `<branch>.<project>.pages.dev`
  and `<hash>.<project>.pages.dev`, one level under the project, so
  `https://*.jimhillr-cookbook.pages.dev` covers every preview of one
  project and nothing else. Auth0 says wildcards should not be used in
  production; here the wildcard is only for previews, and only our deploy
  token can publish under our project's subdomain.
- **Claude support.** Claude knows Terraform and its providers very well,
  and Rust well. There is an OpenTofu MCP server (registry search and
  resource docs, `@opentofu/opentofu-mcp-server`, hosted at
  `mcp.opentofu.org`) and HashiCorp's Terraform MCP server for registry
  docs; both are read-only, which is how the radar says to enable them.
  Crossplane, Pulumi and the Rust crates have less.

## Options

Every option below is free in money. The differences are vendors, Rust,
effort and risk. Claude build effort is a rough count of agent PRs, which
is what spends the $25 a month Claude Code budget.

| | Option | Rust, CDK style | Synth then apply | Money | Free-tier impact | New vendor | Claude support | Build effort |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| **A** | **Rust synth to `.tf.json`, OpenTofu plans and applies** | Yes: our constructs, typed structs | Yes: committed `.tf.json`, saved plan applied | $0 | Small (table below) | None | Strong: Terraform providers, OpenTofu MCP, Rust | About 4 PRs for phase 1, 10 in all |
| A1 | **A, with `terrars`-generated typed bindings** (default) | Yes: generated types for every resource and attribute | Yes | $0 | Same | None | Same | Fewest structs to write; risk is a single-maintainer crate and build time on large providers |
| A2 | A, with hand-written typed structs | Yes, for the resources we write | Yes | $0 | Same | None | Same | About ten structs to start, more upkeep as resources grow |
| B | OpenTofu with hand-written `.tf.json`, no Rust | No: verbose JSON, no constructs | Yes | $0 | Same | None | Strong | About 3 PRs, but every change is hand-edited JSON |
| C | Terraform CLI, HCP Terraform for state | Only via A or B | Yes | $0 up to 500 resources | 500-resource cap | HashiCorp | Strong | Same as A plus an account |
| D | CDK Terrain (CDKTF fork) | No Rust; TypeScript, Python or Go | Yes | $0 | Same | None | Moderate | Same as A, in a language we hold |
| E | Pulumi | No Rust SDK | Partly (preview, then up) | $0 self-managed state; Pulumi Cloud free tier is a new vendor | - | Pulumi (for state) | Good | Large, in a language we hold |
| F | Crossplane | No (YAML) | No: continuous reconcile | An always-on Kubernetes cluster, which no approved vendor offers free | Breaks "free tiers only" | A Kubernetes host | Weak | Large |
| G | Our own Rust reconciler on vendor APIs | Yes | Yes: our own plan format | $0 | Same | None | Moderate (Claude knows the APIs, not our tool) | 15 or more PRs: create, read, update, delete, import and diff for every resource, then keep up with API changes |
| H | Each vendor's own tool (wrangler, Auth0 Deploy CLI, GitHub settings app or `gh api` scripts) | No | Mixed | $0 | Same | A GitHub app for settings | Mixed | Moderate, but three tools, Node, and no shared plan |

Free-tier impact of option A:

| Resource | Use | Limit |
| --- | --- | --- |
| GitHub Actions | One plan per PR that touches `infra/`, one plan and apply per merge, one drift plan a week; a minute or two each | Free on this public repo |
| Auth0 Management API tokens | About one machine-to-machine token per plan or apply, so 30 to 60 a month | 1,000 a month on Free |
| Cloudflare API | Tens of requests a run | 1,200 per 5 minutes |
| GitHub API | Tens of requests a run | 5,000 an hour |
| State | A few kilobytes in a git branch | - |

### Why not the others

- **B** meets everything but Rust and CDK style, which is what Jim asked
  for. It is still where A lands: A's output *is* B, so B is the escape
  hatch if the Rust crate is ever abandoned.
- **C** adds a vendor, a license we don't need and a resource cap, for no
  gain over OpenTofu.
- **D and E** have no Rust and bring Node or another runtime back.
- **F** needs a running cluster, which costs money and breaks "free tiers
  only".
- **G** is the "build" option. It gives full Rust, but re-implements what
  the three providers already do and would need upkeep forever. A keeps
  this door open: the Rust model of what should exist is ours, so if
  OpenTofu ever stops fitting, a Rust applier can read the same model.
- **H** splits one job across three tools and has no single plan to
  review.

## Decision

Option A. Each item is a default, so Jim can answer "yes to all".

1. **A new `infra` project (Rust) is the source of truth.** It holds
   constructs that match how we think, such as `WebApp::new("cookbook")
   .sign_in(Auth0App::import("<client id>"))`, which expands to the Pages
   project and the Auth0 application's callback, origin and logout URLs for
   production, previews (`https://*.<project>.pages.dev`) and localhost.
   `cargo run -- synth` writes `infra/out/*.tf.json`, one file per stack
   (`auth0`, `cloudflare`, `github`), with sorted keys so diffs are clean.
2. **Strongly typed bindings generated by `terrars`** (A1), so every
   resource and attribute is a Rust type checked at compile time, pinned in
   `Cargo.lock`, and generated from the same provider versions OpenTofu
   pins. Phase 1 starts with a spike on the `auth0` stack: adopt `terrars`
   if the generated crate builds in CI and in a sandbox in a reasonable
   time and `tofu validate` accepts its output; generate only the
   resource types we use if a whole provider is too big. If the spike
   fails, or `terrars` stops being maintained, fall back to hand-written
   serde structs for the ten or so resource types we use (A2); our
   constructs sit on top either way, so the switch doesn't touch them.
   **We support `terrars`** (Jim, 2026-11-26): "if we find bugs in terrars
   it is a library we are willing to support and commit code to." A bug
   we hit is fixed upstream with a pull request to `andrewbaxter/terrars`,
   not worked around in our code; until it is released, `infra` uses a
   git dependency on the fixed commit, noted in its `Cargo.toml`. Its
   single maintainer is a reason to contribute, not a reason to drop it.
3. **The synthesized `.tf.json` is committed**, and CI fails if it doesn't
   match a fresh synth, the same way the tech radar's tables are generated
   and checked. That is the reviewed, declarative artifact, and applying it
   needs no Rust build. This is an exception to mise.md's "no build
   artifacts in git", which this decision adds there. *Alternative:* synth
   in CI and pass the JSON between jobs as a workflow artifact.
4. **OpenTofu, not Terraform**, pinned in `infra/mise.toml`, with providers
   pinned in a committed lock file.
5. **Plan on every PR, apply on merge.** PR CI runs `tofu plan` with
   read-only tokens and writes the plan to the job summary. On a push to
   `main`, a job in a protected `infra` environment (main only) plans
   again, saves the plan file, and a second job applies exactly that file
   (`tofu apply plan.tfplan`). Merging the PR is the approval to apply;
   there is no extra click. All infra jobs share one Actions `concurrency`
   group, which is the lock.
6. **Plan guards in Rust** read `tofu show -json` of every plan and fail it
   when:
   - IAC-001: it deletes or replaces anything (see 8);
   - IAC-002: any value marked sensitive is set (secrets stay out of state);
   - IAC-003: it would create resources past a free-tier budget (Pages
     projects, Workers, Cron Triggers, from the same budgets as
     `free-tiers.sh`), or anything that can bill (R2, paid plans);
   - IAC-004: it uses a provider or resource type that isn't allowlisted.
7. **State: encrypted, in this repo, on its own branch.** OpenTofu state
   encryption (AES-GCM, key from a PBKDF2 passphrase in the
   `TOFU_STATE_PASSPHRASE` secret) with the local backend; the apply job
   commits the encrypted state to an `infra-state` branch that only that
   job writes, and that the cleanup sweep lists as kept. With IAC-002 the
   state holds only IDs and settings that are already public in the
   synthesized JSON, so encryption is defence in depth. *Alternatives:* an
   HTTP backend on a small Cloudflare Worker with KV (private and still free,
   but one more thing to build and secure, and it can't manage itself), or
   HCP Terraform (new vendor).
8. **IaC never destroys.** Removing a resource from the code adds a
   `removed` block (forget it, don't destroy it) and a `[[retired]]` entry
   in `standards/resources.toml`, so deletion stays with the cleanup plan's
   `all-retire-resources.yml`, in its safe order and with Jim's approval.
   Removing a URL or setting inside a resource (an Auth0 callback, say) is
   an update, not a deletion, and applies normally; that is the safe
   order's step 2, done for us.
9. **Secrets.** Provider credentials are GitHub environment secrets: a
   read-only set for PR plans and a write set in the `infra` environment
   for apply. Secret values (API tokens, Actions secrets, client secrets)
   are not managed by IaC, because their values would land in state; Jim
   creates and rotates them, and they stay human checks. Revisit with
   OpenTofu's ephemeral values when a provider supports them for these
   resources.
10. **Drift detection, weekly.** `tofu plan -refresh-only
    -detailed-exitcode` for every stack runs in the existing
    `all-free-tier-usage.yml` job, next to the cleanup sweep. Drift
    (exit code 2) turns the run red and goes into the sweep's report.
    Fixing drift is a PR: either change the code to match a deliberate
    dashboard change, or re-apply.
11. **Order of work.** Auth0 first (it closes the two Auth0 human issues
    in `issues/human/`), then GitHub repo settings (delete head branches on
    merge, and the `main` ruleset's required checks), then Cloudflare (import the Pages projects, then
    Access for pane). Every existing resource is brought in with `import`
    blocks, and the first apply of each stack must plan zero changes
    other than the intended ones.
12. **Tech radar.** OpenTofu and "Rust-synthesized Terraform JSON" go in
    Assess now and move to Trial when this is accepted and phase 1 merges.
    CDKTF and CDK Terrain, Pulumi, Crossplane, and Terraform with HCP
    Terraform go in Hold, with the reasons above.

## Amendment: no secrets in state (Jim, 2026-11-26)

Jim: "Never commit secrets to git or store them in tf state." This tightens
items 6 and 9:

- **IAC-002 fails on any sensitive value in state**, not only one our
  configuration sets: a token or client secret the vendor generates
  fails too. Write-only attributes (which never reach state) pass. The few
  attributes a provider marks sensitive that hold nothing secret (Auth0's
  public signing certificates) are listed with their reason in
  `infra/src/guard.rs`.
- **IaC never mints or copies a secret.** A token IaC creates would have its
  value in state, so tokens and their GitHub secrets stay made by hand.
  IaC may still change settings around them where that keeps the value out
  of state.
- **Where state lives (item 7) is reopened**, because a branch of the
  public repo is still git. The options and their costs are put to Jim
  separately; until he answers, nothing is applied. Answered in
  [0009](0009-opentofu-state-on-r2.md): state lives in R2.

## How it fits the cleanup plan and the eval engine

Jim's rule: IaC feeds the orphaned-resources work, but that stays a
separate system, whose job includes making sure everything live is
onboarded to IaC.

- **"Not in IaC" is a sweep finding.** The cleanup plan's claims table
  gains the synthesized `.tf.json` (and later the IaC state) as a claim
  source. The weekly sweep sorts every live resource and setting into
  **managed by IaC**, **claimed elsewhere** (`wrangler.toml` content
  deploys, `resources.toml` `[[keep]]` with a reason and review date),
  **retired** or **orphan**. "Live but not managed by IaC" is reported as
  its own finding, so the onboarding list is always visible and shrinks as
  stacks land; a `[[keep]]` entry needs a reason why something can't be in
  IaC (a secret value, say).
- **The sweep reads more than the providers manage.** It lists GitHub repo
  settings, rulesets, labels, environments and the names of secrets
  (never values), and Claude settings, so a setting nobody wrote down
  still shows up.
- **Claude settings get the same synth, with a different applier.** No
  Terraform provider covers Claude. The `infra` crate also synthesizes
  `infra/out/claude.json`: the project's connectors (and that they are
  read-only), the cloud environment's allowed network hosts and setup
  script, and the scheduled tasks with their schedules. An agent skill,
  `claude-settings`, compares it with what the Claude tools report (the
  environment and trigger reads are available to agents) and reports
  drift in the weekly digest; changing a setting is done by an agent after
  the PR that changes `claude.json` is approved, or by Jim where only a
  person can (connectors). Same pattern: reviewed declarative file, separate
  apply.
- **Human checks shrink.** Every setting that moves into IaC leaves the
  eval engine's `standards/settings.toml` (or is marked `managed = "infra"`)
  and is checked by the weekly drift plan instead of by Jim. What stays a
  human check is what IaC can't safely hold: API token scopes, secret
  rotation, payment methods and billing, and Claude connectors.
- **Threat check.** Its `[auth] callbacks` and the synthesized Auth0 URLs
  must agree; threat check phase 2 decides which reads which. Until then
  the `infra` crate is the source.
- **The cleanup plan's phase 4 Auth0 read** uses the same read-only Auth0
  token as the PR plans, so there is one token, not two.

## Phases

### Phase 1: the crate, Auth0, and the pipeline (about 4 PRs)

- `infra` project: constructs, the synth, `.tf.json` output for an `auth0`
  stack that imports the cookbook and cahokia applications and sets their
  production, preview and localhost URLs; plan guards IAC-001 to IAC-004
  with tests; `mise run ci` runs synth check, `tofu validate` and the
  guards.
- Workflows: PR plan, main plan and apply in the `infra` environment, the
  `infra-state` branch, and the weekly drift plan.
- Docs: `standards/docs/infra.md` (how to add a resource, the guards, how
  to fix drift), the mise.md exception, AGENTS.md rule ("settings made in
  a dashboard go into `infra` when a provider covers them").
- Human tasks (Jim, one time): create a read-only and a write Auth0
  machine-to-machine application for the Management API, and set
  `TOFU_STATE_PASSPHRASE`. These replace the Auth0 dashboard tasks.

### Phase 2: GitHub (about 2 PRs)

- A `github` stack: repo settings (delete head branches on merge), the
  `main` ruleset, labels, the `infra` environment. Human task: a
  fine-grained token with Administration on this repo only, read and write
  variants, with a yearly rotation human check.

### Phase 3: Cloudflare (about 2 PRs)

- A `cloudflare` stack: import the Pages projects (wrangler still deploys
  their content), Access for pane, and later Workers, D1 and DNS as they
  appear. `free-tiers.sh pages-project` stays as a guard until IaC creates
  every Pages project; then IAC-003 replaces it.
- Human task: a Cloudflare token with Pages and Access edit for apply, and
  read for plans.

### Phase 4: onboarding everything (about 2 PRs)

- The sweep's "not in IaC" finding and the `claude.json` synth with the
  `claude-settings` skill, so every remaining hand-made setting is either
  managed, kept with a reason, or retired.

## Cost

$0 in money: open source tools, standard runners on a public repo, and
well under each vendor's free API limits (table above). No new vendor. The
cost is Claude build effort: about 10 agent PRs over the four phases,
phase 1 being about 4, spread over the month's $25 budget alongside other
work. Each phase can stop where it is and still be useful.

## Consequences

- The Auth0 dashboard tasks and the GitHub settings tasks become code,
  in exchange for a one-time set of tokens Jim creates per vendor.
- Dashboard changes to managed settings show up as drift within a week
  and are reverted unless the code is changed to match.
- `infra/out/` is committed generated output, checked in CI.
- OpenTofu becomes a tool the repo depends on, installed by mise.
- If the Rust crate is ever dropped, the committed `.tf.json` keeps
  working with OpenTofu on its own.

## Sources

Read 2026-11-26.

- CDKTF: [hashicorp/terraform-cdk](https://github.com/hashicorp/terraform-cdk)
  (archived 2025-12-10);
  [CDK Terrain on the Thoughtworks Radar](https://www.thoughtworks.com/radar/tools/cdk-terrain).
- HCP Terraform:
  [continuing HCP Terraform's enhanced free tier](https://www.hashicorp.com/blog/continuing-hcp-terraform-s-enhanced-free-tier-experience).
- OpenTofu: [state encryption](https://opentofu.org/docs/language/state/encryption),
  [OpenTofu MCP server](https://mcpservers.org/servers/opentofu/opentofu-mcp-server).
- Rust crates: [terrars](https://github.com/andrewbaxter/terrars),
  [tf-bindgen](https://docs.rs/crate/tf-bindgen/latest),
  [tfschema-bindgen](https://crates.io/crates/tfschema-bindgen),
  versions and dates from the crates.io API.
- Auth0: [wildcards for subdomains](https://auth0.com/docs/get-started/applications/wildcards-for-subdomains).
