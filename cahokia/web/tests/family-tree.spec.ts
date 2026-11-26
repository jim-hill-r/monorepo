import { test, expect, Page } from '@playwright/test';

/**
 * Playwright tests for the family tree page, which draws the sample family
 * from cahokia_core with ui::FamilyTreeView.
 *
 * Run tests with:
 *   npm test -- family-tree.spec.ts
 */

const SAMPLE_PEOPLE = [
  'Thomas Reed',
  'Clara Reed',
  'Henry Reed',
  'Alice Moore',
  'Ruth Price',
  'Samuel Price',
  'Daniel Reed',
  'Margaret Reed',
  'Eleanor Price',
  'Sam Price',
];

async function openFamilyTree(page: Page) {
  await page.goto('/family-tree');
  await page.waitForLoadState('networkidle');
}

test.describe('Family tree page', () => {
  test('shows the page heading and the tree drawing', async ({ page }) => {
    await openFamilyTree(page);

    await expect(page.locator('.page-content h2')).toHaveText('Family Tree');
    const tree = page.locator('svg.family-tree');
    await expect(tree).toBeVisible();
    await expect(tree).toHaveAttribute('aria-label', 'Family tree');
  });

  test('draws a box for every person in the sample family', async ({ page }) => {
    await openFamilyTree(page);

    await expect(page.locator('.family-tree-person')).toHaveCount(SAMPLE_PEOPLE.length);
    for (const name of SAMPLE_PEOPLE) {
      await expect(page.locator('.family-tree-name', { hasText: name })).toHaveCount(1);
    }
    await expect(
      page.locator('[data-person-id="thomas"] .family-tree-years'),
    ).toHaveText('1890 – 1962');
  });

  test('draws lines between parents, children and spouses', async ({ page }) => {
    await openFamilyTree(page);

    await expect(page.locator('.family-tree-edge.spouse')).toHaveCount(3);
    await expect(page.locator('.family-tree-edge.parent-child')).toHaveCount(12);
    await expect(
      page.locator('.family-tree-edge.parent-child[data-from="henry"][data-to="daniel"]'),
    ).toHaveCount(1);
  });

  test('keeps a wide tree inside the page', async ({ page }) => {
    await openFamilyTree(page);

    // The tree scrolls sideways inside its container; the page itself must
    // not get wider than the viewport.
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('is linked from the sidebar', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    await page.locator('.sidebar-toggle').click();
    const link = page.locator('.sidebar-links a', { hasText: 'Family Tree' });
    await expect(link).toBeVisible();
    await link.click();

    await expect(page).toHaveURL(/\/family-tree$/);
    await expect(page.locator('svg.family-tree')).toBeVisible();
    // Following the link closes the sidebar so the tree isn't covered.
    await expect(page.locator('#sidebar')).not.toHaveClass(/open/);
  });
});
