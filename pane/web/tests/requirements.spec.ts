import { test, expect, type Page } from '@playwright/test';

/**
 * Each Project page's Requirements section: the verdicts and eval runs the
 * snapshot reads from the evals-results branch. The snapshot is served with
 * a fixed `requirements` block, so the tests never depend on today's
 * results; the smoke test checks only the structure, against any snapshot.
 */

const RUN = (status: string, at: string, n: number) => ({
  status,
  at,
  commit: 'abc1234',
  evidence: `https://github.com/jimhillr/monorepo/actions/runs/${n}`,
  note: status === 'pass' ? '1 test passed' : '1 of 2 tests failed',
});

const REQUIREMENTS = {
  generated_at: '2026-11-28T04:37:48Z',
  projects: [
    {
      project: 'pane',
      musts: 2,
      musts_in_production: 2,
      coverage: { requirements: 3, met: 1, percent: 33.3, musts_met: 1, musts_percent: 50, warning: true },
      requirements: [
        {
          requirement: 'PANE-001',
          title: 'Today shows what needs Jim',
          priority: 'must',
          verdict: 'met',
          evals: [{ eval: 'today-live', stage: 'cd', runs: [RUN('pass', '2026-11-28T04:25:07Z', 3), RUN('pass', '2026-11-27T04:25:07Z', 2)] }],
        },
        {
          requirement: 'PANE-002',
          title: 'Pane shows the repo as it is on main',
          priority: 'must',
          verdict: 'not-met',
          reasons: ['snapshot-live failed in cd at 2026-11-28 04:25 UTC'],
          evals: [
            {
              eval: 'snapshot-live',
              stage: 'cd',
              runs: [RUN('fail', '2026-11-28T04:25:07Z', 9), RUN('pass', '2026-11-27T04:25:07Z', 8), RUN('error', '2026-11-26T04:25:07Z', 7)],
            },
          ],
        },
        { requirement: 'PANE-003', title: 'All open work can be found', priority: 'should', verdict: 'unknown', reasons: ['no evals'] },
      ],
    },
  ],
};

async function serveRequirements(page: Page, requirements: unknown) {
  await page.route('**/data/snapshot.json', async (route) => {
    const response = await route.fetch();
    const snapshot = await response.json();
    snapshot.requirements = requirements;
    await route.fulfill({ response, json: snapshot });
  });
}

test('a project page shows each requirement verdict with its evals @req:PANE-005', async ({ page }) => {
  await serveRequirements(page, REQUIREMENTS);
  await page.goto('/projects/pane/web');

  const section = page.getByTestId('project-requirements');
  await expect(section.getByRole('heading', { name: 'Requirements' })).toBeVisible();
  // The sub-project shows its top-level project's requirements.
  await expect(page.getByTestId('project-requirements-light')).toHaveText(
    'Red 1 must(s) not met; 1 of 2 musts and 1 of 3 requirements met',
  );
  await expect(section.getByRole('link', { name: 'pane/docs/requirements.md' })).toHaveAttribute(
    'href',
    /\/blob\/[0-9a-f]+\/pane\/docs\/requirements\.md$/,
  );

  const rows = section.getByTestId('requirement');
  await expect(rows).toHaveCount(3);
  const failing = rows.nth(1);
  await expect(failing).toContainText('Not met PANE-002: Pane shows the repo as it is on main');
  await expect(failing).toContainText('snapshot-live failed in cd');
  const evalLine = failing.getByTestId('requirement-eval');
  await expect(evalLine).toContainText('snapshot-live (cd)');
  await expect(evalLine).toContainText('latest fail 2026-11-28 04:25 UTC, 1 of 2 tests failed');
  await expect(evalLine.getByRole('link', { name: 'Evidence' })).toHaveAttribute('href', /\/actions\/runs\/9$/);
  // History: newest first, each run linking its evidence.
  const history = evalLine.locator('.history .run');
  await expect(history).toHaveCount(3);
  await expect(history.nth(2)).toHaveAttribute('aria-label', /^error, 2026-11-26 04:25 UTC/);
  await expect(history.nth(2)).toHaveAttribute('href', /\/actions\/runs\/7$/);

  await expect(rows.nth(2)).toContainText('No eval results yet.');
});

test('a project without eval results says so', async ({ page }) => {
  await serveRequirements(page, null);
  await page.goto('/projects/pane');

  await expect(page.getByTestId('project-requirements-light')).toHaveText('Grey No eval results');
  await expect(page.getByTestId('project-requirements')).toContainText('built without the eval results');
});

test('a project page has a Requirements section with a light @smoke @req:PANE-007', async ({ page }) => {
  await page.goto('/projects/pane');

  await expect(page.getByTestId('project-requirements-light')).toHaveText(/^(Grey|Green|Amber|Red) .+/);
});
