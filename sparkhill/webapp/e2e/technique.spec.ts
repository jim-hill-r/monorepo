import { expect, test, type Page } from '@playwright/test';
import { saveSettings } from './settings-panel';

/**
 * The practice techniques (src/technique.rs), as on the live app: Tracing
 * animates every stroke, Pattern shows only each stroke's start and end
 * dots, which stay while the learner draws, and Freeform shows nothing.
 * The canvas's pixels are read back to check what it shows.
 */

type Point = { x: number; y: number; type?: string };

/** The start dot's fill and the demonstration's stroke colour (canvas_js.rs). */
const START_GREEN = [0x5a, 0x81, 0x00];
const EEL_TEAL = [0x18, 0xb7, 0xbe];

/** The demonstration's path, in canvas pixels. */
async function demonstration(page: Page): Promise<Point[]> {
  return page.evaluate(() => (window as any).eelCanvas.demonstrationPath() as Point[]);
}

/** The canvas pixel's colour at each point, as [r, g, b]. */
async function colours(page: Page, points: Point[]): Promise<number[][]> {
  return page.evaluate((pts) => {
    const canvas = document.getElementById('eel-canvas') as HTMLCanvasElement;
    const ctx = canvas.getContext('2d')!;
    return pts.map((p) => Array.from(ctx.getImageData(Math.round(p.x), Math.round(p.y), 1, 1).data.slice(0, 3)));
  }, points);
}

/** Whether `rgb` is within a few levels of `want` (anti-aliasing aside). */
function near(rgb: number[], want: number[]): boolean {
  return rgb.every((c, i) => Math.abs(c - want[i]) <= 8);
}

/** Starts the level and waits until the learner can try. */
async function start(page: Page): Promise<void> {
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.getByRole('button', { name: "I'm Ready to Try" })).toBeVisible();
}

/** Draws `path` with the mouse and presses ✓. */
async function traceAndCheck(page: Page, path: Point[]): Promise<void> {
  const box = await page.locator('#eel-canvas').boundingBox();
  if (!box) throw new Error('the canvas is not on the page');
  for (const p of path) {
    await page.mouse.move(box.x + p.x, box.y + p.y);
    if (p.type === 'start') await page.mouse.down();
    if (p.type === 'end') await page.mouse.up();
  }
  await page.getByRole('button', { name: '✓' }).click();
}

test('Pattern picked in the settings keeps the start dots while the learner draws @smoke @req:SPRK-009', async ({ page }) => {
  await page.goto('/');
  await saveSettings(page, { technique: 'Pattern' });
  await page.getByRole('link', { name: 'Begin' }).click();
  await expect(page).toHaveURL(/\/letter$/);
  await start(page);
  expect(await page.evaluate(() => (window as any).eelCanvas.technique())).toBe('Pattern');
  await page.getByRole('button', { name: "I'm Ready to Try" }).click();

  const path = await demonstration(page);
  const starts = path.filter((p) => p.type === 'start');
  expect(starts.length).toBeGreaterThan(0);
  for (const rgb of await colours(page, starts)) {
    expect(near(rgb, START_GREEN), `start dot colour ${rgb}`).toBe(true);
  }

  await traceAndCheck(page, path);
  await expect(page.getByText('Great job!')).toBeVisible();
});

test('Freeform shows no demonstration and still passes a letter written right @req:SPRK-009', async ({ page }) => {
  await page.goto('/letter?practice=b,c&tech=Freeform');
  await start(page);
  expect(await page.evaluate(() => (window as any).eelCanvas.technique())).toBe('Freeform');
  const path = await demonstration(page);
  expect(path.length).toBeGreaterThan(0);
  for (const rgb of await colours(page, path)) {
    expect(near(rgb, EEL_TEAL), `demonstration ink at ${rgb}`).toBe(false);
    expect(near(rgb, START_GREEN), `start dot at ${rgb}`).toBe(false);
  }

  await page.getByRole('button', { name: "I'm Ready to Try" }).click();
  await traceAndCheck(page, path);
  await expect(page.getByText('Great job!')).toBeVisible();
  await expect(page.locator('main b')).toHaveText('c');
});

test('Tracing is the default and the chosen technique is remembered @req:SPRK-009', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByLabel('Technique')).toHaveValue('Tracing');
  await page.keyboard.press('Escape');
  await saveSettings(page, { technique: 'Freeform' });
  await page.reload();
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByLabel('Technique')).toHaveValue('Freeform');
});

/** Traces the letter on show along its demonstration, `times` times, until the level ends. */
async function finishWith(page: Page, technique: string, times: number): Promise<void> {
  expect(await page.evaluate(() => (window as any).eelCanvas.technique())).toBe(technique);
  for (let i = 0; i < times; i++) {
    await page.getByRole('button', { name: "I'm Ready to Try" }).click();
    await traceAndCheck(page, await demonstration(page));
  }
  await expect(page.getByText('You are ready for the next level!')).toBeVisible();
}

test('"Go" after a level done with Tracing starts it again with Pattern, then Freeform, then opens the next level @smoke @req:SPRK-004 @req:SPRK-009', async ({ page }) => {
  // Traced right first time 3 times (STABILIZE_COUNT), b is stable.
  await page.goto('/letter?practice=b');
  await start(page);
  await finishWith(page, 'Tracing', 3);

  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/letter\?/);
  await expect(page.locator('main b')).toHaveText('b');
  await finishWith(page, 'Pattern', 3);

  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/letter\?/);
  await expect(page.locator('main b')).toHaveText('b');
  await finishWith(page, 'Freeform', 3);

  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/word$/);
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByLabel('Technique')).toHaveValue('Tracing');
});

test('the technique a level moved on to is kept, so the settings panel shows it @req:SPRK-009', async ({ page }) => {
  await page.goto('/letter?practice=b');
  await start(page);
  await finishWith(page, 'Tracing', 3);
  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page.getByRole('button', { name: "I'm Ready to Try" })).toBeVisible();
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByLabel('Technique')).toHaveValue('Pattern');
});
