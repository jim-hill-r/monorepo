# Contributing to pane_snapshot

## Getting Started

### Install Toolchain

Install the toolchain with mise (see the root AGENTS.md "Dev commands"):

```bash
cd pane/snapshot
mise install
```

### Build

```bash
cargo build
```

### Test

```bash
cargo test
```

To run all CI checks (formatting, linting, build, and tests):

```bash
mise run ci
```

### Fixtures

`tests/fixtures/repo` is a tiny fake monorepo. Its files end in `.fixture`
and every `TODO`/`FIX` in it is tagged `agent-ignore`, so agents and repo
tools looking for real `ISSUES.md` and `mise.toml` files don't pick them up.
The integration tests in `tests/build.rs` copy it into a temporary directory
under the real names. Add a parser test against a fixture file for every new
source the snapshot reads.
