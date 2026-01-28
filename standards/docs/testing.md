# Testing Standards

This document defines the testing standards and requirements for all projects in this monorepo.

## General Testing Requirements

All code changes require appropriate tests. Tests should verify component behavior, logic, edge cases, and ensure that changes don't break existing functionality.

## Unit Tests

All code changes must include unit tests.

### Rust Unit Tests

- Add `#[cfg(test)]` modules with test functions
- Tests should verify component behavior, logic, and edge cases
- Run tests with `cargo test` in the project directory
- Consider using property-based testing with `proptest` or `quickcheck` for complex logic

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let result = my_function(42);
        assert_eq!(result, 84);
    }

    #[test]
    fn test_edge_case() {
        let result = my_function(0);
        assert_eq!(result, 0);
    }
}
```

### TypeScript/JavaScript Unit Tests

- Use the project's testing framework (Jest, Vitest, etc.)
- Test files should be co-located with source files or in a `tests/` directory
- Run tests with `npm test`

## End-to-End Tests

UI changes require end-to-end tests to verify user interactions, navigation, and visual elements.

### Playwright Tests

For Dioxus web applications, Playwright is used for end-to-end testing:

- Add `.spec.ts` files in the `tests/` directory
- Tests should verify user interactions, navigation, and visual elements
- Run tests with `npm test` after starting the dev server (`dx serve --port 8080`)
- See existing test files for examples of patterns and best practices

**Standard Playwright Test Pattern:**

```typescript
import { test, expect } from '@playwright/test';

test('home page loads correctly', async ({ page }) => {
  await page.goto('http://localhost:8080');
  await expect(page.locator('h1')).toContainText('Welcome');
});
```

### SSG Bundle Tests

Tests that validate static site generation (like `ssg-bundle.spec.ts`) do not require a dev server:

- These tests create and test their own static site
- They require the `dx` CLI to be installed (`cargo install dioxus-cli`)
- They run `dx bundle --platform web --ssg` to generate static files
- They verify the generated static site works correctly

**Example SSG Test Pattern:**

```typescript
import { test, expect } from '@playwright/test';
import { execSync } from 'child_process';

test('SSG bundle generates and serves correctly', async ({ page }) => {
  // Generate SSG bundle
  execSync('dx bundle --platform web --ssg', { cwd: __dirname });
  
  // Test the generated files
  await page.goto('file://' + __dirname + '/dist/index.html');
  await expect(page.locator('h1')).toBeVisible();
});
```

## Running Tests

### Using Cast Commands

The `cast test` command automatically runs appropriate tests for your project:

- **Rust projects**: Runs `cargo test`
- **TypeScript/Node.js projects**: Runs `npm test`
- **Hybrid projects** (both Cargo.toml and package.json): Runs both Rust and TypeScript tests

```bash
cast test
```

The `cast ci` command runs all checks including tests, formatting, linting, and building:

- **Rust projects**: Runs `cargo fmt`, `clippy`, `build`, and `test`
- **TypeScript/Node.js projects**: Runs `npm install`, `npm run lint`, `npm run compile`, and `npm test`
- **Hybrid projects**: Runs both Rust and TypeScript CI checks

```bash
cast ci
```

Always run `cast ci` before completing a task to ensure all tests pass.

### Manual Test Execution

For Rust projects:
```bash
cargo test
```

For TypeScript/Node.js projects:
```bash
npm test
```

For Playwright tests specifically:
```bash
npm test  # Runs all tests including Playwright
npx playwright test  # Runs only Playwright tests
```

## Test Organization

### Test File Locations

- **Rust unit tests**: In `#[cfg(test)]` modules within source files or in separate `tests/` modules
- **Rust integration tests**: In the `tests/` directory at project root
- **TypeScript unit tests**: Co-located with source files (`.test.ts`) or in `tests/` directory
- **Playwright tests**: In `tests/` directory (`.spec.ts` files)

### Test Naming Conventions

- Test functions should clearly describe what they test
- Use descriptive names like `test_returns_error_when_invalid_input` instead of `test1`
- Playwright test descriptions should read like user stories

## Test Coverage Goals

While not strictly enforced, aim for:
- At least 80% code coverage for critical business logic
- 100% coverage for public APIs and error handling paths
- All edge cases and error conditions should be tested

### Measuring Code Coverage

#### Rust Projects - Using Tarpaulin

[cargo-tarpaulin](https://github.com/xd009642/tarpaulin) is the recommended tool for measuring code coverage in Rust projects.

**Installation:**

```bash
cargo install cargo-tarpaulin
```

**Configuration:**

Create a `tarpaulin.toml` file in your project root:

```toml
# Code coverage configuration for cargo-tarpaulin
[report]
# Output formats - generate both text summary and HTML report
out = ["Html", "Stdout"]

[coverage]
# Measure code coverage for the entire project
run-types = ["Tests"]

# Note: Tarpaulin automatically excludes #[cfg(test)] blocks from coverage by default
# Use exclude-files to exclude specific file patterns if needed, e.g.:
# exclude-files = ["tests/**/*.rs", "**/test_*.rs"]

# Show coverage statistics for all files
all = true

# Continue running tests even if some fail
fail-under = 0

[html]
# HTML report configuration
dark_mode = false
```

**Usage:**

```bash
# Generate coverage report
cargo tarpaulin --config tarpaulin.toml

# Open the HTML report (generated as tarpaulin-report.html)
xdg-open tarpaulin-report.html  # Linux
open tarpaulin-report.html      # macOS
start tarpaulin-report.html     # Windows
```

**Gitignore:**

Add coverage reports to `.gitignore`:

```
# Code coverage reports
tarpaulin-report.html
cobertura.xml
lcov.info
```

**Example Projects:**

See `cookbook/web` for a complete example of tarpaulin configuration and usage.

## Continuous Integration

All tests are automatically run in CI/CD pipelines:
- Tests must pass before a PR can be merged
- Failed tests will block the merge
- Fix all test failures before requesting review

## Best Practices

1. **Write tests first** when possible (Test-Driven Development)
2. **Keep tests simple** and focused on a single behavior
3. **Use descriptive assertions** that make failures easy to diagnose
4. **Avoid test interdependence** - each test should be able to run independently
5. **Clean up test data** after tests complete
6. **Mock external dependencies** to keep tests fast and reliable
7. **Test error cases** as thoroughly as success cases
8. **Update tests when requirements change** - don't just fix failing tests to pass

## Skipping Tests

While tests should not be skipped without good reason, when necessary:

- **Rust**: Use `#[ignore]` attribute with a comment explaining why
- **TypeScript**: Use `.skip()` with a comment explaining why

Both ignored/skipped tests should have associated tracking issues.
