# Cookbook Web Tests

This directory contains Playwright end-to-end tests for the Cookbook web application.

## Prerequisites

- Node.js (>= 18.0.0)
- npm
- Dioxus CLI (`dx`)
- Cast CLI (for toolchain management)

## Setup

### Automated Setup (Recommended)

Use the Cast toolchain manager to install all required dependencies:

```bash
cd cookbook/web
cast toolchain install
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
1. Start the dev server using `cast run`
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
- **Authentication Navbar**: Tests the authentication navbar functionality
  - Navbar visibility on all pages
  - Login button or loading/error state display
  - Navbar positioning below header
  - Navbar persistence during navigation
  - Button styling and interactivity

## Writing Tests

When adding new features or routes, please add corresponding tests following the existing patterns in the `routing.spec.ts`, `header-navigation.spec.ts`, and `auth-navbar.spec.ts` files.

For more information on Playwright, see the [official documentation](https://playwright.dev/).
