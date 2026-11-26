# Priority Issues

- TODO (agent-generated): Make `README.md` match what homer does. It says homer will "score how well a property matches", but `evaluate_property` only returns pass/fail plus unmet criteria. Either reword the README, or add a match score (criteria met out of criteria set) to `PropertyEvaluation`, print it in `format_property`, and test it.

# Backlog

- TODO (agent-generated): Keep homer's RapidAPI use free, as AGENTS.md requires (Jim, 2026-11-26). Read the `x-ratelimit-requests-remaining` header RapidAPI returns, print a warning when the monthly free quota is nearly used up, and stop before making a request that would go past it. Test it against the mock server.

# Priority Projects

# On Hold Projects
