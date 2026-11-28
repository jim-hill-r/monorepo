---
name: deep-project-review
description: Pick the next project in this monorepo's review rotation, study it in depth (purpose, goals, features, user experience, architecture, security, performance, testing, operations, tech selection against the tech radar) and deliver a detailed Marp slide deck in docs/reviews/ as a pull request for Jim's review. Use for the weekly deep project review or when asked for a deep review of one project.
---

# Deep project review

Produce one detailed review deck for one project, so Jim can see where the
project stands and decide what to do next. Read `AGENTS.md` and
[`docs/reviews/README.md`](../../../docs/reviews/README.md) first. The README
holds the rotation record and the deck format.

This skill writes a review, not fixes. Don't change the project's code. The
only files a run changes are its deck folder, the rotation table in
`docs/reviews/README.md`, new files in the project's `issues/` (step 6), and the tech
radar (step 6).

## 1. Pick the project

Unless you were asked to review a specific project:

1. Candidates are the top-level entries (no `/`) in `config_roots` in the root
   `mise.toml`, except the repo tooling folders `standards` and `jimhillr`.
2. Drop a project that an accepted plan or decision in `standards/docs/` says
   is being removed, and any project with an open review pull request (a PR whose
   branch adds a folder under `docs/reviews/`).
3. Pick the project that has gone longest without a review, using the
   rotation table in `docs/reviews/README.md`. A project with no row has never
   been reviewed and goes first. Break ties in this order: projects under
   `# Priority Projects` in the root `issues/README.md` (in listed order), then the
   rest alphabetically, then projects under `# On Hold Projects` last.

Record the choice and the reason in the deck's title slide and in the
rotation table.

## 2. Gather evidence

Take the time to read the whole project. Every claim in the deck must point
at evidence: a `path:line`, a command and its output, a screenshot, or a
commit or PR link. Say plainly when something is a guess.

- **Purpose and goals.** The project's `README.md`, its `issues/` folder, any
  `docs/`, plans and decisions (its own and in `standards/docs/`), and the
  root `issues/`. Find the long-term goal it was built for. If no goal is
  written down anywhere, say so and infer one from the code, marked as a
  guess, and ask Jim to confirm it.
- **History.** `git log --stat` for the project (run `git fetch --unshallow`
  first if the clone is shallow): when it started, when it was last active,
  what has been done recently, what stalled and when.
- **Features.** Every route, screen, command, API endpoint or game mode,
  found from the code, not only the README.
- **User experience.** Install the tooling and build and run the project as a
  user would (see **Dev commands** in `AGENTS.md`). For a web app, start the
  dev server and walk the main journeys with Playwright, saving 3 to 8
  screenshots at a 1280x800 viewport. For a CLI, record `--help` and one run
  of each main command. For a game, run it under `xvfb-run` and screenshot it
  if it starts. Note first-run experience, empty states, error messages,
  loading, accessibility (keyboard, contrast, labels), mobile layout, and
  anything confusing or broken. If it doesn't build or run, that is a
  finding: record the exact error.
- **Feature demos.** For a project with a live web app, pick the two or
  three journeys that matter most to its users (the goals and features
  above say which) and write a narrated demo script for each in
  `docs/reviews/<folder>/demos/<name>.json`, about a minute each, in the
  format in [`docs/reviews/demo-recorder/README.md`](../../../docs/reviews/demo-recorder/README.md).
  Reuse the selectors from the project's own Playwright tests. The
  `Review decks` run on the PR records them against production with a
  voice and captions; if a demo fails there, fix its script. Skip demos for
  a project with nothing live to show (a library or CLI), and say so on the
  demo slide.
- **Architecture.** Crates or packages, modules, data model, data flow,
  external services and deploy targets.
