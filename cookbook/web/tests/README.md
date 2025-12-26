# Cookbook Web Tests

This directory contains Playwright end-to-end tests for the Cookbook web application.

## Prerequisites

- Node.js (>= 18.0.0)
- npm
- Dioxus CLI (`cargo install dioxus-cli`)

## Setup

Install dependencies:

```bash
npm install
```

Install Playwright browsers:

```bash
npx playwright install
```

## Running Tests

### Start the Development Server

Before running tests, start the Dioxus development server in a separate terminal:

```bash
cd cookbook/web
dx serve --port 8080
```

### Run Tests

Run all tests:

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
- **404 Page**: Verifies that invalid routes show the 404 page
- **Navigation**: Tests that "Back to Home" links work correctly

## Writing Tests

When adding new features or routes, please add corresponding tests following the existing patterns in the `routing.spec.ts` file.

For more information on Playwright, see the [official documentation](https://playwright.dev/).
