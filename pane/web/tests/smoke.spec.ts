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

test('the work page lists items and filters them @smoke', async ({ page }) => {
  await page.goto('/work');

  const count = page.getByTestId('work-count');
  await expect(count).toHaveText(/^Showing \d+ of \d+$/);
  const [, shown, total] = /Showing (\d+) of (\d+)/.exec((await count.textContent()) ?? '') ?? [];
  expect(Number(total)).toBeGreaterThan(0);
  expect(shown).toBe(total);
  await expect(
    page.getByTestId('work-items').getByRole('link', { name: /ISSUES\.md:\d+$/ }).first(),
  ).toHaveAttribute('href', /^https:\/\/github\.com\/jimhillr\/monorepo\/blob\/[0-9a-f]+\/.+#L\d+$/);

  await page.getByLabel('Project').selectOption('pane');
  await page.getByLabel('Kind').selectOption('todo');
  const filtered = Number(/Showing (\d+)/.exec((await count.textContent()) ?? '')?.[1]);
  expect(filtered).toBeGreaterThan(0);
  expect(filtered).toBeLessThan(Number(total));
  for (const meta of await page.getByTestId('work-items').locator('.meta > span:first-child').all()) {
    await expect(meta).toHaveText('pane');
  }
});

test('the human issues page lists open issues with GitHub links @smoke', async ({ page }) => {
  await page.goto('/human-issues');

  const issues = page.getByTestId('human-issues').locator('li');
  await expect(issues.first()).toBeVisible();
  await expect(
    issues.first().getByRole('link', { name: 'Open on GitHub' }),
  ).toHaveAttribute('href', /HUMAN_ISSUES\.md#L\d+$/);
});

test('deliverables list live sites and render a plan in the page @smoke', async ({ page }) => {
  await page.goto('/deliverables');

  await expect(
    page.getByTestId('deliverables-live-sites').getByRole('link').first(),
  ).toHaveAttribute('href', /^https:\/\//);
  const plan = page.getByTestId('deliverables-plans').locator('a.row-title').first();
  await plan.click();
  await expect(page).toHaveURL(/\/docs\/.+\.md$/);
  await expect(page.getByTestId('doc').getByRole('heading', { level: 1 })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Open on GitHub' })).toHaveAttribute(
    'href',
    /^https:\/\/github\.com\/jimhillr\/monorepo\/blob\//,
  );
});

test('a document opens directly by its path', async ({ page }) => {
  await page.goto('/docs/pane/docs/plans/2026-11-review-dashboard.md');

  await expect(page.getByTestId('doc').locator('h1')).toContainText('pane');
  await expect(page.getByTestId('doc').locator('table').first()).toBeVisible();
});

test('an unknown path shows the not found page', async ({ page }) => {
  await page.goto('/no-such-page');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Not found');
  await page.getByRole('link', { name: 'Back to Today' }).click();
  await expect(page).toHaveURL('/');
});

test('today lists what is waiting and what is new since the last visit @smoke', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByTestId('today-waiting').getByRole('heading', { level: 2 })).toContainText(
    'Waiting on you',
  );
  // A first visit has nothing to compare with.
  await expect(page.getByTestId('today-new')).toContainText('Nothing to compare with yet');

  // Pretend the last visit was of an older snapshot that saw nothing.
  await page.evaluate(() =>
    localStorage.setItem(
      'pane.visits.v1',
      JSON.stringify({ current: { commit: '0000000', at: 0, seen: [] }, previous: null }),
    ),
  );
  await page.reload();
  const fresh = page.getByTestId('today-new');
  await expect(fresh).toContainText('Compared with your visit');
  await expect(fresh.getByRole('heading', { name: 'TODOs and FIXes' })).toBeVisible();
  await expect(page.getByRole('link', { name: /\d+ new since your last visit/ })).toBeVisible();

  // A reload of the same snapshot keeps comparing with that visit.
  await page.reload();
  await expect(page.getByTestId('today-new')).toContainText('snapshot 0000000');
});

test('today works without localStorage', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, 'localStorage', {
      get() {
        throw new Error('blocked');
      },
    });
  });
  await page.goto('/');

  await expect(page.getByTestId('today-waiting')).toBeVisible();
  await expect(page.getByTestId('today-new')).toContainText("isn't keeping pane's visits");
});

test('a project page shows its sub-projects, documents, deliverables and work @smoke', async ({ page }) => {
  await page.goto('/projects/pane');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('pane');
  await expect(
    page.getByTestId('project-sub-projects').getByRole('link', { name: 'pane/web' }),
  ).toBeVisible();
  await expect(page.getByTestId('project-latest')).toContainText('Latest plan');
  await expect(
    page.getByTestId('project-deliverables').getByRole('link', { name: /pane-dbc\.pages\.dev|pane/ }).first(),
  ).toBeVisible();
  const work = page.getByTestId('project-work');
  await expect(work).toContainText(/\d+ open TODOs and \d+ FIXes, including its sub-projects\./);
  await expect(work.locator('li').first()).toBeVisible();

  await page.getByTestId('project-latest').getByRole('link').first().click();
  await expect(page).toHaveURL(/\/docs\/pane\/.+\.md$/);
});
