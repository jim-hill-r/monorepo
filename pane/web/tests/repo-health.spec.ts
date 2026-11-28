import { test, expect, type Page } from '@playwright/test';

/**
 * Today's Repo health summary and the Health page: the badges from
 * standards/repo-health.toml, the same file the README's Repo health tables
 * are written from. The snapshot is served with a fixed `repo_health` block
 * so the tests don't depend on the real file's rows; the smoke test checks
 * the real snapshot has some.
 */

const badge = (name: string, alt = name) => ({
  alt,
  image: `data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="80" height="20"/>`,
  link: `https://github.com/jimhillr/monorepo/actions/workflows/${name}.yml`,
});

const REPO_HEALTH = {
  domain: [
    {
      name: 'Build and deploy',
      summary: 'Whether main builds.',
      row: [
        {
          area: 'Main CI/CD',
          runs: 'Every push to `main`',
          checks: 'Repo audit, then CI ([evals.md](standards/docs/evals.md))',
          red: 'the audit failed',
          pane: '/projects',
          badge: [badge('push-main', 'Main CI/CD')],
        },
        { area: 'Nightly CI', runs: 'Daily', checks: 'CI everywhere', red: 'CI failed', badge: [badge('nightly-ci')] },
      ],
    },
    {
      name: 'Threats',
      summary: 'Scans and secrets.',
      row: [
        {
          area: 'Secrets and free tiers',
          runs: 'Weekly',
          checks: 'gitleaks',
          red: 'a secret leaked',
          pane: '/money',
          badge: [badge('all-free-tier-usage'), badge('extra')],
        },
      ],
    },
  ],
};

async function serveRepoHealth(page: Page) {
  await page.route('**/data/snapshot.json', async (route) => {
    const response = await route.fetch();
    const snapshot = await response.json();
    snapshot.repo_health = REPO_HEALTH;
    await route.fulfill({ response, json: snapshot });
  });
}

test('Today opens with the README badges by domain @req:PANE-014', async ({ page }) => {
  await serveRepoHealth(page);
  await page.goto('/');

  const summary = page.getByTestId('today-health');
  await expect(summary.getByRole('heading', { name: /Repo health/ })).toContainText('3 checks');
  const domains = summary.getByTestId('health-domain');
  await expect(domains).toHaveCount(2);
  await expect(domains.nth(1)).toContainText('Threats');
  await expect(domains.nth(1).getByRole('img')).toHaveCount(2);
  await expect(domains.nth(0).getByRole('link', { name: 'Main CI/CD' })).toHaveAttribute(
    'href',
    'https://github.com/jimhillr/monorepo/actions/workflows/push-main.yml',
  );

  await domains.nth(0).getByRole('link', { name: 'Build and deploy' }).click();
  await expect(page).toHaveURL(/\/health$/);
  await expect(page.getByRole('heading', { name: 'Health', level: 1 })).toBeVisible();
});

test('the Health page drills into each check @req:PANE-014', async ({ page }) => {
  await serveRepoHealth(page);
  await page.goto('/health');

  const sections = page.getByTestId('health-section');
  await expect(sections).toHaveCount(2);
  await expect(page.locator('#threats')).toContainText('Scans and secrets.');

  const main = page.locator('#main-ci-cd');
  await expect(main).toContainText('Runs: Every push to main');
  await expect(main).toContainText('Red means: the audit failed');
  await expect(main.getByRole('link', { name: 'evals.md' })).toHaveAttribute(
    'href',
    /\/blob\/[0-9a-f]+\/standards\/docs\/evals\.md$/,
  );
  await main.getByRole('link', { name: 'Projects' }).click();
  await expect(page).toHaveURL(/\/projects$/);

  await page.goto('/health');
  await expect(page.locator('#secrets-and-free-tiers').getByRole('link', { name: 'Money' })).toBeVisible();
  // Nightly CI has no pane page.
  await expect(page.locator('#nightly-ci')).not.toContainText('In pane');
});

test('Today and Health show the real repo health badges @smoke @req:PANE-014', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('health-domain').first()).toBeVisible();
  await page.goto('/health');
  await expect(page.getByTestId('health-check').first()).toBeVisible();
});
