import { chromium, type FullConfig } from '@playwright/test';

/**
 * Waits until the dev server serves a fully loaded page before any test runs.
 *
 * `dx serve` answers the webServer health check before its first build has
 * finished, so the first tests' page.goto calls could outlast their 30s
 * timeout on a cold CI runner. One long page load here absorbs that wait.
 */
export default async function globalSetup(config: FullConfig): Promise<void> {
  const baseURL = config.projects[0]?.use?.baseURL ?? 'http://localhost:8080';
  const browser = await chromium.launch({
    // The sandbox Chromium fallback (standards/tasks/dioxus-web.toml).
    executablePath: process.env.PLAYWRIGHT_CHROMIUM_PATH || undefined,
  });
  try {
    const page = await browser.newPage();
    await page.goto(baseURL, { waitUntil: 'load', timeout: 600 * 1000 });
    await page.waitForLoadState('networkidle', { timeout: 600 * 1000 });
  } finally {
    await browser.close();
  }
}
