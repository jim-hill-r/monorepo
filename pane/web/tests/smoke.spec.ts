import { test as base, expect } from '@playwright/test';

/**
 * Smoke tests for Pane: the home page loads with the snapshot, the main nav
 * works, a project page opens, and no page logs a console error.
 *
 * Tagged `@smoke` for the deploy feedback plan's smoke UAT runs
 * (`npm run smoke`, see playwright.config.ts). They check structure, not
 * particular projects, so they pass against any snapshot of the repo.
 */

/** Fails the test if the page logs a console error or throws. */
const test = base.extend<{ consoleErrors: string[] }>({
  consoleErrors: [
    async ({ page }, use) => {
      const errors: string[] = [];
      page.on('console', (msg) => {
        if (msg.type() === 'error') errors.push(msg.text());
      });
      page.on('pageerror', (err) => errors.push(err.message));
      await use(errors);
      expect(errors, 'console errors').toEqual([]);
    },
    { auto: true },
  ],
});

const NAV: [string, string, string][] = [
  // [nav label, path, page heading]
  ['Today', '/', 'Today'],
  ['Health', '/health', 'Health'],
  ['Projects', '/projects', 'Projects'],
  ['Deliverables', '/deliverables', 'Deliverables'],
  ['Reviews', '/reviews', 'Reviews'],
  ['Human issues', '/human-issues', 'Human issues'],
  ['Work', '/work', 'Work'],
  ['Add', '/add', 'Add'],
];

test('the home page loads the repo snapshot @smoke', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Today');
  await expect(page.getByTestId('snapshot-footer')).toHaveText(
    /^Snapshot of [0-9a-f]{7,}, taken /,
  );
  const projects = page.getByRole('link', { name: /\d+ projects/ });
  await expect(projects).toBeVisible();
  const count = Number((await projects.locator('strong').textContent()) ?? '0');
  expect(count).toBeGreaterThan(0);
  await expect(page.getByRole('alert')).toHaveCount(0);
});

test('the main nav opens every page @smoke', async ({ page }) => {
  await page.goto('/');
  const nav = page.getByRole('navigation', { name: 'Main' });

  for (const [label, path, heading] of NAV) {
    await nav.getByRole('link', { name: label, exact: true }).click();
    await expect(page).toHaveURL(path);
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(heading);
  }
});

test('a project page opens from the projects list @smoke', async ({ page }) => {
  await page.goto('/projects');

  const first = page.getByTestId('projects').locator('a.project-name').first();
  await expect(first).toBeVisible();
  const name = (await first.textContent())?.trim() ?? '';
  expect(name).not.toBe('');

  await first.click();
  await expect(page).toHaveURL(`/projects/${name}`);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(name);
  await expect(page.getByText('No project at this path')).toHaveCount(0);
});

test('a project page opens directly by its path', async ({ page }) => {
  await page.goto('/projects/pane/web');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('pane/web');
  await expect(page.getByText(/open TODOs and \d+ FIXes/)).toBeVisible();
});

test('an unknown path shows the not found page', async ({ page }) => {
  await page.goto('/no-such-page');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Not found');
  await page.getByRole('link', { name: 'Back to Today' }).click();
  await expect(page).toHaveURL('/');
});
