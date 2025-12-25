Standards for this repository are found in the standards project.

## Testing Requirements

All code changes require appropriate tests:
1. **Unit Tests**: All code changes must include unit tests
   - For Rust code: Add `#[cfg(test)]` modules with test functions
   - Tests should verify component behavior, logic, and edge cases
   - Run tests with `cargo test` in the project directory
2. **Playwright Tests**: All UI changes must include Playwright end-to-end tests
   - For Dioxus web applications: Add `.spec.ts` files in the `tests/` directory
   - Tests should verify user interactions, navigation, and visual elements
   - Run tests with `npm test` after starting the dev server (`dx serve --port 8080`)
   - See existing test files for examples of patterns and best practices

## Task Completion Requirements

Before finishing any task that involves code changes:
1. Always run `cast ci` on any projects that have been modified
2. Ensure `cast ci` passes before completing the task
3. Fix any formatting, linting, build, or test failures reported by `cast ci`
4. If changes are made to a workspace project, ensure the workspace configuration supports CI builds (e.g., use `default-members` to exclude platform-specific members that require system dependencies)
5. Verify all unit tests pass with `cargo test`
6. For UI changes, verify Playwright tests pass with `npm test`