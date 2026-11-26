# Contributing to standards

## Getting Started

### Install Toolchain

Nothing to install beyond mise and bash. The scripts also use `git` and `jq`.

### Build

There is nothing to build.

### Test

```bash
cd standards
mise run ci        # checks the scripts parse and runs the repo audit
mise run //:audit  # the audit alone, from anywhere in the repo
```

When you change a task file in `tasks/` or a script in `scripts/`, the pull
request's CI runs every project that uses it.
