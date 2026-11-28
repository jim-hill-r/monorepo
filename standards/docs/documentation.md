- All projects MUST include a README.md. The README.md should include the following sections:
    - {Name of the project} section which includes a short description (executive summary) of the goal of the project, including why it was named that.
- All projects MUST include a CONTRIBUTING.md. The CONTRIBUTING.md should include the following sections:
    - A `Getting Started` section which describes how to install the project's toolchain, how to build, and how to test.
- `mise run //:audit` checks both for every project (a directory with a `mise.toml`): DOC-001 (no README.md), DOC-002 (no heading with the project's package name, directory name or path, such as `bear_down/cloudflare`, or the name with spaces for `_` and `-`, optionally after `The`, such as `The Psy Ents`), DOC-003 (no CONTRIBUTING.md) and DOC-004 (no `Getting Started` heading). They are errors, so they fail the audit (`standards/scripts/project-check.sh`). Projects in the `# On Hold Projects` list in `standards/docs/issues/README.md` are exempt until they come off hold, and must pass before they do.

## User docs

Every top-level project has a user guide for the people who use what it
ships (Jim, 2026-11-27: "All projects should have user docs for supporting
our end users"; [decision 0017](decisions/0017-user-docs.md)). The README
says what a project is and CONTRIBUTING how to build it; the guide says how
to use it.

- **Where:** `<project>/docs/user/README.md`. Sub-projects are covered by
  their parent's guide. A project with several products or long topics adds
  more pages in `docs/user/` and links them from the README there.
- **Who for:** the project's end users: the people on a site or app, the
  players of a game, the developers calling a library, the people running a
  CLI. For internal tools (`chores`, `cost`, `evals`, `standards`) that is
  the agents and maintainers who run them.
- **Sections**, as `##` headings in this order, each with text under it:
  - `## Who it's for`: who uses it and what they get from it.
  - `## Getting started`: how to reach or install it and do the first
    thing, such as the site's URL and signing in, or installing a CLI.
  - `## How to ...`: the main tasks, step by step, with the heading
    starting "How to" (`## How to plan a week`). More task sections may
    follow it.
  - `## Getting help`: common problems and fixes, and how to report one
    (the in-app feedback button where there is one).
- **Style:** plain words for a non-engineer where the users aren't
  engineers, steps that match what ships today, and screenshots where they
  help. A change that alters what a user sees or does updates the guide in
  the same PR.
- **Check:** `chores user-docs`, run by `mise run //:audit`, fails on
  DOC-005 (no guide), DOC-006 (a section missing) and DOC-007 (a section
  with no text), except for projects listed in the baseline,
  `standards/user-docs.toml`. DOC-008 fails when the baseline is stale: a
  listed project passes, its issue file is gone, or it names no project.
  The baseline only shrinks: the PR that adds a project's guide removes
  its line and deletes its `write-user-guide.md` issue. A new project ships
  with its guide.

```markdown
# Cookbook user guide

## Who it's for

Anyone cooking from the family recipes...

## Getting started

Open the site on a phone or computer...

## How to plan a week

1. ...

## Getting help

If a recipe page is blank, ... Report a problem with the feedback button.
```

## Retros

A retro looks back on a piece of work (a project phase, an incident, a
release) and records what to change. Pane reads retros to show them and to
light each project's Retros health signal, so they follow one shape.

- **Where:** repo-wide retros in `standards/docs/retros/`, a project's retros
  in `<project>/docs/retros/`.
- **Name:** one file per retro, `YYYY-MM-DD-<topic>.md`, dated the day it is
  written, such as `2026-12-01-pane-phase-0.md`.
- **Title and date:** an `# H1` title, then a `- Date: YYYY-MM-DD` line. When
  the line is missing, the date comes from the file name.
- **Sections**, in this order:
  - `## What happened`: the work and the outcome, with links to its plan,
    PRs or incident.
  - `## What went well`
  - `## What to change`
  - `## Actions`: a checkbox list, `- [ ]` for open and `- [x]` for done.
    Each open action is also an issue file in the right `docs/issues/` folder,
    and the action links to that file. Tick the action when its issue is
    done.

```markdown
# Pane phase 0

- Date: 2026-12-01

## What happened

## What went well

## What to change

## Actions

- [ ] Run the smoke tests against PR previews ([issue](issues/run-smoke-tests-against-pr-preview.md))
- [x] Add a retro convention
```

## Pull Request Standards

### UI Changes Documentation

All pull requests with UI changes MUST include visual documentation.

**What constitutes a UI change:** Any change that affects what users see or how they interact with the application, including:
- Changes to visual appearance (colors, layout, styling, fonts)
- Changes to user interface components (buttons, forms, menus, navigation)
- Changes to displayed content or text (including error messages, tooltips, help text)
- Changes to animations or transitions
- Changes to responsive behavior or mobile layouts
- Changes to accessibility attributes that affect user experience (ARIA labels, roles)

**Screenshot Requirements:**

- **Screenshots**: Always include screenshots of UI changes in the PR description
  - Take full-page screenshots showing the before and after states when possible
  - Capture screenshots that clearly demonstrate the visual changes
  - Use the playwright browser tools to take screenshots when the dev server is running

**Exceptions:** Backend-only changes, configuration changes, or changes to non-visual code do not require screenshots.