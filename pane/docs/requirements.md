# Pane requirements

What pane promises Jim: one place to review the work going on in the
monorepo. The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-26 from the Goal, The site and Phases
sections of the [review dashboard plan](plans/2026-11-review-dashboard.md)
and the existing tests. Pane has no deep review yet. PANE-001 to PANE-006
cover what is built (phases 0 and 1) and PANE-008 the Reviews page (phase 2);
PANE-007, PANE-009 and PANE-010 are the plan's later phases and have no
evals until they are built.

## PANE-001: Today shows what needs Jim

- Priority: must
- Why: Jim opens pane to learn within a minute what is waiting on him.
- Accept: the home page lists the open human issues oldest first, each with
  its age and a link to its file on GitHub, and marks what is new since
  his last visit; it still works when the browser blocks localStorage.

## PANE-002: Pane shows the repo as it is on main

- Priority: must
- Why: A dashboard that is out of date or misses a project misleads Jim.
- Accept: every deploy builds a fresh snapshot of main before the site is
  bundled; it covers every project in `config_roots`, every `issues/` file
  and every human issue, plan, decision and retro; the footer shows the
  snapshot's commit and how old it is.

## PANE-003: All open work can be found and filtered

- Priority: should
- Why: Jim needs to see the backlog across every project without opening an
  editor.
- Accept: the Work page lists every issue file and `TODO`/`FIX` comment,
  fixes first and then by priority, and filters by project, kind, priority
  and tag; each item links to its line on GitHub at the snapshot's commit.

## PANE-004: Every deliverable is a click away

- Priority: should
- Why: Pane is where Jim finds what the agents shipped.
- Accept: the Deliverables page lists live sites, releases with their
  download links, plans, decisions and retros, newest first; a document
  opens rendered in the page, also from a direct link.

## PANE-005: Each project has one page with everything about it

- Priority: should
- Why: Reviewing a project should not mean visiting five places.
- Accept: a project page, opened from the list or by its path, shows what
  the project is, its sub-projects, documents, deliverables and its open
  work, fixes first, including its sub-projects' work.

## PANE-006: Pane is easy and safe to use on a phone

- Priority: should
- Why: Jim often reviews from his phone.
- Accept: below 720 px the layout is one column under the nav; the nav
  opens every page; an unknown path shows a not-found page with a way
  home; Markdown from the repo is shown as text, never run, and links to
  `javascript:` URLs are blocked.

## PANE-007: Every project and system has a traffic light

- Priority: should
- Why: Jim wants the health of the whole repo at a glance, with the
  evidence on a click.
- Accept: the Health page shows a red, amber or green light per project and
  shared system for each signal (build, deploy, UAT, review, standards,
  requirements), and a click shows the evidence behind the colour.

## PANE-008: Executive reviews are read in pane

- Priority: should
- Why: Review decks are how Jim hears about each project in depth.
- Accept: the Reviews page shows the rotation, each deck as slides in the
  page, its PDF download and a link to its pull request.

## PANE-009: Jim can add a TODO and close a human issue from pane

- Priority: should
- Why: Acting on what he reads should not need an editor or a terminal.
- Accept: the Add page and the Human issues actions open a pull request
  that adds the issue file, or deletes the human issue's file, and pane
  shows the link to it.

## PANE-010: Each project's revenue and cost is shown by month

- Priority: could
- Why: Jim wants to know whether each project pays for itself.
- Accept: a project shows its revenue and its costs per month and whether
  it covers them.
