# Plan: royalty, automated royalties for open source code

Started 2026-11-27 from Jim's request:

> Royalty is intended to be an open source solution for monetizing libraries
> and frameworks people build. The intent is that someone would put a
> royalty.toml in their project and then somehow whenever somebody uses their
> code it results in automated royalty. Not sure the architecture. [...] The
> user config design is too priority and paramount. An mvp implementation
> might just track metrics for a library in a consumers code and file a
> report or something for manual action. A long term solution might involve
> implementing code that reads logs or even runs inline in the code to live
> pass money. The main goal of the royalty.toml is for a producer of code to
> define how they would like to be paid. Setting up free tiers, or dev tiers.
> As well as maximum spend per consumer, per person, per organization as well
> as maximum income the producer is every expecting. The trade offs are we
> want a free barier to entry for a code integrator and a ceiling to an in
> production user ensure their arent cost overruns. Balancing code design
> with performance impact is paramount. All solutions should not affect a
> consumers performance as many libraries and frameworks might be in the hot
> loop. Maybe royalty.toml has config on how and when it runs to gather data.
> I even expect a mode that is just honor code where you collect your own
> data and send money (no tech enforcement just honesty and lawsuits if it
> ever got that serious).

Status: **approved** (Jim, 2026-11-27: "All recommended"). Nothing is
built yet. Jim took the recommended option of every question in
[Decisions for Jim](#decisions-for-jim); the work items are the issue files
in [`../../issues/`](../issues/README.md).

## Summary

- A producer adds a `royalty.toml` to their library. It says who gets paid,
  what is free, what production use costs, and the most anyone can ever be
  charged.
- A consumer runs one command in their own repo. It reads their dependency
  graph, finds every `royalty.toml` that came with their dependencies, works
  out what is owed under the tiers and caps, and writes a report. Nothing
  runs inside their program.
- The MVP stops there: the consumer pays by hand from the report. Later
  phases add an honor-code ledger, counting from CI, deploys and logs, and
  last of all automatic payment.
- Three promises hold in every phase: trying a library is free, a production
  bill has a hard ceiling that both sides can set, and the hot path of the
  consumer's program is never touched unless the consumer turns that on.
- Every phase up to live payment costs $0 and uses only GitHub. Live payment
  needs a payment vendor, which is a decision for Jim.

## Goals

1. A producer can state how they want to be paid in one small, readable
   file, without writing code or running a service.
2. An integrator can add, build, test and evaluate a library for free, with
   no sign-up and no configuration.
3. A production user knows the most they can owe before they adopt a
   dependency, and can set their own lower ceiling.
4. Measuring usage never slows the consumer's program.
5. The whole system works with no enforcement at all (the honor code), and
   each stronger mode is an add-on, never a rewrite.
6. It runs on files and free tiers: no server, no database and no account is
   needed for the MVP.

## Non-goals

- **No runtime kill switch.** Royalty never stops, slows or degrades a
  program for non-payment, in any phase. Disputes are settled by people, and
  in the end by the licence and the courts, as Jim said.
- **No DRM or licence keys.** Nothing is locked.
- **Not a payment processor.** Royalty works out amounts and hands them to an
  existing way of paying.
- **Not a new licence.** Royalty links to whatever licence or agreement the
  producer chose. It does not write one (see [Legal notes](#legal-notes)).
- **Not a registry.** There is no central list of producers or consumers in
  the MVP.

## Actors

| Actor | Who they are | What they need |
| --- | --- | --- |
| **Producer** | Writes and publishes a library or framework. | Say how to be paid, once, in a file. Get paid without chasing people. |
| **Integrator** | A developer trying or building on the library. | Zero cost and zero paperwork until real production use. |
| **Production operator** | Whoever runs the consumer's software for real users. | No performance cost, no surprise bill, nothing that can break production. |
| **Organization** | The legal entity that pays. May own many repos and teams. | One total across all its projects, a budget, and caps that cover the whole organization. |
| **Person** | One human: a solo developer who pays for themself, or one seat inside an organization. | A small personal ceiling. Never billed twice through two projects. |
| **Payee** | The account that receives money. Often the producer, sometimes several maintainers. | To be identified without a secret, and not be impersonated. |

A project is often both a producer and a consumer. One file name,
`royalty.toml`, covers both roles.

## The core trade-offs

| Tension | How this design resolves it |
| --- | --- |
| Free to start, but paid in production | Non-production use (development, tests, CI, staging) is always free, by rule of the format and not by the producer's choice. With no consumer configuration at all, a scan reports and assumes non-production. Paying starts only when the consumer declares production use. |
| Producers want income, consumers fear overruns | Ceilings come from both sides and the lowest one wins. The producer sets caps per project, per person, per organization and per payer. The consumer sets a budget per producer and in total. Terms are pinned to the dependency version, so a price can only change when the consumer upgrades and reviews the change. |
| Accurate usage, but zero performance cost | Usage is measured from things that already exist outside the running program: the lockfile, git history, CI runs, deploy events and logs. Billing units are chosen so these are enough. An in-process counter is the last resort, off by default, compiled out unless the consumer enables it, and held to a hard overhead budget by a benchmark gate. |
| Simple honesty, but room for enforcement | Three modes on one schema: `honor`, `report`, `metered`. Each adds evidence, none changes how amounts are calculated. |
| Paying many producers, with deep dependency trees | By default only direct dependencies are billable. The consumer's total budget caps the whole tree whatever producers ask for. |

## Design rules

These are the project's first requirements (Jim agreed, 2026-11-27). They
move to `docs/requirements.md` as `ROYL-001` onwards (a requirements PR is
`needs-human`).

| # | Rule |
| --- | --- |
| 1 | **No hot-path cost by default.** With default settings, royalty adds no code, no dependency and no I/O to the consumer's built program. |
| 2 | **Non-production use is free.** No tier can charge for the `development`, `test`, `ci` or `staging` environments. |
| 3 | **Royalty never breaks a running program.** No mode can stop or slow a program at run time. Only the consumer's own CI check can fail, and only when the consumer asks it to. |
| 4 | **The lowest ceiling wins.** The amount owed is never above any producer cap that applies, and the report always shows which cap was used. |
| 5 | **Offline and private by default.** The MVP makes no network calls. Nothing leaves the consumer's machine unless the consumer sends it. |
| 6 | **Deterministic.** The same files give the same report, byte for byte, so two people can check each other's numbers. |
| 7 | **Old files stay valid.** A `royalty.toml` with only `[project]` is valid for ever and means "no payment terms". |
| 8 | **No secrets.** A `royalty.toml` holds public identifiers only: account names, never tokens, keys or bank details. |
| 9 | **Integer money.** Amounts are whole US cents and are never floating point. |

## The `royalty.toml` design

This is the most important part of the plan. The format is schema version 1,
and its reference now lives in [royalty-toml.md](../royalty-toml.md): every
table and field with its type, default and meaning, how the amount owed is
worked out (with the exact arithmetic), what `royalty check` reports, how
the existing files map, and the worked examples, including
[what Acme owes in one month](../royalty-toml.md#what-acme-owes-in-one-month).

In short: a producer's file has `[project]`, `[license]`, `[[payee]]`,
`[terms]`, `[[tier]]`, `[caps]`, `[income]` and `[measure]`, and ships
inside the published package. A consumer's file has `[project]` and
`[consumer]` with its budget, policy and deployments, at the root of their
repository. A file with only `[project]`, like every one in this monorepo
today, means "no payment terms".

## Measurement architecture

The rule is simple: **measure outside the program**. The sources, in the
order the tools prefer them:

| # | Source | What it gives | When it runs | Cost to the consumer's program |
| --- | --- | --- | --- | --- |
| 1 | **Dependency graph** (`cargo metadata` and `Cargo.lock`; later `package-lock.json`) | Which packages, which versions, direct or transitive, normal or dev-only. Each package's `royalty.toml` is read from the package source already on disk. | When the consumer runs the scan: by hand, or in CI. | None. The program is not involved. |
| 2 | **Git history** | Seats: distinct commit authors in the period. | With the scan. | None. |
| 3 | **Declared numbers** | Deployments, revenue, custom units. | Written in the consumer's `royalty.toml`. | None. |
| 4 | **CI and deploy events** (Phase 3) | Builds and releases. A workflow step appends one line to a counts file in the repo. | Once per build or deploy. | None. Adds well under a second to a CI job. |
| 5 | **Log sampling** (Phase 3) | Calls. A separate command reads a sample of logs the consumer already keeps (access logs, platform analytics exports). | Offline, on a schedule, on a different machine if wanted. | None. |
| 6 | **In-process counter** (Phase 3, opt-in) | Calls, when there are no logs. | Inside the program. | Budgeted and gated: see below. |

The billing units were chosen so that sources 1 to 3 cover `flat`,
`project`, `seat`, `deployment`, `revenue` and `custom`. A producer who
sticks to those gets a complete system in the MVP with no run-time part at
all, and the plan recommends them in the documentation.

### The in-process counter, if it is ever built

It exists only for producers who truly need per-call pricing where no logs
exist. It is held to these rules, and question 9 asks whether to build it at
all.

| Rule | Detail |
| --- | --- |
| **Off unless both sides opt in** | The producer sets `[measure] runtime = "opt-in"`. The consumer sets `[consumer.measure] runtime = true` and enables a Cargo feature. Without the feature the counter is compiled out completely: no code, no dependency, no symbol. A test compares the built output with and without the producer's integration to prove it. |
| **One add, nothing else** | A counted call does one increment of a per-thread counter. No lock, no allocation, no system call, no clock read, no formatting, no shared cache line (counters are padded). |
| **Sampling without randomness** | "One in N" is a per-thread countdown, not a random number. |
| **Batched and off-thread** | A background thread adds up the per-thread counters every 15 minutes (configurable) and at exit, and writes one line to a local file. It makes no network call. A separate process ships or reports the file. |
| **Fails silent** | The counter never panics, never blocks and never returns an error to the caller. If the file cannot be written the data is dropped. |
| **A hard budget** | At most 2 nanoseconds added per counted call, and at most 1% slower on a tight-loop benchmark, on the CI runner. |
| **A benchmark gate** | A benchmark in the project's CI measures both numbers on every change to the counter crate. Over budget fails the build. The results are committed as files. |
| **Never enforces** | It counts. It cannot stop or slow anything (design rule 3). |

## Modes

All three modes use the same file and the same arithmetic. They differ in
the evidence.

| Mode | Who counts | What the producer sees | Payment | Phase |
| --- | --- | --- | --- | --- |
| `honor` | The consumer, any way they like. The scanner is optional. | Only the money. | The consumer sends it. | 1 (the format) and 2 (the ledger) |
| `report` | The scanner, on the consumer's machine. | A report file the consumer sends to `report_to`. | By hand, from the report. | 1 (the MVP) |
| `metered` | The scanner plus CI, deploy, log or run-time counts. | A signed report, sent automatically if the consumer allows. | Automatic, within the caps and the budget. | 3 and 4 |

A producer's `mode` is what they ask for. A consumer's
`[consumer.policy] modes` is what they will do. If they do not overlap, the
scan says so and falls back to `honor`.

## Phases

### Phase 0: agree the design

Jim answers the questions below. The schema reference moves from this plan
to `docs/royalty-toml.md`, and the design rules become
`docs/requirements.md`.

### Phase 1: the MVP, a scanner and a report

A Rust command-line tool, `royalty`, in this project. Two crates, the same
shape as `cost` and `evals`:

| Crate | Job |
| --- | --- |
| `royalty_model` | The schema types, the parser, validation and the calculation. Pure functions, no I/O, so other tools (pane, a web page) can reuse it. |
| `royalty_cli` | The `royalty` binary. |

| Command | What it does |
| --- | --- |
| `royalty check` | Validates a `royalty.toml`. With a `[consumer.budget]`, also fails when the amount owed would exceed it. |
| `royalty scan` | Reads the consumer's Cargo dependency graph, finds each dependency's `royalty.toml`, and prints what is owed. |
| `royalty report` | Writes the result as `royalty-report.json` (for tools) and `royalty-report.md` (for people): one row per producer family with tier, units, caps applied, amount, and how to pay. |

No network, no account, no payment. The consumer reads the report and pays
by hand. The monorepo is the first user: the audit validates every
`royalty.toml`, and a scan of one of our own projects is committed as an
example.

### Phase 2: honor code and reports that hold up

- `royalty.lock`: the terms and payees the consumer last accepted, with a
  hash. A dependency update that changes a payee or raises a price shows as
  a diff a person must accept. This is the main defence against a hijacked
  payee.
- A payments ledger file, so year caps, carried-over small amounts and
  trial periods work across periods.
- Merging the reports of several repositories under one `[consumer] id`, so
  organization caps apply across them.
- Payee verification against the producer's repository.
- `royalty-status.toml` for the income ceiling.
- A reusable GitHub workflow that comments on a pull request with the change
  in what is owed.
- An npm scanner.
- Payment links in the report.

### Phase 3: metered

CI and deploy counters, the offline log reader, signed reports, and (only if
Jim says yes to question 9) the in-process counter with its benchmark gate.

### Phase 4: live payment

The report becomes a payment instruction that a payment rail carries out,
inside the consumer's budget. This is the first phase that needs a vendor
beyond GitHub, so it starts with a decision record for Jim.

## Payment rails

| Rail | Vendor status | Money cost | Fits |
| --- | --- | --- | --- |
| **GitHub Sponsors** (a link in the report) | GitHub is approved. Each payee must enrol, which needs a person, a bank account and tax forms. | $0 to us. GitHub takes no fee from sponsorships paid by personal accounts, and up to 6% from those paid by organizations (check before relying on it). | Phases 1 and 2. Manual. One-time and monthly amounts. |
| **Invoice and bank transfer** | No vendor. | $0 to us. Bank fees are the payer's. | Phases 1 and 2. Larger amounts. |
| **Any link the producer gives** | The producer's choice, not ours. | $0 to us. | Any phase. |
| **x402** (HTTP 402 with stablecoin settlement) | **New vendor** (a facilitator and a wallet). Needs Jim's sign-off. | Advertised as near zero per payment; no monthly fee. Tax and accounting effort is real. | Phase 4. Machine-to-machine, very small amounts. |
| **Stripe** | **New vendor.** Needs Jim's sign-off. | No monthly fee; about 2.9% plus 30 cents per card payment, so poor for small amounts. | Phase 4. Cards and invoices. |
| **Open Collective** or another fiscal host | **New vendor.** Needs Jim's sign-off. | About 10% of money received. | Phase 4, for projects with no legal entity. |

The revenue plan for this repo's own APIs in PR #764 (HTTP 402 and x402,
GitHub Sponsors) covers the same rails from the seller's side. If it is
approved, royalty's Phase 4 should reuse its 402 handling and its vendor
decision instead of making a second one. Nothing in Phases 1 to 3 depends
on it.

## Privacy, trust and abuse

| Risk | What happens | Defence |
| --- | --- | --- |
| **Forged usage** (a consumer under-reports) | The producer is underpaid. | Accepted in `honor` and `report` modes: that is what the honor code means. The report is deterministic, so a producer who is shown the inputs can recompute it. Signed reports and counted events come in Phase 3. The last resort is the producer's licence or agreement. |
| **Inflated usage** (a producer's package over-counts) | The consumer is overcharged. | The consumer's own tools do all the counting, from the consumer's own files. A producer's code never reports usage. Caps and the budget bound the damage. |
| **Spoofed payee** (a fork, a typosquat or a hijacked release changes who is paid) | Money goes to an attacker. | The terms ship inside the package, so they are covered by the lockfile checksum. `royalty.lock` makes a payee change a reviewed diff. Phase 2 checks that a payee's GitHub account owns or maintains `repository`. Reports show the payee's full identity. |
| **Price jump** | A new release raises the price. | Terms are pinned to the version in use. An upgrade that costs more shows in `royalty check` on the pull request, and fails it when over budget. |
| **Dependency confusion for profit** (someone publishes junk packages with terms and gets them pulled in transitively) | Unwanted bills. | `reach = "direct"` is the default, `[consumer.policy] transitive` can ignore the rest, and the budget is a hard stop. |
| **Consumer data leaking** | A report shows what a company uses and how big it is. | The MVP is offline. A report holds package names, counts and amounts only: no source code, no file paths, no names of people, no IP addresses. Revenue is a band or an amount the consumer chooses to declare. The consumer decides whether to send a report at all. |
| **Seat counting exposing people** | Commit emails are personal data. | The report holds only the count. Emails are read locally and never written out. |
| **Secrets in config** | A token committed by mistake. | The schema has no field for one (design rule 8), and `royalty check` rejects values that look like keys. |

## Legal notes

This is not legal advice. A lawyer should review any terms a producer
relies on before real money moves.

- **Open source licences and royalties.** The Open Source Definition says a
  licence may not require a royalty. So a mandatory royalty tied to the
  licence (`binding = "license"`) makes the code source-available, not open
  source. `voluntary` requests and separate agreements beside an open
  licence (`agreement`, as in dual licensing) keep the code open. This is
  why `voluntary` is the default and question 1 is the first question.
- **Royalty is not a contract.** A `royalty.toml` describes terms. The
  binding text is the licence or agreement that `[license] terms` points to.
- **Tax.** Money received is income for the payee, and payers may need
  invoices or tax forms. Royalty prints amounts and payees; it does not
  handle tax. GitHub Sponsors and similar rails collect tax forms
  themselves.
- **Consumer declarations.** Class, revenue band and seats are the
  consumer's own statements. Under an `agreement` or `license` binding, a
  false one is a breach of that agreement, not something the tool polices.
- **This project's own licence** follows the monorepo's `LICENSE.md`.

## Prior art

Royalty borrows from several existing ideas and differs from each.

| Existing | What it does | What royalty adds |
| --- | --- | --- |
| `.github/FUNDING.yml`, npm's `funding` field | Say where to donate. | Amounts, tiers, caps and a calculation. |
| Dependency-based donation splitters | Split a consumer's chosen budget across their dependency tree. | The producer states terms; the consumer's budget is one input, not the only one. |
| Dual licensing and source-available licences | Make commercial users pay. | A machine-readable statement of the price and its ceilings, and the free non-production rule. |
| Subscription bundles for maintainers | Pay maintainers through a middleman. | No middleman and no account for the MVP. |

## Cost of each phase

Money is what the phase costs us to build and run. Build effort is Claude
Code time, which counts against the $25 a month budget.

| Phase | Money | Free-tier use | Build effort |
| --- | --- | --- | --- |
| 0: agree the design | $0 | None. | Small. Two docs PRs. |
| 1: MVP scanner and report | $0 | GitHub Actions on standard runners for the project's CI (unlimited on a public repo). No other service. | Medium. About eleven small PRs. |
| 2: honor code and reports | $0 | The same, plus read-only GitHub API calls made by the consumer's own token when they choose to verify a payee. Enrolling in GitHub Sponsors, if Jim wants our own projects paid, is free and is a human task. | Large. About eleven PRs. |
| 3: metered | $0 | The same. The benchmark gate runs on standard runners. | Large. The counter and its gate are the costly part; without them, medium. |
| 4: live payment | Depends on the rail (see Payment rails). $0 a month for every option listed; fees are a share of money moved. | A new vendor's free plan, if Jim approves one. | Large, and it needs a decision record first. |

`docs/cost.toml` stays at "$0 a month, no outside service" through Phase 3.

## Decisions for Jim

**Answered (Jim, 2026-11-27): "All recommended."** Each question below
takes its recommended option:

1. `binding` defaults to `voluntary`; `agreement` and `license` are opt-in.
2. Non-production use is always free, as a rule of the format.
3. Caps: consumer is any payer, person and organization can override it,
   plus a per-project cap.
4. Only direct dependencies are billable unless a producer asks for
   `transitive`.
5. At the income ceiling, everyone is free for the rest of the window.
6. The MVP scans Cargo only; npm comes in Phase 2.
7. The tool ships as binaries on GitHub Releases.
8. Reports point to GitHub Sponsors, invoices and plain links first.
9. The in-process counter is decided at the start of Phase 3, not now.
10. Our own libraries keep "no terms" until Phase 2 works, then one
    library trials `voluntary` terms.
11. Phase 1 goes ahead.

The questions as they were asked follow.

Every option costs $0 in money unless it says otherwise. No option here
gets near a free-tier limit: the only service used before Phase 4 is GitHub
Actions on standard runners, which is unlimited on a public repo. "Effort"
is Claude Code build time against the $25 a month budget.

1. **What should the default `binding` be: is a royalty a request, or a
   condition?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. `voluntary` by default, with `agreement` and `license` available **(recommended)** | A file with prices is a request unless the producer opts into something stronger. The project stays truthfully "open source". | $0 | No extra |
   | B. `agreement` by default | Producers get enforceable terms out of the box, but we must ship a template agreement, and that needs a lawyer. | $0 to build; legal review is a cost outside the repo, likely hundreds of dollars | Small extra |
   | C. Only `voluntary` in schema 1 | The simplest and safest. Producers who want enforceable terms cannot say so in the file. | $0 | Slightly less |

2. **Is non-production use always free, as a rule of the format?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Yes, always, and producers cannot turn it off **(recommended)** | The "free barrier to entry" is a guarantee an integrator can rely on without reading each file. | $0 | No extra |
   | B. Free by default, but a producer may charge for it | More freedom for producers; integrators must read every file. | $0 | Small extra |

3. **What do "per consumer, per person, per organization" caps mean?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Consumer is any payer; person and organization are the two kinds of payer and can override it; plus a per-project cap **(recommended, as drafted)** | Simple to compute and to explain. A per-seat ceiling is `max_units` on a seat price. | $0 | No extra |
   | B. A strict hierarchy: person is one seat, consumer is one project, organization is everything | Closer to "per head" thinking, but needs seat-level accounting in every report. | $0 | Small extra |

4. **Who is billable by default in a dependency tree?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Direct dependencies only, and a producer may ask for `transitive` **(recommended)** | No surprise bills from deep trees. Low-level libraries must opt in to be found. | $0 | No extra |
   | B. Everything in the tree | Low-level libraries are paid without doing anything; consumers face long bills and lean on their budget. | $0 | No extra |

5. **When a producer reaches their income ceiling, what happens by
   default?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Everyone is free for the rest of the window **(recommended)** | Matches "the maximum income the producer is ever expecting". Rests on the producer publishing an honest status file. | $0 | Small (Phase 2) |
   | B. The ceiling is information only | Nothing to build or trust; the ceiling has no effect. | $0 | None |

6. **Which ecosystems does the MVP scan?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Cargo only, npm in Phase 2 **(recommended)** | Most of this repo is Rust, so we can test on ourselves at once. | $0 | Medium (the Phase 1 total) |
   | B. Cargo and npm in the MVP | A wider audience sooner. | $0 | One or two more PRs |

7. **How do people get the `royalty` tool?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Binaries on GitHub Releases, built by our CI **(recommended)** | GitHub is approved. Counts against the release-asset budget (2 GB a file; we would use a few megabytes). | $0 | Small |
   | B. Also publish the crates to crates.io | `cargo install royalty` works, and others can use the parser. crates.io is free but is a **new vendor** needing sign-off, an account and a token (a human task and a minted secret). The name `royalty` may be taken. | $0 | Small, plus a human task |
   | C. Source only for now | People build it themselves. | $0 | None |

8. **Which way of paying does the report point to first?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. GitHub Sponsors, invoice and plain links **(recommended)** | No new vendor. Royalty only prints where to pay. | $0 to us. GitHub keeps up to 6% of organization-paid sponsorships | Small (Phase 2) |
   | B. Add Stripe or x402 now | Opens the way to automatic payment sooner, but is a **new vendor** before there is anything to pay for. | $0 a month; about 2.9% plus 30 cents a card payment (Stripe) or near zero (x402) | Large, and a decision record |

9. **Should the in-process counter ever be built?**

   | Option | What it means | Money | Effort |
   | --- | --- | --- | --- |
   | A. Not now; decide at the start of Phase 3 from what producers ask for **(recommended)** | Per-call pricing works from logs or declared numbers until then. The hot path is untouched by construction. | $0 | None now |
   | B. Commit to it in Phase 3, under the overhead budget and benchmark gate | True per-call metering where there are no logs. | $0 | Large |
   | C. Never | The strongest performance promise. Per-call pricing stays log-based or declared for ever. | $0 | None |

10. **Should this monorepo's own libraries ask for royalties?**

    | Option | What it means | Money | Effort |
    | --- | --- | --- | --- |
    | A. Keep every file as "no terms" until Phase 2 works, then add `voluntary` terms to one library as a trial **(recommended)** | We test on ourselves without asking anyone for money early. Being paid needs Jim to enrol in GitHub Sponsors (free; a human task with a runbook, added when the time comes). | $0 | Small |
    | B. Add terms to our libraries in Phase 1 | Real example files sooner, but they would name a payee account that does not exist yet. | $0 | Small, plus the human task now |
    | C. Never; royalty is for other people | Nothing to do. We lose our best test case. | $0 | None |

11. **Go ahead with Phase 1?**

    | Option | What it means | Money | Effort |
    | --- | --- | --- | --- |
    | A. Yes, with the recommended answers above **(recommended)** | The Phase 1 issues become workable. Later phases stay in the backlog. | $0 | Medium, about eleven small PRs |
    | B. Docs only for now | Phase 0 happens; no code. | $0 | Small |
