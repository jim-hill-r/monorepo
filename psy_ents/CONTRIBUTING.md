# Contributing to psy_ents

## Getting Started

### Install Toolchain

The stories are Markdown files with `marp: true` front matter, using the
`uncover` theme, so there is nothing to compile. Install the project's
tasks with mise (see the root AGENTS.md "Dev commands"):

```bash
cd psy_ents
mise install
```

To preview a story you also need [Marp](https://marp.app/): either the
[Marp for VS Code](https://marketplace.visualstudio.com/items?itemName=marp-team.marp-vscode)
extension, or Node.js for the Marp CLI.

### Build

Preview a deck in the browser with the Marp CLI:

```bash
npx @marp-team/marp-cli "The Psy Ents.md" --preview
```

In VS Code, open the deck and use the Marp preview instead.

### Test

```bash
mise run ci
```

`mise run ci` is the docs-only task from `standards/tasks/docs.toml`. It
checks nothing yet (see
[docs-only-ci-checks-nothing](../standards/docs/issues/docs-only-ci-checks-nothing.md)), so
read the deck in the preview before opening a pull request.

## Writing a story

- Add one deck per story, named after the story, such as
  `The Psy Ents Go Climbing.md`, and list it in the README.
- Start it with the same front matter as the other decks:

  ```markdown
  ---
  marp: true
  theme: uncover
  paginate: true
  ---
  ```

- Separate pages with `---`, and follow the rhyme and cadence notes under
  "Writing notes" in the [README](README.md).
