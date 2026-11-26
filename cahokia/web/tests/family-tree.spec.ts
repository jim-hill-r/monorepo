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

/** The x and y of a `translate(x y) scale(s)` transform. */
function translateOf(transform: string | null): [number, number] {
  const match = /translate\(([-\d.e]+) ([-\d.e]+)\)/.exec(transform ?? '');
  if (!match) throw new Error(`no translate in ${transform}`);
  return [parseFloat(match[1]), parseFloat(match[2])];
}

/** Scrolls the drawing into view and returns where it is on screen. */
async function treeBox(page: Page) {
  const tree = page.locator('svg.family-tree');
  await tree.scrollIntoViewIfNeeded();
  const box = await tree.boundingBox();
  if (!box) throw new Error('tree not drawn');
  return box;
}

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

    // The tree pans and zooms inside its frame; the page itself must not
    // get wider than the viewport.
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });

  test('drags to pan the tree', async ({ page }) => {
    await openFamilyTree(page);

    const content = page.locator('.family-tree-content');
    const before = await content.getAttribute('transform');
    const box = await treeBox(page);
    // Start the drag on the background, below the bottom row of people.
    const x = box.x + 20;
    const y = box.y + box.height - 10;
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x + 60, y - 30, { steps: 5 });
    await page.mouse.up();

    const after = await content.getAttribute('transform');
    expect(after).not.toEqual(before);
    const [bx, by] = translateOf(before);
    const [ax, ay] = translateOf(after);
    expect(ax - bx).toBeCloseTo(60, 0);
    expect(ay - by).toBeCloseTo(-30, 0);
  });

  test('zooms with the buttons, the wheel and Fit', async ({ page, isMobile }) => {
    await openFamilyTree(page);

    const level = page.locator('.family-tree-zoom-level');
    const start = await level.textContent();
    await page.getByRole('button', { name: 'Zoom in' }).click();
    await expect(level).not.toHaveText(start ?? '');
    const zoomed = parseInt((await level.textContent()) ?? '0', 10);
    expect(zoomed).toBeGreaterThan(parseInt(start ?? '0', 10));

    // Phones have no mouse wheel.
    if (!isMobile) {
      const box = await treeBox(page);
      await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await page.mouse.wheel(0, 400);
      await expect
        .poll(async () => parseInt((await level.textContent()) ?? '0', 10))
        .toBeLessThan(zoomed);
    }

    await page.getByRole('button', { name: 'Fit' }).click();
    await expect(level).toHaveText(start ?? '');
  });

  test('pinches to zoom on a touch screen', async ({ page, browserName, isMobile }) => {
    // Touch input is sent through the Chrome DevTools Protocol.
    test.skip(browserName !== 'chromium' || !isMobile, 'needs Chromium touch emulation');
    await openFamilyTree(page);

    const level = page.locator('.family-tree-zoom-level');
    const start = parseInt((await level.textContent()) ?? '0', 10);
    const box = await treeBox(page);
    const cdp = await page.context().newCDPSession(page);
    const touch = (type: string, touchPoints: { x: number; y: number; id: number }[]) =>
      cdp.send('Input.dispatchTouchEvent', { type, touchPoints });
    const cx = box.x + box.width / 2;
    const cy = box.y + box.height / 2;
    await touch('touchStart', [{ x: cx - 20, y: cy, id: 1 }, { x: cx + 20, y: cy, id: 2 }]);
    for (let i = 1; i <= 5; i++) {
      await touch('touchMove', [
        { x: cx - 20 - 4 * i, y: cy, id: 1 },
        { x: cx + 20 + 4 * i, y: cy, id: 2 },
      ]);
    }
    await touch('touchEnd', []);

    // The fingers spread from 40 to 80 pixels apart, so the zoom doubles.
    await expect
      .poll(async () => parseInt((await level.textContent()) ?? '0', 10))
      .toBeGreaterThanOrEqual(start * 2 - 1);
  });

  test('clicking a person highlights their ancestors and descendants', async ({ page }) => {
    await openFamilyTree(page);

    await page.locator('[data-person-id="henry"]').click();
    await expect(page.locator('[data-person-id="henry"]')).toHaveClass(/\bfocus\b/);
    for (const id of ['thomas', 'clara']) {
      await expect(page.locator(`[data-person-id="${id}"]`)).toHaveClass(/\bancestor\b/);
    }
    for (const id of ['daniel', 'margaret']) {
      await expect(page.locator(`[data-person-id="${id}"]`)).toHaveClass(/\bdescendant\b/);
    }
    await expect(page.locator('[data-person-id="ruth"]')).toHaveClass(/\bdimmed\b/);
    await expect(page.locator('.family-tree-edge.on-lineage')).toHaveCount(4);
    await expect(page.locator('.family-tree-status')).toHaveText(
      "Highlighting Henry Reed's 2 ancestors and 2 descendants.",
    );

    // Clicking them again clears the focus.
    await page.locator('[data-person-id="henry"]').click();
    await expect(page.locator('.family-tree-person.dimmed')).toHaveCount(0);
  });

  test('picks a focus person from the list and clears it with Escape', async ({ page }) => {
    await openFamilyTree(page);

    await page.locator('select.family-tree-focus').selectOption('eleanor');
    await expect(page.locator('[data-person-id="eleanor"]')).toHaveClass(/\bfocus\b/);
    await expect(page.locator('.family-tree-person.ancestor')).toHaveCount(4);

    await page.locator('svg.family-tree').focus();
    await page.keyboard.press('Escape');
    await expect(page.locator('.family-tree-person.focus')).toHaveCount(0);
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
