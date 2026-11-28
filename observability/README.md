# observability

Monitoring, alerting, logging and paging for everything the repo runs in
production, in one place (Jim, 2026-11-28: "make observability a first
class citizen in our dev platform"). The design is
[decision 0023](../standards/docs/decisions/0023-observability.md).

| Need | How |
| --- | --- |
| Logging | Cloudflare Workers Logs and Observability, 7 days, structured JSON from a shared `tracing` setup; browser errors through [`telemetry`](../telemetry/README.md) |
| Monitoring | The watchdog: a Rust Worker on one Cron Trigger that checks every production site and Worker every 5 minutes and watches error rates after each deploy |
| Alerting | Rules as code in this project, high and low urgency, one GitHub `incident` issue per outage, opened and closed by the watchdog |
| Paging | GitHub Mobile: a high-urgency incident assigns and mentions Jim, so it pushes to his phone |

Status: planned. The work is in [issues/](issues/), epic "Observability",
phases 1 to 4. Until phase 1 lands this is a docs-only project.

Changes here need the `observability-reviewer` (decision 0023, extending
the review roles of decision 0019), and the project is in the
`software-development-platform` tier.

Cost: $0 a month. No vendor besides Cloudflare and GitHub.
