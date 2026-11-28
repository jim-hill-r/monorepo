# Project reviews

Once a week a scheduled agent task, "Weekly deep project review", runs the
[`deep-project-review`](../../.agents/skills/deep-project-review/SKILL.md)
skill. Each run picks one project, studies it in depth (purpose and long-term
goals, progress and what is holding it up, features, user experience,
architecture, where its code lives, its dependencies and vendors, security, performance, quality, operations, and a re-review of
its tech selection against the [tech radar](../../standards/docs/tech-radar.md))
and writes a slide deck here as a pull request for Jim's review and feedback.

Each review lives in its own folder, `<YYYY-MM-DD>-<project>/`, with the deck
in `deck.md`, its screenshots beside it, and the scripts of its demo videos
in `demos/`.

## Reading a deck

- **On GitHub**: open `deck.md`. It renders as one long Markdown page, with
  each slide separated by a rule.
- **As slides**: the `Review decks` workflow builds every deck to HTML and PDF
  on each pull request that touches this folder (and on demand with
  `workflow_dispatch`). Download them from the run's `review-decks`
  artifact. The HTML version is a full slideshow (arrow keys to move, `f`
  for full screen), and plays the demo videos; turn their captions on and
  off with the player's CC button. The PDF shows each video's first frame.
- **Feedback**: leave review comments on the deck's pull request, on the
  lines of the slide you are commenting on.

## Rotation

The skill picks the project that has gone longest without a review (never
reviewed first), so every project gets its turn. This table is the record it
reads. One row per review, newest first.

| Date | Project | Deck | PR |
| --- | --- | --- | --- |
| 2026-11-27 | evals | [deck](2026-11-27-evals/deck.md) | [#815](https://github.com/jimhillr/monorepo/pull/815) |
| 2026-11-25 | cookbook | [deck](2026-11-25-cookbook/deck.md) | [#585](https://github.com/jimhillr/monorepo/pull/585) |

## Deck format

Decks are [Marp](https://marp.app) Markdown: plain Markdown with `---`
between slides, so they diff and review like any other file, read fine on
GitHub without building, and build to HTML and PDF with the open-source Marp
CLI on GitHub Actions at no cost. (Slidev was the other option; it needs a
Node project per deck and doesn't read as well as plain Markdown on GitHub.)

Start every deck with this front matter:

```markdown
---
marp: true
theme: default
paginate: true
size: 16:9
title: <project> deep review, <YYYY-MM-DD>
style: |
  section { font-size: 22px; }
  table { font-size: 18px; }
  section.lead h1 { font-size: 52px; }
---

<!-- _class: lead -->

# <project>
## Deep review, <YYYY-MM-DD>
```

- Every deck, whichever skill or person writes it, has a **Where the code
  lives** slide (every folder, workflow, script, skill, doc and data branch
  that holds part of the project) and a **Dependencies and vendors** slide
  (each direct dependency with its version and use; each vendor with its
  plan, cost, billing check and whether it is approved), after the
  architecture slides (Jim, 2026-11-28).
- One idea per slide; split long tables across slides.
- Pane reads the scorecard: keep it a `| Area | Rating | Why |` table under
  a `# Scorecard` heading (a continuation slide's heading must also say
  Scorecard), rated exactly Good, Fair or Poor.
- Use `<!-- _class: lead -->` for section divider slides.
- Images: `![w:900](screenshot-name.png)`, stored next to `deck.md`.
- Diagrams: text in a fenced code block or a table (Marp doesn't render
  Mermaid).
- Speaker notes, for detail that doesn't fit on the slide, go in an HTML
  comment at the end of the slide.

## Demo videos

Each deck shows the project's most important features working in the live
app, as short videos with a voiceover and closed captions
([decision 0025](../../standards/docs/decisions/0025-review-deck-demo-videos.md)).
A demo is a script in `<folder>/demos/<name>.json`: the lines to say and the
clicks to make while saying them. The
[demo recorder](demo-recorder/README.md) turns it into an MP4, a WebVTT
caption file and a poster image, with Playwright, a local Kokoro voice and
ffmpeg, at no cost. The `Review decks` workflow records the demos of the
decks a pull request changes; the videos are build output like the HTML and
PDF, and are not committed. The recorder's README has the script format and
the `<video>` tag for the slide.
