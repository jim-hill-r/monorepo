import { expect, test, type Page, type Request } from '@playwright/test';

/**
 * Nothing a learner types or draws leaves the device (SPRK-006): the app
 * loads only from its own site, and no request carries the learner's name
 * or a tracing. Progress is kept in the browser (src/progress.rs).
 */

type Point = { x: number; y: number; type?: string };

/**
 * Hosts only `dx serve` itself asks for: its dev-build toast loads the Inter
 * font from Google Fonts. The release build has no toast, and the live
 * site's Content-Security-Policy allows only its own origin (the
 * connects-only-to-itself-live eval), so the app never reaches them.
 */
const DEV_SERVER_ONLY = new Set(['fonts.googleapis.com', 'fonts.gstatic.com']);

/** A name unlikely to appear in any file name or asset. */
const NAME = 'Zephyrina';

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

test('a learner\'s name and tracings never leave the device @req:SPRK-006', async ({ page, baseURL }) => {
  const site = new URL(baseURL!).host;
  const requests: Request[] = [];
  page.on('request', (request) => requests.push(request));

  await page.goto('/');
  await page.getByPlaceholder('Your name').fill(NAME);
  await page.getByRole('link', { name: 'Begin' }).click();
  await page.getByRole('button', { name: 'Start' }).click();
  await traceRight(page);
  await expect(page.getByText('Great job!')).toBeVisible();
  await page.reload();
  await page.getByRole('button', { name: 'Start' }).click();
  await expect(page.locator('main b')).toHaveText('c');

  expect(requests.length).toBeGreaterThan(0);
  for (const request of requests) {
    const url = new URL(request.url());
    if (url.protocol === 'data:' || url.protocol === 'blob:') continue;
    expect(request.url(), 'the request names the learner').not.toContain(NAME);
    if (DEV_SERVER_ONLY.has(url.host)) continue;
    expect(url.host, request.url()).toBe(site);
    expect(request.postData() ?? '', 'the request carries the learner\'s name').not.toContain(NAME);
  }
  const cookies = await page.context().cookies();
  expect(cookies, 'the app sets no cookies').toEqual([]);
});
