# Priority Issues

- TODO (agent-generated): Harden numeric parsing in `parse_property_details`. `price`, `bedrooms`, `livingArea` and `yearBuilt` are read with `as_u64`, so a value sent as a float (for example `1600.0`) is silently dropped, and the `as u32` casts truncate silently on overflow. Accept whole-number floats, use `u32::try_from` instead of `as`, and add tests for both cases.
- TODO (agent-generated): Remove the field-by-field copy of `PropertyDetails` at the end of `evaluate_property`. Derive `Clone` on `PropertyDetails` (or have `evaluate_property` take ownership of the details) so new fields don't have to be added in two places.
- TODO (agent-generated): Make `README.md` match what homer does. It says homer will "score how well a property matches", but `evaluate_property` only returns pass/fail plus unmet criteria. Either reword the README, or add a match score (criteria met out of criteria set) to `PropertyEvaluation`, print it in `format_property`, and test it.

# Backlog

# Priority Projects

# On Hold Projects
