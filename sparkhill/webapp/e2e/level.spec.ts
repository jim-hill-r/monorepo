import { expect, test, type Page } from '@playwright/test';

/**
 * Drives the practice levels in a browser: Begin, the demonstration, tracing
 * with the mouse, "Try again!", and "Go" from letters to words to capitals to
 * the congratulations page. `?practice=` keeps each level to a few letters or
 * words (src/state.rs), and a letter is traced right by following the
 * demonstration's own path (`eelCanvas.demonstrationPath()`).
 */

/** Times a letter must be traced right first time to be stable (STABILIZE_COUNT). */
const STABILIZE_COUNT = 3;
/** Misses before the next letter comes up (RETRY_LIMIT). */
const RETRY_LIMIT = 3;

type Point = { x: number; y: number; type?: string };

/** Checks the header asks to try `expression`. */
async function expectTry(page: Page, expression: string): Promise<void> {
  await expect(page.locator('main b')).toHaveText(expression);
}

/** Starts the level and waits for the first demonstration. */
async function start(page: Page): Promise<void> {
  await expect(page.getByText('Welcome!')).toBeVisible();
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.getByRole('button', { name: "I'm Ready to Try" })).toBeVisible();
}

/** Draws `path` (canvas pixels) on the canvas with the mouse. */
async function draw(page: Page, path: Point[]): Promise<void> {
  const box = await page.locator('#eel-canvas').boundingBox();
  if (!box) throw new Error('the canvas is not on the page');
  for (const p of path) {
    const x = box.x + p.x;
    const y = box.y + p.y;
    if (p.type === 'start') {
      await page.mouse.move(x, y);
      await page.mouse.down();
    } else {
      await page.mouse.move(x, y);
      if (p.type === 'end') await page.mouse.up();
    }
  }
}

/** Traces the letter or word on show along its demonstration and presses ✓. */
async function traceRight(page: Page): Promise<void> {
  await page.getByRole('button', { name: "I'm Ready to Try" }).click();
  const path = await page.evaluate(() => (window as any).eelCanvas.demonstrationPath() as Point[]);
  expect(path.length).toBeGreaterThan(0);
  await draw(page, path);
  await page.getByRole('button', { name: '✓' }).click();
}

/** Traces the expression on show right until the level is done. */
async function finishLevel(page: Page, traces: number): Promise<void> {
  for (let i = 0; i < traces; i++) {
    await traceRight(page);
  }
  await expect(page.getByText('You are ready for the next level!')).toBeVisible();
}

test('Begin on the home page opens the letter level', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Blue Eel', exact: true })).toBeVisible();
  await expect(page.getByText('Writing made simple!')).toBeVisible();
  await page.getByRole('link', { name: 'Begin' }).click();
  await expect(page).toHaveURL(/\/letter$/);
  await expect(page.getByText('Welcome!')).toBeVisible();
});

test('the letter level starts with b', async ({ page }) => {
  await page.goto('/letter');
  await start(page);
  await expectTry(page, 'b');
});

test('tracing a letter right says "Great job!" and shows the next one', async ({ page }) => {
  await page.goto('/letter?practice=b,c');
  await start(page);
  await expectTry(page, 'b');
  await traceRight(page);
  await expect(page.getByText('Great job!')).toBeVisible();
  await expectTry(page, 'c');
});

test('a scribble says "Try again!" until the retry limit, then moves on', async ({ page }) => {
  await page.goto('/letter?practice=b,c');
  await start(page);
  await page.getByRole('button', { name: "I'm Ready to Try" }).click();
  for (let miss = 1; miss < RETRY_LIMIT; miss++) {
    await draw(page, [
      { x: 20, y: 20, type: 'start' },
      { x: 40, y: 30 },
      { x: 60, y: 20, type: 'end' },
    ]);
    await page.getByRole('button', { name: '✓' }).click();
    await expect(page.getByText('Try again!')).toBeVisible();
    await expectTry(page, 'b');
  }
  await page.getByRole('button', { name: '✓' }).click();
  await expect(page.getByText("Let's try another one.")).toBeVisible();
  await expectTry(page, 'c');
});

test('"Go" after the letter level opens the word level', async ({ page }) => {
  await page.goto('/letter?practice=b');
  await start(page);
  await finishLevel(page, STABILIZE_COUNT);
  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/word$/);
  await expect(page.getByText('Welcome!')).toBeVisible();
});

test('"Go" after the word level opens the capital letter level', async ({ page }) => {
  await page.goto('/word?practice=hi');
  await start(page);
  await expectTry(page, 'hi');
  await finishLevel(page, STABILIZE_COUNT);
  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/capital$/);
  await expect(page.getByText('Welcome!')).toBeVisible();
});

test('the capital level starts with L', async ({ page }) => {
  await page.goto('/capital');
  await start(page);
  await expectTry(page, 'L');
});

test('"Go" after the capital level opens the congratulations page', async ({ page }) => {
  await page.goto('/capital?practice=T');
  await start(page);
  await expectTry(page, 'T');
  await finishLevel(page, STABILIZE_COUNT);
  await page.getByRole('button', { name: 'Go' }).click();
  await expect(page).toHaveURL(/\/congratulations$/);
  await expect(page.getByText('You have graduated!')).toBeVisible();
  await page.getByRole('link', { name: 'Start Over' }).click();
  await expect(page.getByText('Writing made simple!')).toBeVisible();
});
