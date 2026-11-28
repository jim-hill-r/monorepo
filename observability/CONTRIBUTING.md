# Contributing to observability

## Getting Started

This is a docs-only project until phase 1 adds the watchdog Worker, so
there is nothing to build yet:

```bash
cd observability
mise run ci   # docs-only project, nothing to check
```

Pick up the next phase from [issues/](issues/) with the `start-a-new-task`
skill. Read [decision 0023](../standards/docs/decisions/0023-observability.md)
first: alert text goes into public GitHub issues, so it must never carry
user data, request bodies, internal hostnames or secrets.

Every change here, and every change to a check, an alert rule or who is
paged, needs the `observability-reviewer`.
