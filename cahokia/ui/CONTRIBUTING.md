# Contributing to ui

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands").
`ui` is a member of the cahokia Cargo workspace, so its tasks run from
`cahokia/`:

```bash
cd cahokia
mise install
```

### Build

To build the crate:

```bash
cargo build -p ui
```

### Test

To run its tests (server-side renders of each component, and the doc tests
in this README's examples):

```bash
cargo test -p ui
```

To run all CI checks for the cahokia workspace (formatting, linting, build,
and tests):

```bash
mise run ci
```

`ui` is in the workspace's `default-members`, so a plain `cargo build` or
`cargo test` in `cahokia/` covers it too. The web app's Playwright tests in
`web/tests/` check these components in a browser; see
[web's CONTRIBUTING](../web/CONTRIBUTING.md).

### Where code goes

- Put components and pages more than one platform shows here, and keep
  platform code (routes, entry points, platform APIs) in `web`, `desktop`
  or `mobile`.
- Keep layout and tree maths in `core`; components here only draw it.
- Style with the design system's `--ds-` tokens only, never raw colours.
- Add a render test next to each component in a `#[cfg(test)] mod tests`,
  and update the README's table and examples when a component changes.
