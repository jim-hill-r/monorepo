---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

Bathroom numbers aren't checked. `parse_property_details` (`../src/main.rs`) takes any JSON number for `bathrooms`, so a negative value from the API is shown and scored, unlike the whole-number fields, which go through `json_whole_number`. And `--min-bathrooms` accepts `NaN`, `inf` and negative values, so `NaN` makes every property fail the criterion. Treat a negative or non-finite API value as unknown (`None`), and reject a negative or non-finite `--min-bathrooms` with a clap `value_parser` error. Add unit tests for the parser and a trycmd transcript for the CLI error.
