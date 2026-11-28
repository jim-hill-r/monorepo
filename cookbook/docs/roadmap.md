# Cookbook roadmap

Written 2026-11-28 from the
[2026-11-25 deep review](../../docs/reviews/2026-11-25-cookbook/deck.md),
which Jim reviewed and agreed with "for the most part", and his two points
of emphasis (Jim, 2026-11-28):

> 1) The content is the product. This project is intended to be published
> as a book someday so the point of this project is for Jim and his wife to
> test drive the content and curate it over time, so we definitely need to
> get tooling built for that. We should likely prioritize a management app
> where Jim and his wife and curate the content. This is the main purpose
> of the auth. Only they should be able to auth. We can probably remove the
> auth in the main app once the management app is there.
>
> 2) The earthy theme is still pretty harsh. [...] This is our brand and
> should be a strength of the app.

The work is tracked as issue files in [`../issues/`](../issues/), grouped
into the epics below; the epic notes are in
[`../issues/README.md`](../issues/README.md).

The mission and vision are in [mission.md](mission.md).

## Where it stands (2026-11-28)

Most of the review's quick fixes have landed since 2026-11-25:

| Review finding | State |
| --- | --- |
| Week 53 broken, plans pinned to 2024 | Fixed: plans are dated Monday to Sunday of the current ISO week |
| Shopping list drops quantities | Fixed: `core::shopping` sums per unit and lists staples apart |
| Intro shown as a raw paragraph | Fixed: rendered as Markdown |
| No `_headers`, framing allowed | Fixed: `web/public/_headers` |
| No release size profile | Fixed: `[profile.release]` in `Cargo.toml` |
| No previous and next day links | Fixed |
| Today's recipe disagreed with the plan | Fixed (#919) |
| User guide | Added (#917) |
| Sign-in code left in the URL | Fixed in `auth_sdk` |
| **All 365 recipes are generated placeholders** | Open: the content epic below |
| **Sign-in with no purpose** (`todo!()` in `auth_sdk/src/web.rs`) | Open: given a purpose by the management app, then removed from the public site |
| Weekly themes and staples (COOK-008) | Open |
| Rearranging plans (COOK-009) | Open |
| 2,100-line `web/src/main.rs` | Open: [`split-web-src-main-rs-1.md`](../issues/split-web-src-main-rs-1.md), being worked in its own thread |
| 404 page prints a debug dump; emoji-only labels | Open, backlog |
| Migration binaries in `recipe_gen`, deprecated `Recipe.id` and `Plan.recipe_days` | Open, backlog |

## Product roadmap

### Now

Curation foundations (epic "Content curation").

Make the content model fit a book before building the tools that edit it.

1. **A recipe format for the book** ([issue](../issues/recipe-book-format.md)): a written spec,
   `docs/content-format.md`, and new fields on every recipe: a `Status`
   (`placeholder`, `draft`, `tested`, `book-ready`), `Yield`, `Source`
   (who it came from, for attribution in the book) and `Equipment`. Every
   generated recipe is marked `placeholder`, so progress toward the book is
   a number anyone can see.
2. **Weekly themes and staples** (COOK-008, [issue](../issues/plan-themes-and-staples.md)): each plan names its theme and
   lists the staples the week assumes, shown on the plan page.
3. **Show progress on the public site** ([issue](../issues/kitchen-tested-mark.md)): a small "kitchen-tested" mark on
   recipes the curators have tested, so readers know which ones to trust.

### Next

The management app (epic "Management app"). Its first three phases are
priority issues, so agents start them alongside the content epic:
[who can sign in](../issues/manage-auth0-curator-role.md), [the app
shell](../issues/manage-app-shell.md) and [the server
check](../issues/manage-api-token-check.md).

A separate Dioxus app at `cookbook/manage`, deployed as the Pages project
`jimhillr-cookbook-manage`, named and built the way
[decision 0018](../../standards/docs/decisions/0018-feature-flags.md)
describes a project's management app (always "management app", never
"console"). In 0018's terms, the two curators are the cookbook's **app
managers**: they hold the Auth0 role `cookbook-manager` with the permission
`manage:cookbook`, which Jim assigns in the Auth0 dashboard, so no name or
email is ever in the public repo.

In the order it is built:

