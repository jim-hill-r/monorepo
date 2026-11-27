---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

When the Zillow API returns an error status, `property_details` (`../src/main.rs`) puts the whole response body in the error message. A gateway or proxy error can be a long HTML page, which floods the terminal and hides the status line. Keep the first 300 characters or so (on a character boundary), collapse whitespace, and add "(truncated)" when cut. Test it with a mockito 502 that returns a long HTML body.
