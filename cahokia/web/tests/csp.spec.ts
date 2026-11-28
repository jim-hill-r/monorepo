import { test, expect } from '@playwright/test';

/**
 * The Content Security Policy allows no inline styles or scripts
 * (`style-src 'self'`). Every page must load without the browser blocking
 * anything, so a new `style` attribute or `<style>` element fails here
 * instead of silently losing its styling in production.
 *
 * Tagged `@smoke`, so after each deploy they also run against production,
 * where Cloudflare Pages sends the real policy from public/_headers.
 */
const PAGES = ['/', '/about', '/history', '/explore', '/family-tree'];

for (const path of PAGES) {
  test(`${path} loads with no Content Security Policy violations`, {
    tag: ['@req:CAHO-004', '@smoke'],
  }, async ({ page }) => {
    // Record violations from the first script that runs, before the app.
    await page.addInitScript(() => {
      (window as any).__cspViolations = [];
      document.addEventListener('securitypolicyviolation', (e) => {
        (window as any).__cspViolations.push(
          `${e.violatedDirective} blocked ${e.blockedURI || 'inline'}` +
            (e.sample ? `: ${e.sample}` : ''),
        );
      });
    });

    await page.goto(path);
    await expect(page.locator('#sidebar')).toBeVisible();
    await page.waitForLoadState('networkidle');

    const violations = await page.evaluate(() => (window as any).__cspViolations);
    expect(violations).toEqual([]);
  });
}
