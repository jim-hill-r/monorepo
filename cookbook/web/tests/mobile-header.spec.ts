import { test, expect } from '@playwright/test';

/**
 * Below 768 px the header is one row: its Home / Recipes / Plans links move
 * into the sidebar, which opens from the menu button.
 */

const MAIN_LINKS = ['Home', 'Recipes', 'Plans'];

test.describe('Mobile header', { tag: '@req:COOK-004' }, () => {
  for (const width of [375, 768]) {
    test.describe(`at ${width} px`, () => {
      test.use({ viewport: { width, height: 700 } });

      test('header is one row with the links hidden', async ({ page }) => {
        await page.goto('/');
        const header = page.locator('#header');
        await expect(header.locator('.header-title')).toBeVisible();
        await expect(header.locator('.header-nav')).toBeHidden();

        const height = await header.evaluate((el) => el.getBoundingClientRect().height);
        expect(height).toBeLessThanOrEqual(61);

        const title = await header.locator('.header-title').boundingBox();
        const hamburger = await header.locator('.hamburger-btn').boundingBox();
        expect(title).not.toBeNull();
        expect(hamburger).not.toBeNull();
        // Title and menu button sit side by side on the one row.
        expect(Math.abs(title!.y + title!.height / 2 - (hamburger!.y + hamburger!.height / 2)))
          .toBeLessThanOrEqual(8);
      });

      test('no sideways scrolling', async ({ page }) => {
        await page.goto('/recipe/10');
        await expect(page.locator('#content h1')).toBeVisible();
        const overflow = await page.evaluate(
          () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(0);
      });

      test('the sidebar holds the main links', async ({ page }) => {
        await page.goto('/');
        // Closed, the sidebar is display: none up to 600 px and slid off
        // screen above that, so check its state rather than visibility.
        const sidebar = page.locator('#sidebar');
        const sidebarNav = sidebar.locator('.sidebar-nav');
        await expect(sidebar).toHaveClass(/hidden/);

        await page.locator('#header .hamburger-btn').click();
        await expect(sidebar).not.toHaveClass(/hidden/);
        await expect(sidebarNav).toBeVisible();
        for (const name of MAIN_LINKS) {
          await expect(sidebarNav.locator('a', { hasText: name })).toBeVisible();
        }

        await sidebarNav.locator('a', { hasText: 'Plans' }).click();
        await expect(page).toHaveURL(/\/plan\/\d+$/);
      });
    });
  }

  test.describe('on desktop', () => {
    test.use({ viewport: { width: 1280, height: 720 } });

    test('links stay in the header, not the sidebar', async ({ page }) => {
      await page.goto('/');
      await expect(page.locator('#sidebar')).toBeVisible();
      await expect(page.locator('#sidebar .sidebar-nav')).toBeHidden();
      for (const name of MAIN_LINKS) {
        await expect(page.locator('#header .header-nav a', { hasText: name })).toBeVisible();
      }
    });
  });
});
