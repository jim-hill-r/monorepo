import { test, expect, type Page } from '@playwright/test';

/**
 * Smoke tests of the exported home page, at desktop and phone widths (the
 * two projects in playwright.config.ts). They catch what the Rust SSR tests
 * can't: a broken hydration, a static file that 404s, a layout that breaks.
 */

const SECTIONS = ['what-we-do', 'who-we-are', 'how-we-do-it'];

/**
 * Opens the home page and returns the list of problems seen on it (page
 * errors, console errors, local requests that failed), which keeps growing
 * as the test goes on, so tests check it last.
 */
async function openHome(page: Page, path = '/'): Promise<string[]> {
  const problems: string[] = [];
  page.on('pageerror', (err) => problems.push(`page error: ${err.message}`));
  page.on('console', (msg) => {
    if (msg.type() === 'error') problems.push(`console error: ${msg.text()} at ${msg.location().url}`);
  });
  page.on('response', (res) => {
    if (res.url().startsWith('http://localhost') && res.status() >= 400) {
      problems.push(`${res.status()} ${res.url()}`);
    }
  });
  await page.goto(path);
  await page.waitForLoadState('networkidle');
  return problems;
}

test.describe('home page', () => {
  test('a visitor arriving at / sees the home page with no errors', async ({ page }) => {
    const problems = await openHome(page);
    await expect(page).toHaveURL(/\/en-US\/?$/);
    await expect(page).toHaveTitle('Spark Hill - Reading Instruction');
    await expect(page.getByRole('heading', { level: 1 }).first()).toBeVisible();
    await expect(page.locator('main')).toHaveCount(1);
    for (const id of [...SECTIONS, 'testimonials']) {
      await expect(page.locator(`#${id}`)).toBeAttached();
    }
    expect(problems).toEqual([]);
  });

  test('the page hydrates into the Perseus app without changing it', async ({ page }) => {
    const wasm = page.waitForResponse((res) => res.url().endsWith('/.perseus/bundle.wasm'));
    const problems = await openHome(page);
    expect((await wasm).status()).toBe(200);
    // A failed hydration either throws (a page error) or renders the page a
    // second time next to the exported HTML.
    await expect(page.locator('main')).toHaveCount(1);
    await expect(page.getByRole('heading', { level: 1 })).toHaveCount(5);
    await expect(page.getByText('Everyone deserves to read')).toHaveCount(1);
    expect(problems).toEqual([]);
  });

  test('every stylesheet loads and styles the page', async ({ page }) => {
    const problems = await openHome(page);
    const sheets = await page.evaluate(() =>
      Array.from(document.styleSheets).map((s) => s.href).filter((href) => href),
    );
    expect(sheets).toHaveLength(3);
    // bg-sky-900 on the nav comes from the stylesheets: unstyled it is clear.
    const navBackground = await page.locator('nav').evaluate((el) => getComputedStyle(el).backgroundColor);
    expect(navBackground).not.toBe('rgba(0, 0, 0, 0)');
    // theme.css's @font-face: the page's own font, not a fallback.
    expect(await page.evaluate(async () => (await document.fonts.load('16px Niramit')).length)).toBe(1);
    expect(problems).toEqual([]);
  });

  test('every image loads', async ({ page }) => {
    const problems = await openHome(page);
    const images = page.locator('img');
    await expect(images).toHaveCount(2);
    for (const img of await images.all()) {
      await img.scrollIntoViewIfNeeded();
      await expect
        .poll(() => img.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0))
        .toBe(true);
    }
    expect(problems).toEqual([]);
  });

  test('the page fits the screen width', async ({ page }) => {
    const problems = await openHome(page);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    expect(problems).toEqual([]);
  });

  test('a section link opened directly lands on its section', async ({ page }) => {
    const problems = await openHome(page, '/en-US/#how-we-do-it');
    await expect(page.locator('#how-we-do-it')).toBeInViewport();
    expect(problems).toEqual([]);
  });
});

test.describe('nav on desktop', () => {
  test.skip(({ isMobile }) => isMobile, 'the section links are hidden below md');

  for (const id of SECTIONS) {
    test(`the nav link to #${id} scrolls to its section`, async ({ page }) => {
      const problems = await openHome(page);
      await page.locator(`nav a[href$="#${id}"]`).click();
      await expect(page).toHaveURL(new RegExp(`/en-US/#${id}$`));
      await expect(page.locator(`#${id}`)).toBeInViewport();
      expect(problems).toEqual([]);
    });
  }
});

test.describe('nav on a phone', () => {
  test.skip(({ isMobile }) => !isMobile, 'phone layout only');

  test('shows the name and Contact Us but not the section links', async ({ page }) => {
    const problems = await openHome(page);
    const nav = page.locator('nav');
    await expect(nav.getByRole('link', { name: 'Spark Hill' })).toBeVisible();
    await expect(nav.getByRole('link', { name: 'Contact Us' })).toBeVisible();
    for (const id of SECTIONS) {
      await expect(nav.locator(`a[href$="#${id}"]`)).toBeHidden();
    }
    expect(problems).toEqual([]);
  });
});
