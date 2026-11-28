import { expect, test } from '@playwright/test';

/**
 * The app installs to a phone's home screen like blue.eel.education: the
 * page links a web app manifest whose icons load, and has the live app's
 * title, theme colour and no-zoom viewport (index.html).
 */
test('links a manifest with icons that load @smoke @req:SPRK-007', async ({ page, request }) => {
  await page.goto('/');
  await expect(page).toHaveTitle('Blue Eel');
  await expect(page.locator('meta[name="theme-color"]')).toHaveAttribute('content', '#178CA4');
  await expect(page.locator('meta[name="viewport"]')).toHaveAttribute('content', /user-scalable=no/);

  const href = await page.locator('link[rel="manifest"]').getAttribute('href');
  expect(href).toBe('/assets/manifest.webmanifest');
  const manifest = await (await request.get(href!)).json();
  expect(manifest.name).toBe('Blue Eel');
  expect(manifest.display).toBe('standalone');
  expect(manifest.theme_color).toBe('#178CA4');

  const icons: string[] = manifest.icons.map((icon: { src: string }) => icon.src);
  icons.push((await page.locator('link[rel="apple-touch-icon"]').getAttribute('href'))!);
  for (const src of icons) {
    const response = await request.get(src);
    expect(response.ok(), src).toBeTruthy();
    expect(response.headers()['content-type'], src).toMatch(/^image\//);
  }
});
