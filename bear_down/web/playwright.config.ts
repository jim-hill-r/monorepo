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
 * Playwright tests for the bear_down page. They expect the app on
 * http://localhost:8080 (`mise run dev`, which is dx serve) and stub the
 * telemetry Worker, so no test ever posts a real vote.
 */
export default defineConfig({
  testDir: './tests',
  globalSetup: './global-setup.ts',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  timeout: process.env.CI ? 60 * 1000 : 30 * 1000,
  workers: process.env.CI ? 2 : 4,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'off',
    screenshot: 'off',
    video: 'off',
  },
  projects: [
    { name: 'chromium', use: { ...synthetic(devices['Desktop Chrome']) } },
    { name: 'Mobile Chrome', use: { ...synthetic(devices['Pixel 5']) } },
  ],
  webServer: {
    command: 'mise run dev',
    url: 'http://localhost:8080',
    reuseExistingServer: !process.env.CI,
    timeout: 600 * 1000,
    cwd: __dirname,
    stdout: 'pipe',
    stderr: 'pipe',
  },
});
