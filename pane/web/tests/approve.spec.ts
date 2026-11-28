import { test, expect } from '@playwright/test';

/**
 * The Approve page (decision 0014): GitHub's public API answers are faked,
 * so the test checks pane's queue, not GitHub's data or rate limit.
 */

const repo = 'https://github.com/jimhillr/monorepo';

function pull(number: number, head: string, base: string, labels: string[], draft = false) {
  return {
    number,
    title: `PR ${number}`,
    html_url: `${repo}/pull/${number}`,
    draft,
    body: `<!-- attribution -->\n_Requested by **Jim**_\n\n**Summary.** Change ${number}, see [the plan](docs/plan.md).\n\n## Testing`,
    labels: labels.map((name) => ({ name })),
    user: { login: 'jimhillr-agt' },
    head: { ref: head },
    base: { ref: base },
    created_at: '2026-11-20T00:00:00Z',
  };
}

const pulls = [
  pull(1, 'clear', 'main', []),
  pull(2, 'costly', 'main', ['cost']),
  pull(3, 'decide', 'main', ['needs-human']),
  pull(4, 'draft', 'main', [], true),
  pull(5, 's1', 'main', []),
  pull(6, 's2', 's1', []),
];

test('the approve page lists green PRs with summaries, badges and approve links @req:PANE-011', async ({ page }) => {
  await page.route('https://api.github.com/**', (route) =>
    route.fulfill({
      json: route.request().url().includes('/search/')
        ? { total_count: 4, items: [1, 2, 3, 4].map((number) => ({ number })) }
        : pulls,
    }),
  );
  await page.goto('/approve');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Approve');
  const approve = page.getByTestId('queue-approve').locator('li');
  await expect(approve).toHaveCount(2);
  await expect(approve.first()).toHaveAttribute('data-attention', 'clear');
  await expect(approve.first()).toContainText('Change 1, see the plan.');
  await expect(approve.first().getByRole('link', { name: 'Approve on GitHub' })).toHaveAttribute(
    'href',
    `${repo}/pull/1/files`,
  );
  await expect(approve.nth(1)).toHaveAttribute('data-attention', 'look');
  await expect(approve.nth(1).locator('.badge', { hasText: 'cost' })).toBeVisible();

  await expect(page.getByTestId('queue-decide').locator('li')).toHaveCount(1);
  await expect(page.getByTestId('queue-stacks')).toContainText('2 pull requests on main');
  await expect(page.getByText('1 drafts not shown')).toBeVisible();
});

test('the approve page says when GitHub refuses @req:PANE-011', async ({ page }) => {
  page.on('console', () => {});
  await page.route('https://api.github.com/**', (route) =>
    route.fulfill({ status: 403, json: { message: 'API rate limit exceeded' } }),
  );
  await page.goto('/approve');

  await expect(page.getByRole('alert')).toContainText('used up');
  await expect(page.getByRole('button', { name: 'Try again' })).toBeVisible();
});

const merged = {
  total_count: 2,
  items: [
    {
      number: 21,
      title: 'pane: a merged change',
      html_url: `${repo}/pull/21`,
      body: '**Summary.** Merged without review.',
      labels: [],
      user: { login: 'jimhillr-agt' },
      pull_request: { merged_at: '2026-11-27T12:00:00Z' },
    },
    {
      number: 22,
      title: 'Tidy the docs',
      html_url: `${repo}/pull/22`,
      body: 'Tidy.',
      labels: [{ name: 'security' }],
      user: { login: 'jimhillr-agt' },
      pull_request: { merged_at: '2026-11-27T13:00:00Z' },
    },
  ],
};

test('merged PRs are acknowledged after the fact or flagged @req:PANE-012', async ({ page }) => {
  await page.route('https://api.github.com/**', (route) => route.fulfill({ json: merged }));
  await page.goto('/acknowledge');

  await expect(page.getByRole('heading', { level: 1 })).toHaveText('Acknowledge');
  const rows = page.getByTestId('ack-list').locator('li');
  await expect(rows).toHaveCount(2);
  // Newest merge first; the security label is orange.
  await expect(rows.first()).toHaveAttribute('data-attention', 'look');
  await expect(rows.nth(1).getByRole('link', { name: 'Flag for follow-up' })).toHaveAttribute(
    'href',
    /^https:\/\/github\.com\/jimhillr\/monorepo\/new\/main\?filename=pane%2Fdocs%2Fissues%2Ffollow-up-pr-21\.md&value=/,
  );

  await rows.first().getByRole('button', { name: 'Acknowledge' }).click();
  await expect(rows).toHaveCount(1);
  await page.reload();
  await expect(page.getByTestId('ack-list').locator('li')).toHaveCount(1);

  await page.getByRole('button', { name: 'Acknowledge all 1' }).click();
  await expect(page.getByText('Every merged pull request is acknowledged.')).toBeVisible();
});
