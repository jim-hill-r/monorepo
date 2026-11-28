# Cookbook Web Tests

This directory contains Playwright end-to-end tests for the Cookbook web application.

## Prerequisites

- Node.js (>= 18.0.0)
- npm
- Dioxus CLI (`dx`)
- [mise](https://mise.jdx.dev) (for toolchain management)

## Setup

### Automated Setup (Recommended)

Install the toolchain with mise:

```bash
cd cookbook/web
mise install
```

This will automatically install:
- Dioxus CLI (`dx`)
- Playwright and its browser dependencies
- npm packages from package.json

### Manual Setup (Alternative)

If you prefer to install dependencies manually:

```bash
# Install npm dependencies
npm install

# Install Playwright browsers
npx playwright install --with-deps chromium

# Install Dioxus CLI
cargo install dioxus-cli --version 0.7.2
```

## Running Tests

### Automated Dev Server (Recommended)

The tests are configured to automatically start and stop the development server. Simply run:

```bash
npm test
```

The Playwright configuration will:
1. Start the dev server using `mise run dev`
2. Wait for it to be ready at http://localhost:8080
3. Run all tests
4. Shut down the dev server automatically

### Manual Dev Server (Alternative)

If you prefer to manage the dev server yourself, start it in a separate terminal:

```bash
cd cookbook/web
dx serve --port 8080
```

Then run tests:

```bash
npm test
```

Run tests in headed mode (see the browser):

```bash
npm run test:headed
```

Run tests in debug mode:

```bash
npm run test:debug
```

Run tests with UI mode:

```bash
npm run test:ui
```

View test report:

```bash
npm run test:report
```

## Test Coverage

The test suite covers:

- **Home Page**: Verifies the main page loads with correct content
- **Recipe Routes**: Tests all recipe endpoints (`/recipe/1` through `/recipe/365`)
- **Plan Routes**: Tests all plan endpoints (`/plan/1` through `/plan/52`)
- **Input Validation**: Tests that invalid day and week parameters are properly handled
- **404 Page**: Verifies that invalid routes show the 404 page
- **Navigation**: Tests that "Back to Home" links work correctly
- **Header Navigation**: Tests the persistent header navigation bar functionality
  - Header visibility and content on all pages
  - Navigation link functionality (Home, Recipes, Plans)
  - Header persistence during page navigation
- **Mobile Header** (`mobile-header.spec.ts`): below 768 px the header is
  one row and its Home / Recipes / Plans links move into the sidebar
  - `nav.ts` has `mainNavLink`, which finds those links at any width
- **Recipe Day Links** (`recipe-day-nav.spec.ts`): previous / next day links
  on each recipe, wrapping from day 365 to day 1
- **Authentication Navbar**: Tests the authentication navbar functionality
  - Navbar visibility on all pages
  - Login button or loading/error state display
  - Navbar positioning below header
  - Navbar persistence during navigation
  - Button styling and interactivity

## Writing Tests

When adding new features or routes, please add corresponding tests following the existing patterns in the `routing.spec.ts`, `header-navigation.spec.ts`, and `auth-navbar.spec.ts` files.

Wait for what the test needs, never for `waitForLoadState('networkidle')`:
Playwright discourages it, and the Auth0 discovery request can keep the
network busy past the test timeout on CI. After `page.goto` or
`page.reload`, call `pageReady(page)` from `nav.ts`, which waits for the
page's `#content h1`, its stylesheets and its colour transitions. After a click
that navigates, assert the new URL with `await expect(page).toHaveURL(...)`,
which waits for it. Then use web-first assertions (`await
expect(locator).toBeVisible()` and the like), which retry until they pass.

For more information on Playwright, see the [official documentation](https://playwright.dev/).
