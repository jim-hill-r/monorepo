import { test, expect, Page } from '@playwright/test';

/**
 * Cahokia uses the design system's earthy theme by default: the page's
 * colours come from its --ds- tokens (design_system/dist/tokens.css), not
 * from colours written into Cahokia's own CSS.
 *
 * The dev server is automatically started by Playwright before tests run.
 * See playwright.config.ts webServer configuration for details.
 *
 * Run tests with:
 *   npm test -- earthy-colors.spec.ts
 */

/** A token's value as the browser computes a colour, e.g. "rgb(31, 24, 18)". */
async function token(page: Page, name: string): Promise<string> {
  return page.evaluate((n) => {
    const probe = document.createElement('span');
    probe.style.color = `var(${n})`;
    document.body.appendChild(probe);
    const value = getComputedStyle(probe).color;
    probe.remove();
    return value;
  }, name);
}

async function style(page: Page, selector: string, property: string): Promise<string> {
  return page.locator(selector).first().evaluate(
    (el, p) => getComputedStyle(el).getPropertyValue(p),
    property,
  );
}

test.describe('Earthy theme', () => {
  test('is the default theme', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'earthy');
  });

  test('keeps the dark earthy look for dark-mode users', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    // Earthy dark bg #1f1812 and text #f3e7d8.
    expect(await style(page, 'body', 'background-color')).toBe('rgb(31, 24, 18)');
    expect(await style(page, 'body', 'color')).toBe('rgb(243, 231, 216)');
  });

  test('uses light earthy colours for light-mode users', async ({ page }) => {
    await page.emulateMedia({ colorScheme: 'light' });
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    // Earthy light bg #f7f1e8 and text #2b2118.
    expect(await style(page, 'body', 'background-color')).toBe('rgb(247, 241, 232)');
    expect(await style(page, 'body', 'color')).toBe('rgb(43, 33, 24)');
  });

  test('the header, hero and sidebar take their colours from the tokens', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    expect(await style(page, '#header', 'background-color')).toBe(await token(page, '--ds-color-accent'));
    expect(await style(page, '.header-title h1', 'color')).toBe(await token(page, '--ds-color-on-accent'));
    expect(await style(page, '#hero h1', 'color')).toBe(await token(page, '--ds-color-accent'));
    expect(await style(page, '#sidebar', 'background-color')).toBe(await token(page, '--ds-color-surface-raised'));
    expect(await style(page, '#navbar', 'background-color')).toBe(await token(page, '--ds-color-surface'));
  });

  test('every page uses the same background', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const bg = await token(page, '--ds-color-bg');
    for (const link of ['About', 'History', 'Explore']) {
      await page.locator(`.header-nav a:has-text("${link}")`).click();
      await page.waitForLoadState('networkidle');
      expect(await style(page, 'body', 'background-color')).toBe(bg);
    }
  });
});
