---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 45
added: 2026-11-26
---

Decision 0010, items 2, 5 and 7: add IAC-005 to `infra/src/guard.rs`, next to IAC-001 to IAC-004, once PR #701 merges. It doesn't fail a plan; it reports, per stack, whether the plan creates or updates a credential-class resource (the table in decision 0010: Auth0 client creation, `auth0_client_credentials`, `auth0_client_grant`, `auth0_resource_server*`, connections and roles; Cloudflare API tokens and `cloudflare_zero_trust_access_*`; GitHub secrets, variables, environments, rulesets, branch protection, deploy keys, collaborators and repo settings), with a machine-readable output the workflows use to pick the `infra` or `infra-credentials` job and a readable summary (resource, before and after) for the job summary. It hard-fails any plan that touches the `infra-credentials` environment, its reviewers or secrets, or a ruleset bypass list. Editing an Auth0 app's URLs is not a credential change. Fixtures go in `infra/tests/fixtures/plans/`, one per class plus a URL-only update that must pass as non-credential. Document it in the guard table in `infra/README.md`. The workflow split is in `iac-phase-1-workflows.md`.
