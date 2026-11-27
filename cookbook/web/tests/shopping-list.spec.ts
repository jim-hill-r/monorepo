import { test, expect } from '@playwright/test';

/**
 * The plan page's shopping list adds up (COOK-003): each ingredient appears
 * once with its quantities summed, and staples are listed separately.
 *
 * Week 48 has seven recipes that each use 2 tablespoons olive oil, 2 cloves
 * garlic and salt and pepper.
 */
test.describe('Shopping list', { tag: '@req:COOK-003' }, () => {
  test('sums repeated ingredients and lists staples apart', async ({ page }) => {
    await page.goto('/plan/48');
    await page.waitForLoadState('networkidle');

    const list = page.locator('.plan-shopping-list');
    const toBuy = list.locator('.shopping-list:not(.shopping-staples) .shopping-item');
    const staples = list.locator('.shopping-staples .shopping-item');

    await expect(list.getByRole('heading', { name: 'Staples' })).toBeVisible();
    await expect(staples).toHaveText([
      '14 cloves garlic',
      '14 tablespoons olive oil',
      'Salt and pepper',
    ]);
    await expect(toBuy.filter({ hasText: '3 cups quinoa' })).toHaveCount(1);
    await expect(toBuy.filter({ hasText: 'olive oil' })).toHaveCount(0);
  });
});
