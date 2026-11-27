# The `royalty.toml` reference

This page is the reference for the `royalty.toml` format, schema version 1:
every table and field, how a package's file is found, how the amount owed
is worked out, what `royalty check` reports, and worked examples. The why
behind the design (the actors, the trade-offs, the measurement architecture
and the phases) stays in the
[royalty plan](plans/2026-11-royalty-plan.md), which Jim approved on
2026-11-27 with the recommended answer to every decision. Where a rule
comes from one of those decisions, this page says so.

The format follows the plan's [design rules](plans/2026-11-royalty-plan.md#design-rules),
which become `ROYL-001` onwards in `requirements.md`. The ones that shape
the file most:

- **Nothing in the consumer's program.** No field can make royalty add code,
  a dependency or I/O to a consumer's built program. The only run-time
  counter is opt-in on both sides and does not exist yet (decision 9).
- **Non-production use is free.** No tier can charge for development,
  tests, CI or staging (decision 2).
- **The lowest ceiling wins.** Producer caps and the consumer's budget all
  apply, and the report says which one set the number.
- **Old files stay valid.** A file with only `[project]` means "no payment
  terms", for ever.
- **No secrets.** Every value is public: account names, never tokens, keys
  or bank details.
- **Integer money.** Amounts are whole US cents.

A change to the format changes this page in the same PR. A change that
alters what a consumer owes, or answers one of the plan's decisions
differently, is labelled `needs-human`.

## What works in which phase

Every field below is part of schema 1 from the start, so a producer can
write their full terms today. Some only take effect in a later phase of the
plan; until then the tools read them, check them and show them in the
report, but don't apply them.

| Field or feature | Takes effect |
| --- | --- |
| Everything not listed below | Phase 1 |
| `trial_days`, `year_cents` across more than one period, amounts carried under `minimum_payment_cents` | Phase 2 (needs the payments ledger) |
| Organization caps across several repositories that share a `[consumer] id` | Phase 2 (needs `royalty report --merge`) |
| `[income] on_reach` | Phase 2 (needs `royalty-status.toml`) |
| `[consumer.policy] review_above_cents` | Phase 2 (needs `royalty.lock`) |
| Units `build`, `release`, `call`; sources `ci`, `deploy`, `logs` | Phase 3 |
| Source `runtime`, `[measure] runtime`, `[consumer.measure] runtime` | Only if Jim says yes at the start of Phase 3 (decision 9) |
| Payee method `x402` | Phase 4 |

## One file, two roles

| Role | Tables it uses | Where the file lives |
| --- | --- | --- |
| Producer | `[project]`, `[license]`, `[[payee]]`, `[terms]`, `[[tier]]`, `[caps]`, `[income]`, `[measure]` | At the root of the published package, and shipped inside it, so a consumer has it on disk as soon as the dependency is downloaded. |
| Consumer | `[project]`, `[consumer]` and its sub-tables | At the root of the consumer's repository. It is never read by anyone but the consumer's own tools. |

Shipping the producer's file inside the package matters. It means the
scanner needs no network and no registry, and it means the terms are fixed to
the version the consumer actually uses.

## Top level

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `schema` | integer | `1` | The format version. A reader refuses a number it does not know. Within one version, new optional fields may be added, and readers warn about (but accept) fields they do not know, so a typo is seen but a newer file still works. `royalty check --strict` turns those warnings into errors. |

## `[project]` (required)

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

## `[license]`: how payment relates to the licence

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

## `[[payee]]`: who is paid

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

