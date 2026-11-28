import { expect, test, type Page } from '@playwright/test';
import { saveSettings } from './settings-panel';

/**
 * Each learner's progress is kept in the browser (src/progress.rs) under the
 * name typed as Student in the settings panel: a reload
 * carries on where the learner was, and another name starts afresh.
 */

type Point = { x: number; y: number; type?: string };

/** Traces the letter on show along its demonstration and presses ✓. */
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

async function beginAs(page: Page, name: string): Promise<void> {
  await page.goto('/');
  await saveSettings(page, { student: name });
  await page.getByRole('link', { name: 'Begin' }).click();
  await page.getByRole('button', { name: 'Start' }).click();
}

test('a reload carries on where the learner was @smoke @req:SPRK-005', async ({ page }) => {
  await beginAs(page, 'Ada');
  await expect(page.locator('main b')).toHaveText('b');
  await traceRight(page);
  await expect(page.locator('main b')).toHaveText('c');

  await page.reload();
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.locator('main b')).toHaveText('c');

  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByLabel('Student')).toHaveValue('Ada');
});

test('another learner starts afresh @smoke @req:SPRK-005', async ({ page }) => {
  await beginAs(page, 'Ada');
  await traceRight(page);
  await expect(page.locator('main b')).toHaveText('c');

  await beginAs(page, 'Bo');
  await expect(page.locator('main b')).toHaveText('b');
});
