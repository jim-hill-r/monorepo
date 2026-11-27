import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import { readFileSync } from 'fs';
import { join } from 'path';

/**
 * Cahokia in every design system combination: five themes, light and
 * dark, standard and accessible (20 in all). Each loads every main page,
 * one test per page, with the reader's saved choice and runs axe for WCAG 2
 * A and AA, plus AAA contrast (7:1) in accessible mode.
 *
 * The themes come from design_system/dist/tokens.json, so a new theme is
 * tested without editing this file. Runs in desktop Chromium only: the other
 * browsers run the rest of the suite.
 *
 * Before running these tests, start the Dioxus dev server:
 *   mise run dev
 */

const tokens = JSON.parse(
  readFileSync(join(__dirname, '../../../design_system/dist/tokens.json'), 'utf8'),
);
const themes: string[] = tokens.$extensions.design_system.themes;
const schemes = ['light', 'dark'];
const pages = ['/', '/about', '/history', '/explore', '/family-tree'];

test.describe('Design system themes', () => {
  test.beforeEach(({}, testInfo) => {
    test.skip(testInfo.project.name !== 'chromium', 'runs in desktop Chromium only');
  });

  for (const theme of themes) {
    for (const scheme of schemes) {
      for (const a11y of ['off', 'on']) {
        const name = `${theme} ${scheme}${a11y === 'on' ? ' accessible' : ''}`;
        // One test per page, so each stays well inside the test timeout:
        // all five pages with axe in one test outran CI's 60s budget.
        for (const path of pages) {
          test(`${name} ${path} has no accessibility violations`, async ({ page }) => {
            await page.addInitScript((choice) => {
              window.localStorage.setItem('ds-theme', JSON.stringify(choice));
            }, { theme, scheme, a11y });

            await page.goto(path);
            await page.waitForLoadState('networkidle');

            const html = page.locator('html');
            await expect(html).toHaveAttribute('data-theme', theme);
            await expect(html).toHaveAttribute('data-scheme', scheme);
            await expect(html).toHaveAttribute('data-a11y', a11y);

            const aa = await new AxeBuilder({ page })
              .withTags(['wcag2a', 'wcag2aa'])
              .analyze();
            expect(aa.violations, `${name} ${path}`).toEqual([]);

            if (a11y === 'on') {
              const aaa = await new AxeBuilder({ page })
                .withRules(['color-contrast-enhanced'])
                .analyze();
              expect(aaa.violations, `${name} ${path} (AAA contrast)`).toEqual([]);
            }
          });
        }
      }
    }
  }

  test('the appearance picker changes and saves the theme', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const html = page.locator('html');
    await expect(html).toHaveAttribute('data-theme', 'earthy');

    await page.locator('.sidebar-toggle').click();
    const picker = page.locator('.ds-theme-picker');
    await picker.getByLabel('Theme').selectOption('gamey');
    await picker.getByLabel('Colours').selectOption('dark');
    await expect(html).toHaveAttribute('data-theme', 'gamey');
    await expect(html).toHaveAttribute('data-scheme', 'dark');

    await page.reload();
    await page.waitForLoadState('networkidle');
    await expect(html).toHaveAttribute('data-theme', 'gamey');
    await expect(html).toHaveAttribute('data-scheme', 'dark');
  });
});