- **Where the code lives.** Every place in the repo that holds part of the
  project, not only its folder: its crates and sub-projects, the workflows
  in `.github/workflows/` and scripts in `standards/scripts/` that build,
  deploy or run it, skills in `.agents/skills/`, its standards, plans and
  decisions in `standards/docs/`, files it owns in other projects (such as
  each project's `evals.toml` for evals), long-lived data branches, and
  other projects that call it or that it calls. Find them with `grep -rl`
  for its name, crate names and paths, and list each with its path.
- **Dependencies and vendors.** Every direct dependency from the manifests
  (`cargo tree --depth 1 -e normal,build` per crate, `package.json`,
  `mise.toml` tools, pinned GitHub Actions), with its version and what the
  project uses it for, and every vendor or hosted service it reaches (from
  `docs/cost.toml`, workflows, config and the code), with its plan, monthly
  cost, how its billing is checked (the tech radar) and whether it is on the
  approved vendor list in `AGENTS.md`.
- **Security.** A STRIDE threat model of the assets, actors and trust
  boundaries; authentication and authorization; input handling; secrets and
  configuration; CORS and headers for web apps; and known-vulnerable
  dependencies (`cargo audit` and `npm audit` where they apply; install
  `cargo-audit` with `cargo install --locked cargo-audit` if needed). Rate
  each finding Critical, High, Medium or Low with a one-line reason.
- **Performance.** Measure what you can: build time, binary or wasm bundle
  size, test run time, startup time, page weight. Then read the hot paths
  for needless allocation, blocking work in async code, N+1 lookups or
  unbounded growth.
- **Quality and testing.** Run the project's CI task and record the result.
  Compare against `standards/docs/` (rust, testing, documentation, naming,
  typescript). Note what is tested, what isn't, and lints that are off.
- **Operations and cost.** How it is built, deployed and observed, what
  vendors it uses, what it costs, and whether that matches the vendor rule in
  `AGENTS.md`.
- **Tech selection against the tech radar.** Read
  [`standards/docs/tech-radar.md`](../../../standards/docs/tech-radar.md) and
  `standards/radar/radar.json`, then re-review every technology and vendor
  the project uses (from its manifests, lockfiles, `mise.toml`, workflows and
  config), and every decision record that chose one for it. For each, record:
  - its radar ring, or "not on the radar";
  - whether the project uses it the way the radar and its decision say
    (for example a Hold blip in new code, a Trial blip outside its
    conditions, a vendor without sign-off, or a version behind what the rest
    of the repo uses);
  - whether the decision still holds today. Check what changed since it was
    made: new releases or deprecations, free-tier limits or pricing, a
    better-fitting approved vendor, how well Claude supports it (a Claude
    Code plugin or MCP server we could enable read-only, per the radar's
    selection criteria), and lessons from this review. Look these up at the
    source (release notes, pricing and limits pages) and cite them with the
    date read;
  - whether the project should be an exemplar for a blip that has none, or
    has stopped being a good one.

## 3. Judge the state against the goals

This is the heart of the deck. For each long-term goal, say how far along the
project is (not started, early, partly there, mostly there, done) with
evidence, and what is holding it up. Blockers can be technical (a broken
build, a missing piece of architecture), process (no tests, no deploy), a
decision only Jim can make, or plain lack of attention. Rank them by how much
they slow the project down.

## 4. Write the deck

Create `docs/reviews/<YYYY-MM-DD>-<project>/deck.md` (the run's date in
America/New_York; replace `/` in a project path with `-`). Put screenshots
and other images in the same folder as PNG, each under 300 KB, and reference
them with relative paths. Follow the format in `docs/reviews/README.md`. Use
these sections, one or more slides each, in this order:

1. **Title**: project, date, reviewer, why this project was picked this week.
2. **Executive summary**: what the project is in one sentence, its state
   against its goals, the three biggest blockers, a scorecard (one row each
   for goals, UX, architecture, security, performance, quality, operations
   and tech selection, rated Good, Fair or Poor with a few words of why),
   and the top five recommendations. Pane reads the scorecard
   (`pane_snapshot`'s `scorecard` parser) and shows it on its Reviews page,
   so keep it a `| Area | Rating | Why |` table under a heading that says
   Scorecard, rated exactly Good, Fair or Poor.
3. **Purpose and long-term goals**: who it is for, what problem it solves,
   the goals and where they are written down.
4. **Progress against goals and what is holding it up** (step 3).
5. **Features**: a table of every feature with its status (works, partial,
   broken, stub).
6. **Feature demos**: one slide per demo video, each with a heading that
   names the journey and the `<video>` tag from the recorder's README.
7. **User experience**: journeys walked, screenshots, what works, what hurts.
8. **Architecture**: components, data flow, deployment. Draw diagrams as
   text in a fenced code block or as a table; Marp doesn't render Mermaid.
9. **Where the code lives**: a `| What | Where |` table of every place in
   the repo that holds part of the project (step 2), and one line on
   whether it is one project or spread across the repo, and why.
10. **Dependencies and vendors**: a `| Dependency | Version | Used for |`
    table of every direct dependency (split across slides when long), then
    a `| Vendor | Plan | Cost a month | Billing check | Approved |` table of
    every vendor and hosted service. Say "None" when there are none.
11. **Security**: threat model, then findings by severity.
12. **Performance**: measurements, then findings.
13. **Code quality and testing**: CI result, coverage of tests, gaps against
    the standards.
14. **Operations and cost**: build, deploy, observability, cost.
15. **Tech selection against the radar**: a table of every technology and
    vendor the project uses, with its ring, whether its use fits the radar,
    whether the decision behind it still holds (and what changed, with
    sources), and the recommended radar change if any (move ring, add or
    drop a blip, add or drop this project as an exemplar).
16. **Risks and technical debt**.
17. **Recommendations**: a prioritized table with impact, effort (S, M, L)
    and the area each one moves forward.
18. **Questions for Jim**: decisions or facts only Jim can give.
19. **Appendix**: method, commands run with their results, and evidence
    links.

Aim for 25 to 45 slides. Keep each slide to what fits on one screen: split a
long table across slides rather than shrinking it. Write in plain sentences
for a reader who knows the repo but hasn't opened this project in months.

Build the deck locally before pushing, and fix anything that overflows or
fails to render:

```sh
npx --yes @marp-team/marp-cli@4.5.1 --no-stdin --html --allow-local-files \
  docs/reviews/<folder>/deck.md -o /tmp/deck.html
```

## 5. Record the rotation

Add a row for this review to the rotation table in `docs/reviews/README.md`
(date, project, deck link, PR link once the PR exists). Keep one row per
review, newest first. Pane's Reviews page reads this table
(`pane_snapshot`'s `reviews` parser), so keep its `| Date | Project | Deck |
PR |` columns, ISO dates, and a `[deck](<folder>/deck.md)` link.

## 6. File the follow-ups

Add each recommendation as its own issue file in the project's `issues/`
folder (`priority: priority`, tagged `[agent-generated]`, in the shape the
`code-review` skill uses), each linking back to the deck. Merging the review
PR accepts them; Jim can delete any he disagrees with in review. Don't add an
issue that duplicates an existing one.

Apply the radar changes from section 15 in the same PR: edit
`standards/radar/radar.json` (ring, description, exemplars, sources, and
`isNew` set to `"TRUE"` for a blip that moved), then run
`bash standards/scripts/radar.sh render`. List each change in the PR
description. A change that approves a new vendor, or moves a vendor into
Adopt or Trial, needs Jim's sign-off: label the PR `needs-human`. A decision
that no longer holds gets an issue file in `standards/issues/` to write a
superseding decision record, not a rewrite of the old one.

## 7. Open the pull request

Deliver with the `open-pull-request` skill, with these differences:

- A review PR only adds docs, so it doesn't wait for other agent PRs to
  close, and there is no project CI to run beyond `standards` when the radar
  changed. The `Review decks` check records the deck's demo videos and
  builds the deck to HTML and PDF, and attaches them all to the run.
- Title: `<project>: deep review <YYYY-MM-DD>`.
- The description has the sections "Pull request descriptions" in
  `standards/docs/pull-requests.md` requires, with What changed and Why
  giving the state against goals, top blockers and top recommendations in
  at most 50 words each, links to the deck file on the branch,
  says where to download the HTML and PDF (the `Review decks` run's
  artifacts), and asks Jim for feedback as PR review comments on the deck.
- Request a review from `jimhillr`, and mark the PR ready for review once the
  `Review decks` check is green.

When Jim leaves feedback, convert the PR to draft before changing anything,
answer each comment in the deck (or reply on the PR saying why not), and mark
it ready for review again once the `Review decks` check is green.
