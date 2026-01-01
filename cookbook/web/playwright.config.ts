import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Cookbook web application tests.
 * 
 * Tests expect the Dioxus web app to be running on http://localhost:8080
 * Start the dev server with: dx serve --port 8080 (from cookbook/web directory)
 * 
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',
  
  /* Run tests in files in parallel */
  fullyParallel: true,
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  
  /* Retry on CI only */
  retries: process.env.CI ? 1 : 0,
  
  /* Use a limited number of workers on CI to balance speed and stability. */
  workers: process.env.CI ? 2 : 8,
  
  /* Reporter to use. */
  reporter: 'list',
  
  /* Shared settings for all the projects below. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: 'http://localhost:8080',
    
    /* Collect trace when retrying the failed test. */
    trace: 'on-first-retry',

    screenshot: 'off',

    video: 'off',
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },

    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },

    /* Test against mobile viewports. */
    {
      name: 'Mobile Chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'Mobile Safari',
      use: { ...devices['iPhone 12'] },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    // IMPORTANT: Use 'cast run' instead of 'dx serve --port 8080'
    // cast run is the correct command as it handles framework detection
    // and ensures consistency with the development workflow.
    // The CI workflow ensures cast is available in PATH before running tests.
    command: 'cast run',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,

    stdout: 'pipe',

    stderr: 'pipe',
  },
});
