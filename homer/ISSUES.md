# Priority Issues

- TODO (agent-generated): In `evaluate_property` (`src/main.rs`), make `max_bedrooms` fail when the bedroom count is unknown, like every other criterion does. Today the `None` arm silently passes. Report `Bedrooms: unknown (max is N)` and add a unit test.
- TODO (agent-generated): Make `ZillowClient` testable. Let the base URL be injected (for example `ZillowClient::with_base_url`) instead of hardcoding `https://zillow-com1.p.rapidapi.com/property` in `property_details`, then add tests against a local mock server (for example the `mockito` dev-dependency) for a successful response, a non-2xx status, and a body that is not JSON.
- TODO (agent-generated): Harden numeric parsing in `parse_property_details`. `price`, `bedrooms`, `livingArea` and `yearBuilt` are read with `as_u64`, so a value sent as a float (for example `1600.0`) is silently dropped, and the `as u32` casts truncate silently on overflow. Accept whole-number floats, use `u32::try_from` instead of `as`, and add tests for both cases.
- TODO (agent-generated): Remove the field-by-field copy of `PropertyDetails` at the end of `evaluate_property`. Derive `Clone` on `PropertyDetails` (or have `evaluate_property` take ownership of the details) so new fields don't have to be added in two places.
- TODO (agent-generated): Make `README.md` match what homer does. It says homer will "score how well a property matches", but `evaluate_property` only returns pass/fail plus unmet criteria. Either reword the README, or add a match score (criteria met out of criteria set) to `PropertyEvaluation`, print it in `format_property`, and test it.

# Backlog

# Priority Projects

# On Hold Projects
