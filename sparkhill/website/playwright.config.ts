import { defineConfig, devices } from '@playwright/test';

/**
 * A device with `jimhillr-evals` added to its user agent. Every test run
 * says it is synthetic, so the PostHog snippet doesn't load and the
 * telemetry Worker drops its events (standards/docs/evals.md, EVL-005).
 */
const synthetic = (device: (typeof devices)[string]) => ({
  ...device,
  userAgent: `${device.userAgent} jimhillr-evals`,
});

/**
 * Playwright smoke tests for the sparkhill website's static export.
 *
 * They load dist/exported (from `mise run release`, the files ../cloudflare
 * deploys) through e2e/serve.mjs, which serves them as Cloudflare Pages
 * does. `mise run e2e` makes the export first.
 *
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: 'list',

  use: {
    baseURL: 'http://localhost:8090',
    trace: 'on-first-retry',
    screenshot: 'off',
    video: 'off',
    // A sandbox whose Chromium doesn't match this Playwright can point at it.
    launchOptions: { executablePath: process.env.PLAYWRIGHT_CHROMIUM_PATH || undefined },
  },

  projects: [
    { name: 'desktop', use: { ...synthetic(devices['Desktop Chrome']) } },
    { name: 'phone', use: { ...synthetic(devices['Pixel 5']) } },
  ],

  webServer: {
    command: 'node e2e/serve.mjs 8090',
    url: 'http://localhost:8090/',
    reuseExistingServer: !process.env.CI,
    cwd: __dirname,
  },
});
