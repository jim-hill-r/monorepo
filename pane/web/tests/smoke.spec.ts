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
const test = base.extend<{ consoleErrors: string[]; githubApi: void }>({
  // The Approve page reads GitHub's public API; answer it with empty lists
  // so the smoke tests never depend on GitHub or its rate limit.
  githubApi: [
    async ({ page }, use) => {
      await page.route('https://api.github.com/**', (route) =>
        route.fulfill({
          json: route.request().url().includes('/search/') ? { total_count: 0, items: [] } : [],
        }),
      );
      await use();
    },
    { auto: true },
  ],
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
  ['Approve', '/approve', 'Approve'],
  ['Acknowledge', '/acknowledge', 'Acknowledge'],
  ['Health', '/health', 'Health'],
  ['Projects', '/projects', 'Projects'],
  ['Money', '/money', 'Money'],
  ['Deliverables', '/deliverables', 'Deliverables'],
  ['Reviews', '/reviews', 'Reviews'],
  ['Human issues', '/human-issues', 'Human issues'],
  ['Work', '/work', 'Work'],
  ['Add', '/add', 'Add'],
];

test('the home page loads the repo snapshot @smoke @req:PANE-002', async ({ page }) => {
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

test('the main nav opens every page @smoke @req:PANE-006', async ({ page }) => {
  await page.goto('/');
  const nav = page.getByRole('navigation', { name: 'Main' });

  for (const [label, path, heading] of NAV) {
    await nav.getByRole('link', { name: label, exact: true }).click();
    await expect(page).toHaveURL(path);
    await expect(page.getByRole('heading', { level: 1 })).toHaveText(heading);
  }
});

test('a project page opens from the projects list @smoke @req:PANE-005', async ({ page }) => {
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

test('a project page opens directly by its path @req:PANE-005', async ({ page }) => {
  await page.goto('/projects/pane/web');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('pane/web');
  await expect(page.getByText(/open TODOs and \d+ FIXes/)).toBeVisible();
});

test('the work page lists items and filters them @smoke @req:PANE-003', async ({ page }) => {
  await page.goto('/work');

  const count = page.getByTestId('work-count');
  await expect(count).toHaveText(/^Showing \d+ of \d+$/);
  const [, shown, total] = /Showing (\d+) of (\d+)/.exec((await count.textContent()) ?? '') ?? [];
  expect(Number(total)).toBeGreaterThan(0);
  expect(shown).toBe(total);
  await expect(
    page.getByTestId('work-items').getByRole('link', { name: /issues\/[^/]+\.md:\d+$/ }).first(),
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

test('the human issues page lists open issues with GitHub links @smoke @req:PANE-001', async ({ page }) => {
  await page.goto('/human-issues');

  const issues = page.getByTestId('human-issues').locator('li');
  await expect(issues.first()).toBeVisible();
  await expect(
    issues.first().getByRole('link', { name: 'Open on GitHub' }),
  ).toHaveAttribute('href', /issues\/human\/[^/]+\.md#L\d+$/);
});

test('deliverables list live sites and render a plan in the page @smoke @req:PANE-004', async ({ page }) => {
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

test('a document opens directly by its path @req:PANE-004', async ({ page }) => {
  await page.goto('/docs/pane/docs/plans/2026-11-review-dashboard.md');

  await expect(page.getByTestId('doc').locator('h1')).toContainText('pane');
  await expect(page.getByTestId('doc').locator('table').first()).toBeVisible();
});

test('an unknown path shows the not found page @req:PANE-006', async ({ page }) => {
  await page.goto('/no-such-page');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Not found');
  await page.getByRole('link', { name: 'Back to Today' }).click();
  await expect(page).toHaveURL('/');
});

test('today lists what is waiting and what is new since the last visit @smoke @req:PANE-001', async ({ page }) => {
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

test('today works without localStorage @req:PANE-001', async ({ page }) => {
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

test('a project page shows its sub-projects, documents, deliverables and work @smoke @req:PANE-005', async ({ page }) => {
  await page.goto('/projects/pane');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('pane');
  await expect(
    page.getByTestId('project-sub-projects').getByRole('link', { name: 'pane/web' }),
  ).toBeVisible();
  await expect(page.getByTestId('project-latest')).toContainText('Latest plan');
  await expect(
    page.getByTestId('project-deliverables').getByRole('link', { name: /jimhillr-pane\.pages\.dev|pane/ }).first(),
  ).toBeVisible();
  const work = page.getByTestId('project-work');
  await expect(work).toContainText(/\d+ open TODOs and \d+ FIXes, including its sub-projects\./);
  await expect(work.locator('li').first()).toBeVisible();

  await page.getByTestId('project-latest').getByRole('link').first().click();
  await expect(page).toHaveURL(/\/docs\/pane\/.+\.md$/);
});

test('the money page shows each project this month, a 12-month chart and the budgets @smoke @req:PANE-010', async ({ page }) => {
  await page.goto('/money');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Money');
  const month = page.getByTestId('money-month');
  await expect(month.getByRole('columnheader', { name: 'Net' })).toBeVisible();
  await expect(month.getByRole('rowheader', { name: 'Total' })).toBeVisible();
  await expect(month.locator('tfoot td').last()).toHaveText(/^−?\$[\d,]+\.\d{2}$/);
  await expect(page.getByTestId('money-chart').locator('svg rect.bar')).toHaveCount(12);
  // Claude Code is the one vendor with a budget today.
  await expect(page.getByTestId('money-shared').getByRole('link', { name: 'Claude Code' })).toBeVisible();
});

test('a project page shows its money from docs/cost.toml @req:PANE-010', async ({ page }) => {
  await page.goto('/projects/pane');

  const money = page.getByTestId('project-money');
  await expect(money.getByRole('heading', { name: 'Money' })).toBeVisible();
  await expect(money.getByRole('link', { name: 'docs/cost.toml' })).toHaveAttribute(
    'href',
    /\/blob\/[0-9a-f]+\/pane\/docs\/cost\.toml$/,
  );
  await expect(page.getByTestId('project-money-month')).toContainText(/net −?\$[\d,]+\.\d{2}/);
});

/**
 * `mise run dev` doesn't build the review decks (the release does), so
 * locally the deck's slides are stubbed with one Marp-like slide. Against
 * the live site (BASE_URL set) the real deck loads.
 */
async function stubDecks(page: import('@playwright/test').Page) {
  if (process.env.BASE_URL) return;
  await page.route('**/reviews/*/deck.html', (route) =>
    route.fulfill({
      contentType: 'text/html',
      body: '<!doctype html><title>deck</title><div class="marpit"><section>Stub slide</section></div>',
    }),
  );
}

test('the reviews page lists the rotation and opens a deck as slides @smoke @req:PANE-008', async ({ page }) => {
  await stubDecks(page);
  await page.goto('/reviews');

  const rotation = page.getByTestId('reviews-rotation');
  await expect(rotation.getByRole('columnheader')).toHaveText(['Date', 'Project', 'Deck', 'Scorecard', 'Signed off', 'Comments']);
  const row = rotation.locator('tbody tr').first();
  await expect(row.locator('td').first()).toHaveText(/^\d{4}-\d{2}-\d{2}$/);
  await expect(row.getByRole('link', { name: 'PDF' })).toHaveAttribute(
    'href',
    /^\/reviews\/[^/]+\/deck\.pdf$/,
  );

  // Each deck's scorecard ratings, counted (every deck has one: the
  // snapshot's every_real_deck_has_a_scorecard test checks it).
  await expect(row.getByTestId('scorecard-summary')).toHaveText(/^\d+ (Good|Fair|Poor)( · \d+ (Good|Fair|Poor))*$/);

  // Who signed the deck off: "Not yet", or the latest reviewer and date.
  await expect(row.getByTestId('review-signed-off')).toHaveText(/^(Not yet|[a-z0-9-]+(, needs changes)?, \d{4}-\d{2}-\d{2}.*)$/);

  await row.getByRole('link', { name: 'Slides' }).click();
  await expect(page).toHaveURL(/\/review\/[^/]+$/);
  await expect(page.getByRole('heading', { level: 1 })).toHaveText(/ deep review$/);
  const scorecard = page.getByTestId('review-scorecard');
  await expect(scorecard.getByRole('columnheader')).toHaveText(['Area', 'Rating', 'Why']);
  await expect(scorecard.locator('tbody tr').first().locator('td').first()).toHaveText(/^(Good|Fair|Poor)$/);
  await expect(page.getByTestId('review-signoffs').getByRole('heading', { level: 2 })).toHaveText('Sign-offs');
  const deck = page.getByTestId('deck');
  await expect(deck).toHaveAttribute('src', /^\/reviews\/[^/]+\/deck\.html$/);
  await expect(page.frameLocator('[data-testid="deck"]').locator('section').first()).toBeVisible();
  await expect(page.getByRole('link', { name: 'Download PDF' })).toHaveAttribute(
    'href',
    /^\/reviews\/[^/]+\/deck\.pdf$/,
  );
  await expect(page.getByRole('link', { name: /^Comment on #\d+$/ })).toHaveAttribute(
    'href',
    /^https:\/\/github\.com\/jimhillr\/monorepo\/pull\/\d+$/,
  );
  await expect(page.getByRole('link', { name: 'deck.md on GitHub' })).toHaveAttribute(
    'href',
    /\/blob\/[0-9a-f]+\/docs\/reviews\/[^/]+\/deck\.md$/,
  );

  await page.getByRole('button', { name: 'Full screen' }).click();
  await expect
    .poll(() => page.evaluate(() => document.fullscreenElement?.getAttribute('data-testid') ?? null))
    .toBe('deck');
});

test('a reviewed project shows its latest review @req:PANE-008', async ({ page }) => {
  await stubDecks(page);
  await page.goto('/reviews');

  const row = page.getByTestId('reviews-rotation').locator('tbody tr').first();
  const project = row.locator('td').nth(1).getByRole('link');
  const name = (await project.textContent())?.trim() ?? '';
  await project.click();
  await expect(page).toHaveURL(`/projects/${name}`);

  const review = page.getByTestId('project-review');
  await expect(review.getByRole('link', { name: /^Deep review, \d{4}-\d{2}-\d{2}$/ })).toBeVisible();
  await expect(review.getByTestId('scorecard-summary')).toBeVisible();
  await review.getByRole('link', { name: /^Deep review/ }).click();
  await expect(page).toHaveURL(/\/review\/[^/]+$/);
});

test('deliverables list the review decks @req:PANE-008', async ({ page }) => {
  await stubDecks(page);
  await page.goto('/deliverables');

  const decks = page.getByTestId('deliverables-reviews');
  await expect(decks.getByRole('heading', { level: 2 })).toContainText('Review decks');
  await expect(decks.getByRole('link', { name: 'Download PDF' }).first()).toHaveAttribute(
    'href',
    /^\/reviews\/[^/]+\/deck\.pdf$/,
  );
  await decks.locator('a.row-title').first().click();
  await expect(page).toHaveURL(/\/review\/[^/]+$/);
  await expect(page.getByTestId('deck')).toBeVisible();
});

test('an unknown review shows a message @req:PANE-008', async ({ page }) => {
  await page.goto('/review/1999-01-01-nothing');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Review not found');
  await page.getByRole('link', { name: 'All reviews' }).click();
  await expect(page).toHaveURL('/reviews');
});
