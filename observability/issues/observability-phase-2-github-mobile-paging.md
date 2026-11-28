---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Observability / Phase 2: GitHub Mobile paging"
order: 20
added: 2026-11-28
---

Page Jim through GitHub Mobile ([decision 0023](../../standards/docs/decisions/0023-observability.md)). Add the `jimhillr-alerts` GitHub App manifest (Issues read and write on this repository only), a human task with a runbook for Jim to create and install it and to set GitHub Mobile push and his watch level to Participating and @mentions, the `incident` label in the infra `github` stack, and a workflow in `infra-credentials` that copies the App key into the watchdog's Worker secret. Then the watchdog opens an `incident` issue (key `<project>/<check>` first in the title) after two failed checks, assigning and mentioning Jim for high urgency, mentions again after an hour, and closes it on recovery. Alert text carries no user data (public repo). Add secret-rotation and settings.toml rows. Prove it with a fire drill.
