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
in [`../../issues/`](../../issues/README.md).

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

This is the most important part of the plan. The format is schema version 1.

### One file, two roles

| Role | Tables it uses | Where the file lives |
| --- | --- | --- |
| Producer | `[project]`, `[license]`, `[[payee]]`, `[terms]`, `[[tier]]`, `[caps]`, `[income]`, `[measure]` | At the root of the published package, and shipped inside it, so a consumer has it on disk as soon as the dependency is downloaded. |
| Consumer | `[project]`, `[consumer]` and its sub-tables | At the root of the consumer's repository. It is never read by anyone but the consumer's own tools. |

Shipping the producer's file inside the package matters. It means the
scanner needs no network and no registry, and it means the terms are fixed to
the version the consumer actually uses.

### Top level

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `schema` | integer | `1` | The format version. A reader refuses a number it does not know. Within one version, new optional fields may be added, and readers warn about (but accept) fields they do not know, so a typo is seen but a newer file still works. `royalty check --strict` turns those warnings into errors. |

### `[project]` (required)

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | The project's name. |
| `description` | string | required | One sentence. |
| `repository` | URL | none | The source repository. Used to check that a payee belongs to the project. |
| `packages` | list of strings | `[name]` | The published package names these terms cover, for a repository that publishes several. |
| `family` | string | `name` | Packages with the same `family` and the same payees are billed once and share one set of caps. This stops a framework split into ten crates from charging ten times. |