1. **Who can sign in**: the Auth0 API, role and permission as IaC in the
   `infra` auth0 stack, with multi-factor sign-in for anyone holding the
   role (a credential change: Jim's second approval, decision 0010), and a
   human task with a runbook for Jim to give the role to himself and his
   wife.
2. **The app shell**: sign-in through `auth_sdk`, a "you can't use this
   app" page for anyone without the permission, and a read-only list of
   recipes and plans with each recipe's status and counts toward the book.
3. **The server check**: a Pages Function on the management app's own
   origin that verifies the Auth0 access token and the `manage:cookbook`
   permission on every call. This, not the app's pages, is what keeps
   everyone else out.
4. **Saving as content PRs**: the management app's server saves a
   curator's change as a commit to a content PR that touches only
   `cookbook/content/`, through a GitHub App token minted by Infra Mint
   (a credential change). The reviewer account's written rules gain a case
   for those PRs, so a curator's save reaches the public site on the next
   deploy without anyone using git.
5. **The cooking log**: "we cooked this" with the date, who cooked (a display name, never an email), a
   rating, notes and what they changed, kept in the recipe's own Markdown
   file. It is the heart of test-driving the content, and the notes are
   public like the recipes.
6. **The recipe editor**: change a recipe's title, ingredients, steps,
   status and notes in a form with a live preview, with no git.
7. **The plan editor** (COOK-009): move a recipe to another day or week,
   swap in a different recipe, and set a week's theme and staples.
8. **Remove sign-in from the public site**: once the curators are using the
   management app, the public site drops its Login button, `auth_sdk`, the
   Auth0 call on every page load and its Auth0 callback URLs. Open PR #783
   (the public site's sign-in redirect) becomes unneeded then.

All content, cooking notes included, stays Markdown in this repo, the
book's source (Jim, 2026-11-28: "Let's do A, but I don't think we need
private notes"). There is no database.

### Later

The book (epic "Book").

1. **Book progress**: a page in the management app showing recipes by
   status, which weeks are complete, and what to cook next to finish a
   week.
2. **Manuscript export**: a `chores` command (Rust) that builds one
   Markdown manuscript from the book-ready content, ordered by week, with
   the intro, themes, staples and sources, ready for a publisher or a
   print-on-demand tool. The tool for EPUB or PDF is chosen then, free and
   approved vendors first.

Reader features the review listed but that wait until the content is real:
search and tags, favourites (without accounts, kept in the browser), tick
boxes and a printable view for the shopping list.

### Brand: the earthy theme

The earthy theme lives in the shared `design_system` and is being
redesigned in its own thread ("Make cookbook's earthy theme pleasant"),
with theme design references added to
[athenaeum](../../athenaeum/). This roadmap only depends on it: the
management app uses the same theme, so it picks up the redesign for free.

## Technical roadmap

### Now

- **Split `web/src/main.rs`** into modules and move date and plan logic
  into `core` ([issue](../issues/split-web-src-main-rs-1.md), in progress in its
  own thread). The public-site changes above that touch `web/src/main.rs`
  (themes on the plan page, the tested mark, removing sign-in) wait for it
  to merge, to avoid conflicts.
- **One content library for both apps.** `core` and `data_md` already
  hold the model and the Markdown format; the management app uses them as
  they are, so the public site and the management app can never disagree
  on what a recipe is. `data_md` gains a writer that round-trips every new
  field ([issue](../issues/recipe-book-format.md)).

### Next

- **Token checks on Cloudflare.** The management app's Pages Function
  verifies Auth0 tokens in Rust (`workers-rs`). It is the first such
  server check in the repo; decision 0018's flags Worker can reuse it.
- **Evals**: each phase adds its requirements to
  [`requirements.md`](requirements.md) and their evals to
  [`../evals.toml`](../evals.toml) as it ships. The management app gets
  requirements for "only curators can sign in or call the API", "a
  curator can log a cook", "a curator can edit a recipe without git" and
  "the public site shows the curators' edits after the next deploy".

### Later

- **Retire what is done**: the four migration binaries in `recipe_gen`,
  and the deprecated `Recipe.id` (`day-N`) and `Plan.recipe_days`, with an
  index by UUID instead of a scan.
- **Small fixes** left from the review: the 404 page's debug dump and the
  emoji-only recipe labels.

## Cost

$0 a month, all on free tiers and approved vendors (Cloudflare, GitHub,
Auth0 as an approved exception):

- One more Pages project, `jimhillr-cookbook-manage` (budget 25).
- Pages Functions requests from two curators, far under the free 100,000
  a day.
- Two Auth0 monthly active users once the public site drops sign-in, down
  from today's estimate of 5.
- No database: content and cooking notes are Markdown in the repo. One
  GitHub App token for content PRs, minted and rotated by Infra Mint.

Claude build effort is about 3 PRs for the content epic, about 8 for the
management app and about 2 for the book, spread over several weeks of the
$25 a month budget.

## Decisions

1. **Where curated content lives** (Jim, 2026-11-28, on a decision card:
   "Let's do A, but I don't think we need private notes"). Recipes, plans
   and the cooking log are Markdown in this repo, and the management app
   saves changes as content PRs. Not chosen: everything in D1 copied to
   the repo nightly (two copies to keep in step), and a private content
   repo (a read token for the public build). All three were $0.

## Open questions

1. **Weekly themes**: what a theme is (a cuisine, a season, an
   ingredient) and whether staples are per week or one pantry list (the
   review's question 3). The content epic starts with a free-text theme
   and per-week staples, which work for any answer.
