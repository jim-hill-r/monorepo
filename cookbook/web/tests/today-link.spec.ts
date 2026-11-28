import { test, expect } from '@playwright/test';
import { mainNavLink } from './nav';

/**
 * Tests that the "today's recipe" links open the recipe the week's plan puts
 * on today's date, so the Recipes link, the sidebar and the plan agree, and
 * that they never point past day 365.
 *
 * Plans follow ISO weeks, not the day of the year: on Saturday 28 November
 * 2026 (day 332 of the year) week 48's plan has day 335's recipe.
 *
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 */

test.describe("Today's recipe is the plan's recipe for today", { tag: ['@req:COOK-001', '@req:COOK-006'] }, () => {
  test.beforeEach(async ({ page }) => {
    // Midday local time on Saturday 28 November 2026, in ISO week 48
    await page.clock.setFixedTime(new Date(2026, 10, 28, 12, 0, 0));
  });

  test('home page, Recipes link and sidebar open the same recipe', async ({ page }) => {
    await page.goto('/');

    const browseLink = page.getByRole('link', { name: 'Browse Recipes' });
    await expect(browseLink).toHaveAttribute('href', '/recipe/335');

    // In the header, or in the sidebar on phones.
    const recipesLink = await mainNavLink(page, 'Recipes');
    await expect(recipesLink).toHaveAttribute('href', '/recipe/335');

    const sidebarToday = page.locator('#sidebar').getByRole('link', { name: 'Sat, 28-Nov' });
    await expect(sidebarToday).toHaveAttribute('href', '/recipe/335');
  });

  test("this week's plan dates the same recipe today", async ({ page }) => {
    await page.goto('/plan/48');

    const todayInPlan = page.locator('.plan-recipes .recipe-item', { hasText: '(Sat, 28-Nov)' });
    await expect(todayInPlan.getByRole('link')).toHaveAttribute('href', '/recipe/335');
  });
});

test.describe("Today's recipe link on 31 December of a leap year", { tag: '@req:COOK-001' }, () => {
  test.beforeEach(async ({ page }) => {
    // Midday local time on day 366 of 2028, the Sunday of ISO week 52
    await page.clock.setFixedTime(new Date(2028, 11, 31, 12, 0, 0));
  });

  test('home page links to the plan recipe, not past day 365', async ({ page }) => {
    await page.goto('/');

    const browseLink = page.getByRole('link', { name: 'Browse Recipes' });
    await expect(browseLink).toHaveAttribute('href', '/recipe/364');
  });

  test('main Recipes link opens a recipe, not the invalid day page', async ({ page }) => {
    await page.goto('/');

    // In the header, or in the sidebar on phones.
    const headerLink = await mainNavLink(page, 'Recipes');
    await expect(headerLink).toHaveAttribute('href', '/recipe/364');

    await headerLink.click();
    await expect(page).toHaveURL(/\/recipe\/364$/);
    await expect(page.getByRole('heading', { level: 1 })).not.toHaveText('Invalid Day');
  });
});
