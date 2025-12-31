# Playwright Tests for Cahokia Web

This directory contains end-to-end tests for the Cahokia web application using Playwright.

## Prerequisites

- Node.js 18 or higher
- npm
- Dioxus CLI (`dx`) - Required for SSG bundle tests: `cargo install dioxus-cli`

## Setup

Install dependencies:

```bash
npm install
```

Install Playwright browsers:

```bash
npx playwright install --with-deps chromium
```

## Running Tests

### Automatic Dev Server Management

The Playwright configuration now automatically manages the `dx serve` development server:

- **Automatically starts** - The dev server starts automatically when you run tests
- **Waits for ready** - Tests wait until the server is ready before running
- **Reuses server** - In development, reuses an existing server if already running
- **Auto cleanup** - Stops the server automatically after tests complete

You no longer need to manually start `dx serve` before running tests!

### Standard Tests (with Auto-Managed Dev Server)

Simply run the tests, and the dev server will start automatically:

```bash
# Run all tests
npm test

# Run tests in headed mode (see browser)
npm run test:headed

# Run tests in debug mode
npm run test:debug

# Open Playwright UI
npm run test:ui

# Show test report
npm run test:report

# Run specific test file (e.g., SSG bundle test)
npm test -- ssg-bundle.spec.ts
```

### SSG Bundle Tests

Tests that use SSG (Static Site Generation) functionality do not require a dev server:

- **Does NOT require dev server** - They create their own static site and server
- **Requires `dx` CLI** - Must have `dioxus-cli` installed
- **Takes longer** - Bundle creation can take several minutes on first run

SSG test files:
- `ssg-bundle.spec.ts` - Validates that `dx bundle --platform web --ssg` works
- `example-ssg.spec.ts` - Demonstrates running standard tests against SSG bundle

To run SSG tests:
```bash
# Run all SSG tests
npm test -- "*-ssg.spec.ts"

# Run specific SSG test
npm test -- ssg-bundle.spec.ts
npm test -- example-ssg.spec.ts
```

## Writing Tests

Tests are located in the `tests/` directory and should have the `.spec.ts` extension.

### Standard Tests (Dev Server)

Example test that requires dev server:

```typescript
import { test, expect } from '@playwright/test';

test('should load the home page', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  expect(await page.title()).toBeTruthy();
});
```

### SSG Bundle Tests (No Dev Server)

Use the SSG server fixture to test against static bundle:

```typescript
import { expect, createSSGWorkerFixture } from './fixtures/ssg-server';

// Create a test instance with SSG server fixture
const test = createSSGWorkerFixture({
  port: 8091, // Choose an available port
  bundleTimeout: 600000, // 10 minutes for bundle creation
});

test.describe('My SSG Tests', () => {
  // Override the base URL to use the SSG server
  test.use({ baseURL: 'http://localhost:8091' });

  test('should load the home page', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/.+/);
  });
});
```

The SSG fixture provides:
- **Automatic bundle creation** - Builds SSG bundle once per worker
- **Static HTTP server** - Serves the static assets
- **Worker-scoped setup** - Efficient for running multiple tests
- **Automatic cleanup** - Stops server after tests complete

See `example-ssg.spec.ts` for a complete example.

## Configuration

The Playwright configuration is in `playwright.config.ts`. Key settings:

- Base URL: `http://localhost:8080`
- Test directory: `./tests`
- Browsers: Chromium, Firefox, WebKit, Mobile Chrome, Mobile Safari
- Screenshots: Captured on failure
- Traces: Captured on first retry

## CI Integration

The `webServer` configuration in `playwright.config.ts` is now enabled by default. This automatically starts the Dioxus server before running tests:

```typescript
webServer: {
  command: 'dx serve --port 8080',
  url: 'http://localhost:8080',
  reuseExistingServer: !process.env.CI,
  timeout: 120 * 1000,
},
```

In CI environments, the server is started fresh for each test run. In local development, it will reuse an existing server if one is already running on port 8080.
