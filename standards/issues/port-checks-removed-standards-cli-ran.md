---
kind: todo
priority: priority
added: 2026-11-26
---

Port the checks the removed `standards` CLI ran into `standards/scripts/audit.sh`: NAM-002 (directory matches package name), NAM-004 (`poc_` prefix), DOC-001 to DOC-004 (README.md and CONTRIBUTING.md with a Getting Started section) and LIN-001/LIN-002 (Cargo lints). About 20 projects miss a README.md or CONTRIBUTING.md today, so add them as warnings first, or fix the projects in the same PR.
