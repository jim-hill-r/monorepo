---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The report's `Address:` line echoes the address the user typed (`parse_property_details` sets `address: address.to_string()`), not the property Zillow matched, so a fuzzy match to a different house looks like the right one. Read Zillow's `address` object (`streetAddress`, `city`, `state`, `zipcode`) and `zpid` into `PropertyDetails`, print the matched address (with the typed one when they differ) and the zpid, add them to `--json` (README example too), and test both the match and the mismatch case.
