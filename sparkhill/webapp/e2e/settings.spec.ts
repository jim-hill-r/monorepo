import { expect, test, type Page } from '@playwright/test';
import { saveSettings } from './settings-panel';

/**
 * The left practice menu and the settings panel, as on the live app
 * (src/main.rs, src/components/settings_panel.rs, src/settings.rs).
 */

type Point = { x: number; y: number; type?: string };

/** Starts the level and waits until the learner can try. */
async function start(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.getByRole('button', { name: "I'm Ready to Try" })).toBeVisible();
}

/** Traces the expression on show along its demonstration and presses ✓. */
async function traceRight(page: Page): Promise<void> {
  await page.getByRole('button', { name: "I'm Ready to Try" }).click();
  const path = await page.evaluate(() => (window as any).eelCanvas.demonstrationPath() as Point[]);
  const box = await page.locator('#eel-canvas').boundingBox();
  if (!box) throw new Error('the canvas is not on the page');
  for (const p of path) {
    await page.mouse.move(box.x + p.x, box.y + p.y);
    if (p.type === 'start') await page.mouse.down();
    if (p.type === 'end') await page.mouse.up();
  }
  await page.getByRole('button', { name: '✓' }).click();
}

test('the menu opens Letter Practice and Word Practice @smoke @req:SPRK-010', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Menu' }).click();
  const menu = page.getByRole('navigation', { name: 'Practice' });
  await expect(menu.getByRole('link', { name: /Capital Practice/ })).toBeVisible();
  await menu.getByRole('link', { name: /Word Practice/ }).click();
  await expect(page).toHaveURL(/\/word$/);
  await expect(menu).toBeHidden();
  await start(page);
  await expect(page.locator('main b')).toHaveText('hi');

  await page.getByRole('button', { name: 'Menu' }).click();
  await menu.getByRole('link', { name: /Letter Practice/ }).click();
  await expect(page).toHaveURL(/\/letter$/);
  await start(page);
  await expect(page.locator('main b')).toHaveText('b');
});

test('the settings panel has Student, Sequence and Technique, and no upload switch @smoke @req:SPRK-010 @req:SPRK-006', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).click();
  const panel = page.getByRole('dialog', { name: 'Settings' });
  await expect(panel.getByLabel('Student')).toBeVisible();
  await expect(panel.getByLabel('Sequence')).toBeVisible();
  await expect(panel.getByLabel('Technique')).toHaveValue('Tracing');
  await expect(panel.getByText(/upload/i)).toHaveCount(0);
  await expect(panel.getByRole('checkbox')).toHaveCount(0);
  await page.keyboard.press('Escape');
  await expect(panel).toBeHidden();
});

test('a sequence practises only its letters, and its words on the word level @smoke @req:SPRK-008', async ({ page }) => {
  await page.goto('/');
  await saveSettings(page, { sequence: 'd, f, fun' });
  await page.getByRole('link', { name: 'Begin' }).click();
  await start(page);
  await expect(page.locator('main b')).toHaveText('d');
  await traceRight(page);
  await expect(page.locator('main b')).toHaveText('f');

  await page.goto('/word');
  await start(page);
  await expect(page.locator('main b')).toHaveText('fun');

  // The capital level isn't in the sequence, so it has its own groups.
  await page.goto('/capital');
  await start(page);
  await expect(page.locator('main b')).toHaveText('L');
});

test('saving the settings during practice starts the level again with them @req:SPRK-010', async ({ page }) => {
  await page.goto('/letter');
  await start(page);
  await expect(page.locator('main b')).toHaveText('b');

  await saveSettings(page, { sequence: 'g' });
  await expect(page.getByText('Welcome!')).toBeVisible();
  await start(page);
  await expect(page.locator('main b')).toHaveText('g');

  // Clearing the sequence brings the whole level back, where it was left.
  await saveSettings(page, { sequence: '' });
  await start(page);
  await expect(page.locator('main b')).toHaveText('b');
});

test('a sequence keeps its own progress, apart from the whole level @req:SPRK-005 @req:SPRK-008', async ({ page }) => {
  await page.goto('/');
  await saveSettings(page, { student: 'Ada', sequence: 'd, f' });
  await page.goto('/letter');
  await start(page);
  await traceRight(page);
  await expect(page.locator('main b')).toHaveText('f');

  await saveSettings(page, { sequence: '' });
  await start(page);
  await expect(page.locator('main b')).toHaveText('b');

  await saveSettings(page, { sequence: 'd, f' });
  await start(page);
  await expect(page.locator('main b')).toHaveText('f');
});
