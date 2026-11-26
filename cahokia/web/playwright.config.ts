import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright configuration for Cahokia web application tests.
 * 
 * Tests expect the Dioxus web app to be running on http://localhost:8080
 * Start the dev server with: dx serve --port 8080 (from cahokia/web directory)
 * 
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests',

  /* Wait for the dev server's first build before running tests. */
  globalSetup: './global-setup.ts',
  
  /* Run tests in files in parallel */
  fullyParallel: true,
  
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  
  /* Retry on CI only */
  retries: process.env.CI ? 1 : 0,
  
  /* Use a limited number of workers on CI to balance speed and stability. */
  workers: process.env.CI ? 2 : 4,

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
    // `mise run dev` is the project's standard dev task (dx serve on port 8080).
    // Falls back to `cast run` while the Cast CI still runs these tests.
    command: 'if command -v mise >/dev/null 2>&1; then mise run dev; else cast run; fi',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 600 * 1000, // first dx build in CI is slow
    cwd: __dirname,
    
    stdout: 'pipe',

    stderr: 'pipe',
  },
});
