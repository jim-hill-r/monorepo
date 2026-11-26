import { test, expect } from '@playwright/test';

/**
 * Tests that the "today's recipe" links never point past day 365.
 *
 * 31 December of a leap year is day 366 of the year, but the cookbook has
 * 365 recipes, so the links must fall back to day 365 instead of opening the
 * "Invalid Day" page.
 *
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 */

test.describe("Today's recipe link on 31 December of a leap year", () => {
  test.beforeEach(async ({ page }) => {
    // Midday local time on day 366 of 2028
    await page.clock.setFixedTime(new Date(2028, 11, 31, 12, 0, 0));
  });

  test('home page links to recipe day 365', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const browseLink = page.getByRole('link', { name: 'Browse Recipes' });
    await expect(browseLink).toHaveAttribute('href', '/recipe/365');
  });

  test('header Recipes link opens a recipe, not the invalid day page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const headerLink = page.locator('header').getByRole('link', { name: 'Recipes', exact: true });
    await expect(headerLink).toHaveAttribute('href', '/recipe/365');

    await headerLink.click();
    await expect(page).toHaveURL(/\/recipe\/365$/);
    await expect(page.getByRole('heading', { name: 'Invalid Day' })).toHaveCount(0);
  });
});
