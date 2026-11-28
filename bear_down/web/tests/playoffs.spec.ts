import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * The playoff guide (decision 0008): a playoff round's rooting.json names
 * its round, and the page drops the goals that are already decided. The
 * fixture is the 2020 wild card round, written by the CLI with
 * `--games core/fixtures/seasons.csv --season 2020 --week 18`.
 */
const wildCard = readFileSync(join(__dirname, 'fixtures', 'rooting-wild-card.json'), 'utf8');

async function serve(page: Page, json: string): Promise<void> {
  await page.route('**/jimhillr-telemetry.*/event', (route) =>
    route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' } }),
  );
  await page.route('**/rooting.json', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: json }),
  );
}

test('a wild card round names the round and keeps only the goals still open', async ({ page }, info) => {
  await serve(page, wildCard);
  await page.goto('/');
  await expect(page.locator('p.week')).toHaveText('Wild card round, 2020 season');
  await expect(page.locator('li.game')).toHaveCount(6);
  await expect(page.getByRole('button', { name: 'Bears Super Bowl' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'NFC North' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Make the playoffs' })).toHaveCount(0);
  // The Bears' own game leads the Super Bowl ranking.
  await expect(page.locator('li.game').first()).toContainText('Saints');
  // UAT: the playoff guide, in the PR's report.
  await info.attach('wild-card-round', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
});

test('after the wild card round only the Super Bowl goal is left', async ({ page }) => {
  const later = JSON.parse(wildCard);
  later.round = 'divisional';
  await serve(page, JSON.stringify(later));
  await page.goto('/');
  await expect(page.locator('p.week')).toHaveText('Divisional round, 2020 season');
  await expect(page.getByRole('button', { name: 'Bears Super Bowl' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'NFC North' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Make the playoffs' })).toHaveCount(0);
});

test('once the Bears are knocked out the page says there is nothing to root for', async ({ page }) => {
  const out = JSON.parse(wildCard);
  out.round = 'divisional';
  out.bears.odds.champion = 0;
  await serve(page, JSON.stringify(out));
  await page.goto('/');
  await expect(page.getByText('The Bears have been eliminated from the playoffs')).toBeVisible();
  await expect(page.locator('li.game')).toHaveCount(0);
});