## `[terms]`: the general terms

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `mode` | `honor`, `report` or `metered` | `honor` | The evidence the producer asks for. See [Modes](plans/2026-11-royalty-plan.md#modes). |
| `currency` | string | `USD` | Only `USD` in schema 1. |
| `period` | `month`, `quarter` or `year` | `month` | How often amounts are worked out and settled. Every price is "per period". |
| `reach` | `direct` or `transitive` | `direct` | `direct`: billable only when the consumer's own manifest names the package. `transitive`: billable anywhere in the dependency graph. |
| `trial_days` | integer | `0` | Days of free production use, counted from the first report that shows production use. |
| `minimum_payment_cents` | integer | `500` | Amounts below this roll into the next period, so nobody pays 12 cents by card. A balance still below it at the end of the year is dropped. |
| `report_to` | email or URL | none | Where a consumer sends a report in `report` mode. |

## `[[tier]]`: free, dev and paid tiers

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

## Billable units

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

## `[caps]`: the most a producer will ever charge

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
`max_units`. This reading of "per consumer, per person, per organization"
is Jim's answer to decision 3 in the plan.

## `[income]`: the most the producer expects to earn

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

## `[measure]`: how and when usage is gathered

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

## `[consumer]`: the consumer's side

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

## How the amount is worked out

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
set the final number. [Exact rules](#exact-rules) has the arithmetic.

## Exact rules

The steps above are the order. These are the details every reader must get
the same, so two tools (or two people) always agree to the cent.

### Where a producer's file is read from

- The scanner reads `royalty.toml` from the directory that holds the
  package's manifest, in the package source already on disk (for Cargo, the
  registry cache, a git checkout or a path dependency). It never downloads
  anything.
- The file applies to a package only when the package's name is in
  `[project] packages` (which defaults to `[name]`). Names are compared with
  `-` and `_` treated as the same. A file that doesn't name the package it
  sits beside is ignored, and the scan says so, so a vendored or copied file
  can't put terms on someone else's package.
- A producer must ship the file in the published package. Cargo includes
  every file in the package directory unless `include` or `exclude` in
  `Cargo.toml` leaves it out, so a producer who uses `include` adds
  `"royalty.toml"` to it.
- A package with no `royalty.toml`, or with one that has no `[terms]` and no
  `[[tier]]`, has "no payment terms". It is listed under "no terms" and owes
  nothing.

### Families

- Packages whose files give the same `family` and the same set of payees
  (by `name` and `github`) are one family. The same `family` with different
  payees is two families, and the scan warns, because that is what a
  hijacked or forked package looks like.
- When a family appears at several versions, the terms of the highest
  version (by semver) apply, once.

### Direct and transitive

- A package is **direct** when the manifest of any member of the consumer's
  workspace names it as a normal dependency, or names another package of
  the same family. Otherwise it is **transitive**.
- A package reached only through `dev-dependencies` or
  `build-dependencies` is non-production, wherever it sits.
- `reach = "direct"` (the default, decision 4) makes a transitive family
  "not billable (transitive)". With `reach = "transitive"` it is billable,
  subject to `[consumer.policy] transitive`.

### Periods

- A report covers one calendar month, in UTC. Seats and other counts are
  taken over the producer's own period that ends in that month.
- Seats are the distinct author emails (compared ignoring case) of the
  non-merge commits that touch the consumer's project directory, by author
  date in the period. Authors whose name or email marks them as a bot
  (`[bot]`, `dependabot`, `github-actions`, `renovate`) are left out. Emails
  are read only to tell people apart and are never written out. Outside a
  git repository, seats are "not measured" unless `[consumer] seats` says.
- A family with `period = "month"` is due every month. With `quarter` it is
  due in March, June, September and December, and with `year` in December,
  for the whole of its period. A family that is not due in the report's
  month is listed with "not due this month" and $0.
- The consumer's `period_cents` budget and `per_producer_period_cents`
  limit apply to what is due in the report's month, whatever each
  producer's period is. Size them for the months when quarterly and yearly
  amounts fall due.

### Tier conditions

A condition that is left out always holds. A condition whose input the
consumer hasn't given never holds, so missing information can only ever
move a consumer to a later tier in the file, and the producer decides
what that is by the order of their tiers.

| Condition | Holds when |
| --- | --- |
| `environment` | The use is production. The only allowed value is `production`; any other value is an error, because non-production use never reaches the tiers. |
| `entity` | `[consumer] entity` (default `person`) is in the list. |
| `class` | `[consumer] class` (default `commercial`) is in the list. |
| `revenue_below_usd` | The consumer declared `annual_revenue_below_usd` and it is at most this value. A consumer who declares no band never matches. |
| `seats_at_most` | The measured or declared seat count is at most this value. If seats can't be counted, it does not hold. |
| `dependency` | The package is `direct` or `transitive`, as above. |

### Prices

For each `[[tier.price]]`, with `units` the measured or declared count:

```text
charged = max(0, min(units, max_units) - included)     # max_units absent: no limit
amount  = charged * cents * 1_000_000 / per            # millionths of a cent
```

For `unit = "revenue"`, `units` is the revenue the consumer declares for
the period, in whole cents, and:

```text
amount = units * basis_points * 1_000_000 / 10_000      # millionths of a cent
```

- `max_units` limits the units counted, before `included` is taken off:
  example 4's 100-seat limit with 3 seats included charges at most 97 seats.
- All arithmetic is on unsigned 128-bit integers, and each division rounds
  down. The prices of the chosen tier are added in millionths, then the
  total is rounded down to whole cents once, per family per period.
- A unit that can't be measured from the sources both sides allow, and that
  the consumer hasn't declared with `[[consumer.package]] action =
  "declare"`, is "not measured": the price adds $0 and the report says so.
  Nothing is ever guessed.
- `deployment` with no `[[consumer.deployment]]` in a production project is
  one deployment: the project itself.

### Caps and budget

After pricing, for each family in turn, in this order:

1. **`[[consumer.package]]`**: `decline` sets a `voluntary` amount to $0;
   `cap` lowers it to `period_cents`. Neither can lower an `agreement` or
   `license` amount; `royalty check` warns and the amount stands.
2. **`[consumer.policy] transitive`**: for a billable transitive family,
   `report` and `ignore` set a `voluntary` amount to $0 (`report` still
   lists it). An `agreement` or `license` amount stands.
3. **Producer caps**: the lower of `[caps.project] period_cents` and the
   payer cap, where the payer cap is `[caps.person]` for a consumer whose
   `entity` is `person` and `[caps.organization]` for an organization, each
   falling back to `[caps.consumer]` (decision 3). Within one table, the
   lower of `period_cents` and what is left of `year_cents` applies. Until
   the Phase 2 ledger exists, nothing has been paid earlier in the year, so
   `year_cents` limits the one period on its own.
4. **Per-producer budget**: `per_producer_period_cents` lowers a
   `voluntary` amount; an `agreement` or `license` amount above it is
   "over budget".

Then across all families:

5. **Total budget**: with `on_exceed = "scale"`, when the total is over
   `period_cents`, the `agreement` and `license` amounts are kept whole and
   the space left (`period_cents` minus them) is shared among the
   `voluntary` amounts in proportion: each becomes
   `amount * space / voluntary_total`, rounded down. If the space is
   negative, every voluntary amount becomes $0 and the result is "over
   budget". With `on_exceed = "fail"`, nothing is scaled and the result is
   "over budget".
6. **Small amounts**: a family's final amount under its
   `minimum_payment_cents` is "held back" and owes $0 this period. (From
   Phase 2 the ledger carries it to the next period.)

"Over budget" never changes a number. It makes `royalty check` fail, so the
consumer removes the dependency, agrees a price, or raises the budget,
before the change merges. Scaling shares space in family-name order, so the
result is the same every time.

### Shares between payees

Each payee gets `amount * share / sum of shares`, rounded down; the cents
left over go to the payees in file order, one each. The report lists each
payee's share and their payment methods in the payee's order.

### Modes

The producer's `[terms] mode` is what they ask for; the consumer's
`[consumer.policy] modes` is what they will do. The scan uses the
producer's mode if the consumer allows it, and `honor` otherwise, and says
so. Every mode gives the same amount.

## What `royalty check` reports

`royalty check` reads one `royalty.toml` and prints each finding. An error
makes it exit 1. A warning doesn't, unless `--strict` is given. Each finding
names its code and the requirement it protects.

| Code | Level | Finding |
| --- | --- | --- |
| RTOML-001 | error | The file isn't valid TOML, a field has the wrong type, or a word field outside `[[tier]]` and `[consumer]` has a value that isn't one of its choices (such as `binding = "optional"`). |
| RTOML-002 | error | `schema` is a number this reader doesn't know. |
| RTOML-003 | warning | A field or table this reader doesn't know (often a typo). The rest of the file is still read. |
| RTOML-004 | error | `[project]`, or its `name` or `description`, is missing or empty. |
| RTOML-005 | error | `binding` is `agreement` or `license` and `[license] terms` is missing. |
| RTOML-006 | error | A `[[payee]]` has no `[[payee.method]]`, or a method misses the field its `kind` needs. A `kind` this reader doesn't know is a warning, and is skipped when paying. |
| RTOML-007 | error | A value looks like a secret: a token or key prefix (such as `ghp_`, `github_pat_`, `sk_live_`, `xoxb-`, `AKIA`), a private key block, or a long random string in a field that holds a name or a URL. |
| RTOML-008 | error | A tier is unusable: no `id`, a duplicate `id`, an unknown condition or unit, an `environment` other than `production`, a `custom:` unit with no `describe`, a `revenue` price with no `basis_points`, any other price with no `cents`, or `per` of 0. |
| RTOML-009 | error | A money or count field is negative, or `[terms] currency` isn't `USD`. |
| RTOML-010 | warning | `[terms]` or `[[tier]]` without `[[payee]]`: nobody could be paid. |
| RTOML-011 | warning | A tier after one with an empty `when`, which can never be reached. |
| RTOML-012 | error | A `[consumer]` field is invalid: an unknown word (such as a `stage`, `entity`, `class` or `on_exceed`), a `[[consumer.deployment]]` with no `manifest`, or a `[[consumer.package]]` whose `action` misses its values. |
| RTOML-013 | warning | `binding = "license"`: the code is source-available, not open source. |
| RTOML-014 | error | In a consumer's Cargo project, the scan breaks the consumer's budget or policy: an `agreement` or `license` amount over `per_producer_period_cents`, a total over the budget that can't be scaled (`on_exceed = "fail"`, or amounts that are never scaled), or a dependency whose `binding` isn't in `[consumer.policy] bindings`. The message names the dependency and the limit. |

A file with only `[project]` (every file in this monorepo today) has no
findings. When the file has a `[consumer]` table and sits beside a
`Cargo.toml`, `royalty check` also runs the scan for the month (`--period`,
the current month by default) and reports RTOML-014 when the result is
"over budget" or a dependency's `binding` isn't in `[consumer.policy]
bindings`. Run in the consumer's CI, that stops a pull request that would
break the budget before it merges.

## Transitive dependencies

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

## How the existing files map

About twenty projects in this monorepo have a file like this, as
[configuration.md](../../standards/docs/configuration.md) requires:

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

## Worked examples

### 1. A free hobby library

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

### 2. A framework: free to build on, paid in production

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

### 3. Honor code only

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

### 4. Per-seat pricing for organizations, with caps

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

### 5. A consumer that caps its own spend

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

### What Acme owes in one month

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
