import { test, expect } from '@playwright/test';

/**
 * Session replay (decision 0001): the home page says visits may be recorded,
 * and PostHog never loads off the production host, so local runs, previews
 * and smoke tests spend no recordings.
 */
test.describe('Session replay privacy', () => {
  test('the home page shows the privacy note', async ({ page }) => {
    await page.goto('/');
    const note = page.locator('.privacy-note');
    await expect(note).toContainText('recorded with PostHog');
    await expect(note).toContainText('never recorded');
  });

  test('PostHog does not load off the production host', async ({ page }) => {
    const posthogRequests: string[] = [];
    page.on('request', (request) => {
      if (request.url().includes('posthog.com')) posthogRequests.push(request.url());
    });
    await page.goto('/');
    await expect(page.locator('.privacy-note')).toBeVisible();
    expect(posthogRequests).toEqual([]);
  });
});
