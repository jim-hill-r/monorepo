import { test, expect } from '@playwright/test';

/** Each recipe links to the day before and the day after, round the year. */

const CASES = [
  { day: 10, previous: 9, next: 11 },
  { day: 1, previous: 365, next: 2 },
  { day: 365, previous: 364, next: 1 },
];

test.describe('Recipe previous / next day links', { tag: '@req:COOK-006' }, () => {
  for (const { day, previous, next } of CASES) {
    test(`day ${day} links to days ${previous} and ${next}`, async ({ page }) => {
      await page.goto(`/recipe/${day}`);
      const dayNav = page.getByRole('navigation', { name: 'Other days' });
      await expect(dayNav.getByRole('link', { name: 'Previous day' }))
        .toHaveAttribute('href', `/recipe/${previous}`);
      await expect(dayNav.getByRole('link', { name: 'Next day' }))
        .toHaveAttribute('href', `/recipe/${next}`);
    });
  }

  test('next and previous move between recipes', async ({ page }) => {
    await page.goto('/recipe/10');
    const heading = page.locator('#content h1');
    await expect(heading).toBeVisible();
    const day10 = await heading.textContent();

    const dayNav = page.getByRole('navigation', { name: 'Other days' });
    await dayNav.getByRole('link', { name: 'Next day' }).click();
    await expect(page).toHaveURL('/recipe/11');
    await expect(heading).not.toHaveText(day10 ?? '');

    await dayNav.getByRole('link', { name: 'Previous day' }).click();
    await expect(page).toHaveURL('/recipe/10');
    await expect(heading).toHaveText(day10 ?? '');
  });
});
