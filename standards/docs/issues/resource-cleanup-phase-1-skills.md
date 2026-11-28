---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 1: claims, guards and a read-only sweep"
order: 40
added: 2026-11-26
---

Resource cleanup phase 1 (see the [plan](../plans/2026-11-resource-cleanup.md)). Give the `human-issues-digest` skill a step that reads the latest sweep report and, for each new orphan, opens one draft PR adding a `[[retired]]` entry (or `[[keep]]` when it is clearly Jim's) labelled `needs-human`, lists it in the digest, and sets `deleted` on retired entries the sweep confirms are gone. Give the `open-pull-request` skill a checklist line: a PR that renames or removes a resource adds its retirement entry and lists the inbound references to remove first.
