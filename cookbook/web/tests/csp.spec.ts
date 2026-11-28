import { test, expect, type Page } from '@playwright/test';
import { pageReady } from './nav';
import { readFileSync } from 'fs';
import { join } from 'path';

/**
 * Every page loads under the production Content Security Policy with no
 * violations, so a new inline `<style>` or `style` attribute (which the
 * policy's `style-src 'self'` blocks) fails here before it ships.
 *
 * `dx serve` doesn't send the Pages headers, so each page's HTML response is
 * given the CSP from public/_headers, the header Cloudflare Pages sends.
 *
 * Before running these tests, start the Dioxus dev server:
 *   mise run dev
 */

const headers = readFileSync(join(__dirname, '../public/_headers'), 'utf8');
const csp = headers
  .split('\n')
  .map((line) => line.trim())
  .find((line) => line.startsWith('Content-Security-Policy:'))!
  .slice('Content-Security-Policy:'.length)
  .trim();

const pages = ['/', '/recipe/1', '/plan/1', '/no-such-page'];

/** Records every CSP violation the page reports, from its first script on. */
async function watchViolations(page: Page): Promise<string[]> {
  const violations: string[] = [];
  await page.exposeFunction('reportCspViolation', (v: string) => violations.push(v));
  await page.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (e) => {
      (window as unknown as { reportCspViolation: (v: string) => void }).reportCspViolation(
        `${e.effectiveDirective} blocked ${e.blockedURI || 'inline'} (${e.sourceFile}:${e.lineNumber})`,
      );
    });
  });
  await page.route('**/*', async (route) => {
    if (route.request().resourceType() !== 'document') return route.continue();
    const response = await route.fetch();
    await route.fulfill({
      response,
      headers: {
        ...response.headers(),
        // dx serve sends no content type for its index.html fallback
        // (/recipe/1 and the like), and WebKit downloads a fulfilled
        // response without one instead of rendering it.
        'content-type': 'text/html; charset=utf-8',
        'content-security-policy': csp,
      },
    });
  });
  return violations;
}

test.describe('Content Security Policy', () => {
  test('the policy allows no inline styles', () => {
    expect(csp).toContain("style-src 'self';");
    expect(csp).not.toContain("'unsafe-inline'");
  });

  for (const path of pages) {
    test(`${path} loads with no CSP violations`, async ({ page }) => {
      const violations = await watchViolations(page);
      await page.goto(path);
      await pageReady(page);
      await expect(page.locator('#header')).toBeVisible();
      expect(violations).toEqual([]);
    });
  }
});