This is the whole of every `royalty.toml` in the monorepo today, so all of
them are valid schema 1 files as they stand. See
[How the existing files map](#how-the-existing-files-map).

### `[license]`: how payment relates to the licence

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `code` | SPDX expression | none | The licence of the code, such as `MIT OR Apache-2.0`. Information only. |
| `binding` | `voluntary`, `agreement` or `license` | `voluntary` | How strong the request is. See the table below. |
| `terms` | path or URL | none | The human-readable terms. Required unless `binding` is `voluntary`. |

| `binding` | Meaning | Still open source? |
| --- | --- | --- |
| `voluntary` | The amounts are a request. The code licence is unchanged and nobody is in breach for not paying. | Yes. |
| `agreement` | Payment is owed under a separate agreement the consumer accepts, such as a commercial licence offered beside a copyleft one (dual licensing) or a support contract. | The code can be, under its open licence. |
| `license` | Payment is a condition of the code licence itself. | No. An OSI-approved licence cannot require a royalty, so this is "source available". The tools say so plainly. |

A consumer's budget can scale down a `voluntary` amount. It can never reduce
an `agreement` or `license` amount; it can only flag it (see
[How the amount is worked out](#how-the-amount-is-worked-out)).

### `[[payee]]`: who is paid

One or more. No secrets: only public account names.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Who this is. |
| `share` | integer | `1` | A weight. Each payee gets `share / sum of shares` of every payment. |
| `github` | string | none | The payee's GitHub user or organization. Used to verify the payee against `repository`. |
| `[[payee.method]]` | table | at least one | A way to pay this payee, in the payee's order of preference. |

| `method.kind` | Other fields | Notes |
| --- | --- | --- |
| `github-sponsors` | `account` | The GitHub Sponsors account. GitHub is an approved vendor. |
| `invoice` | `contact` (an email or URL) | The consumer asks for an invoice and pays by bank transfer. No vendor. |
| `link` | `url` | Any payment page the producer already has. The tools only print it. |
| `x402` | `network`, `address` | Reserved for Phase 4. A public receiving address only. |

Other kinds can be added within schema 1. A reader that does not know a kind
skips it and uses the next.

### `[terms]`: the general terms

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `mode` | `honor`, `report` or `metered` | `honor` | The evidence the producer asks for. See [Modes](#modes). |
| `currency` | string | `USD` | Only `USD` in schema 1. |
| `period` | `month`, `quarter` or `year` | `month` | How often amounts are worked out and settled. Every price is "per period". |
| `reach` | `direct` or `transitive` | `direct` | `direct`: billable only when the consumer's own manifest names the package. `transitive`: billable anywhere in the dependency graph. |
| `trial_days` | integer | `0` | Days of free production use, counted from the first report that shows production use. |
| `minimum_payment_cents` | integer | `500` | Amounts below this roll into the next period, so nobody pays 12 cents by card. A balance still below it at the end of the year is dropped. |
| `report_to` | email or URL | none | Where a consumer sends a report in `report` mode. |

### `[[tier]]`: free, dev and paid tiers

Tiers are checked in file order for each use of the package, and the **first
one that matches applies**. A use that matches no tier is free. One tier is
built in and always checked first: the non-production tier of design rule 2.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `id` | string | required | A short name, shown in reports. |
| `title` | string | none | A human-readable name. |
| `when` | table | matches everything | The conditions. All of them must hold. |
| `free` | boolean | `false` | The tier costs nothing. A tier with no `price` is also free. |
| `[[tier.price]]` | table | none | One or more prices. They add up. |

Conditions in `when`. Where a value is a list, any one entry matching is
enough.

| Condition | Values | Comes from |
| --- | --- | --- |
| `environment` | `production` (the others are always free) | The consumer's declared stage and deployments. |
| `entity` | `person`, `organization` | `[consumer] entity`. |
| `class` | `commercial`, `open-source`, `nonprofit`, `education`, `government` | `[consumer] class`. |
| `revenue_below_usd` | integer | `[consumer] annual_revenue_below_usd`, a band the consumer declares. |
| `seats_at_most` | integer | Measured seats (see units). |
| `dependency` | `direct`, `transitive` | The dependency graph. |

Prices. The price is `cents` for every `per` units, in each period.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `unit` | see below | required | What is counted. |
| `cents` | integer | required (except `revenue`) | Whole US cents. |
| `per` | integer | `1` | How many units `cents` buys. `cents = 100, per = 1000000` is one dollar per million. |
| `included` | integer | `0` | Units that are free before charging starts. |
| `max_units` | integer | none | Units beyond this are not charged. |
| `basis_points` | integer | none | Only for `unit = "revenue"`: hundredths of a percent. |
| `describe` | string | none | Required for a `custom:` unit: what one unit is. |

### Billable units

Every unit says how it is measured, and none of the first six needs any code
in the consumer's program.

| Unit | One unit is | Measured from | Hot-path cost | Available |
| --- | --- | --- | --- | --- |
| `flat` | The consumer using the package at all in the period. | The dependency graph. | None | Phase 1 |
| `project` | One consumer project (one lockfile) that depends on the package. | The dependency graph. | None | Phase 1 |
| `seat` | One person who committed to a consumer project that depends on the package, in the period. Bots are left out. The consumer can declare the number instead. | `git log`, or `[consumer] seats`. | None | Phase 1 |
| `deployment` | One production deployment whose build contains the package. | The consumer's `[[consumer.deployment]]` list. | None | Phase 1 |
| `build` | One CI build that compiled the package. | A CI step that appends to a counts file. | None | Phase 3 |
| `release` | One deploy to production of a build that contains the package. | The deploy workflow. | None | Phase 3 |
| `call` | One call or request, estimated by sampling. | Offline log sampling first; the opt-in run-time counter only if the consumer enables it. | None for logs; budgeted for the counter | Phase 3 |
| `revenue` | A share, in basis points, of the revenue the consumer attributes to the product. | Declared by the consumer. Never measured. | None | Phase 1 (declared) |
| `custom:<name>` | Whatever `describe` says, such as gigabytes processed. | Declared by the consumer. | None | Phase 1 (declared) |

Registry downloads ("installs") are deliberately not a unit. They are
inflated by CI caches and mirrors, are trivial to forge, and say nothing
about real use. `project` is the honest version of the same idea.

### `[caps]`: the most a producer will ever charge

Each cap has a limit per period and a limit per calendar year. Leave one out
and it does not apply.

| Table | Limits the total owed to this producer by |
| --- | --- |
| `[caps.project]` | One consumer project (one repository or lockfile). |
| `[caps.person]` | One person who pays for themself. |
| `[caps.organization]` | One organization, across every project it owns. |
| `[caps.consumer]` | Any single payer. It is the fallback when the more specific cap for a person or an organization is not set. |

| Field | Type | Meaning |
| --- | --- | --- |
| `period_cents` | integer | The most owed in one period. |
| `year_cents` | integer | The most owed in one calendar year. |

A ceiling on what one seat can cost is written on the price itself, with
`max_units`. Question 3 asks Jim to confirm this reading of "per consumer,
per person, per organization".

### `[income]`: the most the producer expects to earn

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `ceiling_cents` | integer | none | The most the producer expects to receive in one window, from everyone together. |
| `window` | `year` or `lifetime` | `year` | What the ceiling covers. |
| `on_reach` | `free` or `continue` | `free` | `free`: once the ceiling is reached, nobody owes anything for the rest of the window. `continue`: the ceiling is information only. |
| `status` | path or URL | `royalty-status.toml` in `repository` | A small public file where the producer records what they have received in the window. |

No consumer can see a producer's income, so the ceiling rests on the
producer's honesty, in the same way payment rests on the consumer's. The
producer keeps `royalty-status.toml` up to date (Phase 2 gives them a command
for it), and consumers who choose to go online read it. In Phase 1 the
ceiling is printed in the report and nothing more.

### `[measure]`: how and when usage is gathered

The producer says what evidence they accept. The consumer says what they
allow (`[consumer.measure]`). The scanner uses only what both allow, and a
unit that cannot be measured that way falls back to a number the consumer
declares.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `sources` | list of `lockfile`, `git`, `declared`, `ci`, `deploy`, `logs`, `runtime` | `["lockfile", "git", "declared"]` | The evidence the producer accepts, weakest cost first. |
| `sample_one_in` | integer | `1000` | For `call`: count one in this many and multiply up. |
| `runtime` | `never` or `opt-in` | `never` | Whether the package offers the in-process counter at all. There is no "required" value, by design rule 1. |

When it runs is the consumer's choice, because it is the consumer's machine:
see `[consumer.measure] run`.

### `[consumer]`: the consumer's side

A consumer needs none of this to try a library. They add it when they go to
production, or when they want a budget.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `id` | string | none | A stable name for the payer, such as `github:acme`. Reports with the same `id` are added together before organization caps apply. |
| `entity` | `person` or `organization` | `person` | Who pays. |
| `name` | string | none | Shown in reports. |
| `class` | see tier conditions | `commercial` | Self-declared. |
| `annual_revenue_below_usd` | integer | none | A self-declared band, such as `1000000`. Never an exact figure. |
| `stage` | `development` or `production` | `development` | Whether this project runs in production. This one line is the switch from free to paid. |
| `seats` | integer | counted from git | Overrides the count. |

`[[consumer.deployment]]`, one per production deployment:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | The service's name. |
| `manifest` | path | The manifest (such as `services/api/Cargo.toml`) whose dependency graph is deployed. Only packages in that graph count for this deployment. |

`[consumer.budget]`, the consumer's own ceiling:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `period_cents` | integer | none | The most this consumer will pay to all producers together in one period. |
| `year_cents` | integer | none | The same for a calendar year. |
| `per_producer_period_cents` | integer | none | The most for any one producer family in one period. |
| `on_exceed` | `fail` or `scale` | `fail` | `fail`: `royalty check` exits non-zero, so a pull request that would break the budget is stopped before it merges. `scale`: voluntary amounts are scaled down in proportion to fit. |

`[consumer.policy]`, what the consumer accepts:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `bindings` | list | `["voluntary", "agreement", "license"]` | A dependency whose `binding` is not listed fails `royalty check`. A company that cannot take on paid licence terms lists only `voluntary`. |
| `modes` | list | `["honor", "report"]` | The modes the consumer will take part in. |
| `transitive` | `pay`, `report` or `ignore` | `pay` | What to do about producers with `reach = "transitive"`: pay them, list them without paying, or leave them out. `report` and `ignore` can only reduce `voluntary` amounts. |
| `review_above_cents` | integer | `0` | A change in terms that raises the period total by more than this must be accepted by a person (Phase 2). |

`[consumer.measure]`:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `sources` | list | `["lockfile", "git", "declared"]` | The evidence sources the consumer allows. |
| `run` | `manual`, `ci` or `schedule` | `manual` | When the scan runs: by hand, on every pull request, or on a timer in the consumer's CI. |
| `runtime` | boolean | `false` | Allow the in-process counter. Both this and the producer's `runtime = "opt-in"` are needed. |

`[[consumer.package]]`, an exception for one dependency:

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | The package or family. |
| `action` | `decline`, `cap` or `declare` | `decline` a voluntary request, `cap` it at `period_cents`, or `declare` a number for a unit that is not measured. |
| `period_cents`, `unit`, `units` | | The values for `cap` and `declare`. |
| `reason` | string | Why. Shown in the report. |

### How the amount is worked out

The steps are fixed, so every reader gets the same answer.

1. **Find the packages.** Read the consumer's dependency graph. Keep each
   package that ships a `royalty.toml` with `[terms]`. Group them by
   `family`. If two versions of a family are present, the newer version's
   terms apply and it is counted once.
2. **Drop what is out of reach.** A family with `reach = "direct"` that no
   consumer manifest names is listed as "not billable (transitive)".
3. **Pick the environment.** A dependency used only for development or builds
   (a Cargo `dev-dependency`) is non-production. So is everything when
   `stage = "development"`. Non-production is free: stop here.
4. **Pick the tier.** The first tier whose `when` matches. None: free.
5. **Price it.** For each price: units, minus `included`, limited to
   `max_units`, times `cents`, divided by `per`. Sums are kept in millionths
   of a cent and rounded down to a whole cent once, per family per period.
6. **Apply the producer's caps.** The project cap, then (after adding up the
   consumer's projects that share an `id`) the person or organization cap,
   falling back to the consumer cap. Year caps use the year-to-date total
   from earlier reports.
7. **Apply the income ceiling**, if the consumer has read a `status` file
   that says it is reached and `on_reach = "free"`.
8. **Apply the consumer's budget.** The per-producer limit, then the total.
   A `voluntary` amount can be scaled down. An `agreement` or `license`
   amount is never reduced: it is marked "over budget" and `royalty check`
   fails, so the consumer removes the dependency, negotiates, or raises the
   budget, before the change merges.
9. **Hold back small amounts.** Anything under `minimum_payment_cents` is
   carried to the next period.

The report shows every step for every family, including which cap or budget
set the final number.

### Transitive dependencies

Deep dependency trees are where cost overruns would come from, so the
defaults are cautious.

- **The end consumer pays, not the library in the middle.** A library is not
  a production operator. It never owes anything just for depending on
  another library.
- **Direct by default.** With `reach = "direct"`, a producer is paid only by
  consumers who chose them by name. Adding one framework cannot bring a
  hundred bills with it.
- **A producer can ask for `transitive` reach.** A low-level library that is
  only ever used indirectly needs this. The consumer's
  `[consumer.policy] transitive` and total budget still decide.
- **Counted once.** However many paths lead to a family, and however many of
  its packages are present, it is billed once.
- **The consumer's total budget covers everything.** It is the last step and
  no producer setting can exceed it without `royalty check` failing first.
- **Reserved for later:** an `[[upstream]]` table, so a framework that
  already pays its own dependencies can say so and the consumer is not asked
  twice. It is not in schema 1.

### How the existing files map

About twenty projects in this monorepo have a file like this, as
[configuration.md](../../../standards/docs/configuration.md) requires:

```toml
[project]
name = "auth_sdk"
description = "Generic library for producing auth flows in applications, targeting OAuth2 and OpenID Connect providers."
```

| In the file | Read as |
| --- | --- |
| No `schema` | Schema 1. |
| `[project]` with `name` and `description` | Unchanged. |
| No `[terms]`, `[[payee]]` or `[[tier]]` | "No payment terms." The package is free for everyone, and a scan lists it under "no terms". |
| No `[consumer]` | A consumer in `development` stage with no budget. A scan of the project reports and owes nothing. |

So no migration is needed, and none of those files changes in this plan. A
project that wants to be paid later adds tables; it never has to rewrite
what is there.

### Worked examples

#### 1. A free hobby library

The author wants nothing but would take a coffee.

```toml
schema = 1

[project]
name = "tinycolor"
description = "Parse and convert colours."
repository = "https://github.com/sam-example/tinycolor"

[license]
code = "MIT"
binding = "voluntary"

[[payee]]
name = "Sam Example"
github = "sam-example"
[[payee.method]]
kind = "github-sponsors"
account = "sam-example"

[terms]
mode = "honor"

# Everyone is free. Companies in production are asked for a dollar a month.
[[tier]]
id = "suggested"
when = { environment = ["production"], entity = ["organization"] }
[[tier.price]]
unit = "flat"
cents = 100

[caps.consumer]
year_cents = 1200

[income]
ceiling_cents = 60000        # "I never expect more than $600 a year"
```

#### 2. A framework: free to build on, paid in production

```toml
schema = 1

[project]
name = "webframe"
description = "A web framework."
repository = "https://github.com/webframe-example/webframe"
packages = ["webframe", "webframe_macros", "webframe_router"]
family = "webframe"

[license]
code = "AGPL-3.0-only"
binding = "agreement"        # or use it under the AGPL and owe nothing
terms = "COMMERCIAL-TERMS.md"

[[payee]]
name = "Webframe maintainers"
github = "webframe-example"
[[payee.method]]
kind = "github-sponsors"
account = "webframe-example"
[[payee.method]]
kind = "invoice"
contact = "billing@webframe.example"

[terms]
mode = "report"
period = "month"
reach = "direct"
trial_days = 90
report_to = "reports@webframe.example"

# Development, tests, CI and staging are free by rule. No tier needed.

[[tier]]
id = "community"
title = "Open source, education, nonprofits and small companies"
when = { class = ["open-source", "education", "nonprofit"] }
free = true

[[tier]]
id = "small"
when = { revenue_below_usd = 1000000 }
free = true

[[tier]]
id = "production"
when = { environment = ["production"] }
[[tier.price]]
unit = "deployment"
cents = 2000                 # $20 a month for each production deployment
included = 1                 # the first one is free

[caps.project]
period_cents = 10000         # $100 a month for one project
[caps.organization]
period_cents = 50000         # $500 a month for a whole organization
year_cents = 500000

[income]
ceiling_cents = 25000000     # $250,000 a year pays the team; after that it is free
on_reach = "free"
```

#### 3. Honor code only

No tool is needed on either side. The file is a public statement of what the
producer thinks is fair.

```toml
schema = 1

[project]
name = "geocalc"
description = "Geodesic maths."

[license]
code = "Apache-2.0"
binding = "voluntary"

[[payee]]
name = "Geocalc project"
[[payee.method]]
kind = "link"
url = "https://geocalc.example/pay"

[terms]
mode = "honor"
period = "year"

[[tier]]
id = "commercial"
when = { environment = ["production"], class = ["commercial"] }
[[tier.price]]
unit = "revenue"
basis_points = 10            # 0.1% of the revenue you attribute to the product

[caps.consumer]
year_cents = 500000          # never more than $5,000 a year from anyone

[measure]
sources = ["declared"]       # count it yourself, however you like
```

#### 4. Per-seat pricing for organizations, with caps

```toml
schema = 1

[project]
name = "formkit"
description = "Form building toolkit."
repository = "https://github.com/formkit-example/formkit"

[license]
code = "LicenseRef-Formkit-Source-Available"
binding = "license"
terms = "LICENSE.md"

[[payee]]
name = "Ana Example"
share = 70
github = "ana-example"
[[payee.method]]
kind = "github-sponsors"
account = "ana-example"

[[payee]]
name = "Ben Example"
share = 30
github = "ben-example"
[[payee.method]]
kind = "github-sponsors"
account = "ben-example"

[terms]
mode = "report"
period = "month"
minimum_payment_cents = 1000

[[tier]]
id = "solo"
when = { entity = ["person"] }
free = true

[[tier]]
id = "team"
when = { environment = ["production"], entity = ["organization"] }
[[tier.price]]
unit = "seat"
cents = 500                  # $5 a seat a month
included = 3                 # the first three seats are free
max_units = 100              # never charged for more than 100 seats

[caps.project]
period_cents = 20000
[caps.organization]
period_cents = 40000         # $400 a month, however many projects and seats
year_cents = 400000
[caps.person]
year_cents = 0               # a person never pays

[measure]
sources = ["lockfile", "git", "declared"]
```

#### 5. A consumer that caps its own spend

```toml
[project]
name = "acme_shop"
description = "Acme's web shop."

[consumer]
id = "github:acme-example"
entity = "organization"
name = "Acme Example Ltd"
class = "commercial"
annual_revenue_below_usd = 5000000
stage = "production"

[[consumer.deployment]]
name = "shop-api"
manifest = "services/api/Cargo.toml"

[[consumer.deployment]]
name = "shop-worker"
manifest = "services/worker/Cargo.toml"

[consumer.budget]
period_cents = 4500          # $45 a month to all producers together
year_cents = 50000
per_producer_period_cents = 4000
on_exceed = "scale"

[consumer.policy]
bindings = ["voluntary", "agreement"]
transitive = "report"

[consumer.measure]
run = "ci"
```

#### What Acme owes in one month

Acme has 14 people committing and two production deployments, and its
`webframe` trial is over. It depends directly on `webframe` (example 2) and
on `chartlib`, a voluntary library that asks $5 a seat a month with a $40
project cap. It depends indirectly on `tinycolor` (example 1).

| Family | Tier | Units | Before caps | After producer caps | After Acme's budget | Why |
| --- | --- | --- | --- | --- | --- | --- |
| `webframe` | production | 2 deployments, 1 included | $20.00 | $20.00 | $20.00 | An agreement: never scaled. |
| `chartlib` | team | 14 seats | $70.00 | $40.00 | $25.00 | The producer's project cap, then scaled (it is voluntary) to fit Acme's $45 total. |
| `tinycolor` | not billable | | $0 | $0 | $0 | Transitive, and its reach is `direct`. Listed only. |
| **Total** | | | $90.00 | $60.00 | **$45.00** | The budget holds. |

If `webframe` alone had come to more than $45, nothing would be scaled:
`royalty check` would fail on the pull request that caused it.

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
