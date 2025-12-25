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

### Standard Tests (Require Dev Server)

Before running most tests, you need to start the Dioxus development server:

```bash
# From the cahokia/web directory
dx serve --port 8080
```

### Run Tests

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

### SSG Bundle Test

The `ssg-bundle.spec.ts` test is special - it validates Static Site Generation (SSG) functionality:

- **Does NOT require dev server** - It creates its own static site and server
- **Requires `dx` CLI** - Must have `dioxus-cli` installed
- **Takes longer** - Bundle creation can take several minutes on first run
- **Tests**: Verifies `dx bundle --platform web --ssg` works and produces a functional static site

To run only the SSG test:
```bash
npm test -- ssg-bundle.spec.ts
```

## Writing Tests

Tests are located in the `tests/` directory and should have the `.spec.ts` extension.

Example test:

```typescript
import { test, expect } from '@playwright/test';

test('should load the home page', async ({ page }) => {
  await page.goto('/');
  await page.waitForLoadState('networkidle');
  expect(await page.title()).toBeTruthy();
});
```

## Configuration

The Playwright configuration is in `playwright.config.ts`. Key settings:

- Base URL: `http://localhost:8080`
- Test directory: `./tests`
- Browsers: Chromium, Firefox, WebKit, Mobile Chrome, Mobile Safari
- Screenshots: Captured on failure
- Traces: Captured on first retry

## CI Integration

The `webServer` configuration in `playwright.config.ts` is commented out by default. You can uncomment it to automatically start the Dioxus server before running tests in CI.

```typescript
webServer: {
  command: 'dx serve --port 8080',
  url: 'http://localhost:8080',
  reuseExistingServer: !process.env.CI,
  timeout: 120 * 1000,
},
```
