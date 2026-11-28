---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Web standard / Phase 3: site check"
order: 30
added: 2026-11-28
---

Add chores site-check: an offline check of a site's build folder for the rules Lighthouse can't assert (WEB-101 to WEB-108 across pages, WEB-201 content in the HTML, WEB-202 llms.txt format, WEB-203 .md copies, WEB-205 JSON-LD parses and has required properties, WEB-206 robots.txt matches the lists, WEB-305 _headers caching, WEB-401 every site listed), against the baseline in standards/web.toml. Run it in mise run //:audit, warnings for a week first. See [web.md](../standards/docs/web.md).
