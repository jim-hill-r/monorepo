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

### Performance tests

How fast things are is tested too, in [performance.md](performance.md):
a flow with a time in its requirement gets a `@perf` Playwright test that
asserts that time (PERF-301), and a Cloudflare Worker gets `load` evals run
against a local `wrangler dev` (PERF-1xx). Page loads are Lighthouse CI's
job ([web.md](web.md)).

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

### Using mise tasks

Every project has the same tasks (see [mise.md](mise.md)):

```bash
mise run test   # the project's tests (cargo test, plus npm test / Playwright for web apps)
mise run ci     # format check, clippy, build and tests: what CI runs
```

Always run `mise run ci` before completing a task to ensure all tests pass.

#### Code Coverage

Line coverage of every Rust project is measured each night (Jim,
2026-11-27) by `nightly-coverage.yml` with
[cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov), through
`standards/scripts/coverage.sh`. It runs the same tests as `mise run ci`
(`cargo test`, so a workspace's default members), one job per top-level
Rust project; a project inside another Cargo workspace is counted in that
workspace.

| Rule | What it checks | Where | Status |
| --- | --- | --- | --- |
| COV-001 | A Rust project's line coverage is at least 80%. | `nightly-coverage.yml` | Warning (`::warning::` on the run) until Jim makes it an error |

Results go to the `coverage-results` data branch (never merged into
`main`): `projects/<project>.json` holds each project's latest number,
`history/<yyyy-mm>.jsonl` every run, and `summary.json` the total (covered
lines over all lines). The README's coverage badge is `badge.svg` on that
branch, and `badges/<project>.svg` is one per project for a project's own
README. Green is 80% or more, yellow 60% or more, red below. It is the
second data branch CI may push to, after `evals-results` (SEC-007).

Coverage isn't measured on PRs, so they stay fast. To see a project's
number locally:

```bash
cd <project>
bash ../standards/scripts/coverage.sh run   # writes target/coverage/coverage.json
cargo llvm-cov --open                       # the HTML report
```

`coverage.sh` uses `cargo-llvm-cov` from your PATH, or else downloads the
pinned release with mise. Coverage files (`lcov.info` and `coverage/`) are
excluded from git via `.gitignore`.

Cost: $0. The repo is public, so Actions minutes are free; the workflow
uses no Rust cache, so it doesn't eat the 10 GB Actions cache budget, and
its artifacts are a few hundred bytes kept for 3 days.

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

COV-001 above measures the first goal repo-wide (as a warning for now);
the others are for review. Aim for:
- At least 80% code coverage for critical business logic
- 100% coverage for public APIs and error handling paths
- All edge cases and error conditions should be tested

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
