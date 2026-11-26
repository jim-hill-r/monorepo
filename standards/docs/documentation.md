- All projects MUST include a README.md. The README.md should include the following sections:
    - {Name of the project} section which includes a short description (executive summary) of the goal of the project, including why it was named that.
- All projects MUST include a CONTRIBUTING.md. The CONTRIBUTING.md should include the following sections:
    - A `Getting Started` section which describes how to install the project's toolchain, how to build, and how to test.

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
    Each open action is also a `TODO` line in the right `ISSUES.md`, and the
    action links to that file. Tick the action when its TODO is done.

```markdown
# Pane phase 0

- Date: 2026-12-01

## What happened

## What went well

## What to change

## Actions

- [ ] Run the smoke tests against PR previews ([ISSUES.md](../../ISSUES.md))
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