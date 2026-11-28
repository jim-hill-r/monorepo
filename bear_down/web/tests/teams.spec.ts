import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * BEAR-011: a fan can pick any team and get its guide (decision 0010). The
 * deploy writes teams/<team>.json; these tests stub it with this week's
 * sample guide relabelled as another team's, and stub the telemetry Worker
 * to check that another team's guide sends nothing.
 */
const sample = JSON.parse(readFileSync(join(__dirname, '..', 'public', 'rooting.json'), 'utf8'));

function guideFor(team: string) {
  return { ...sample, team, banner: null };
}

async function stubTeams(page: Page): Promise<string[]> {
  const fetched: string[] = [];
  await page.route(/\/teams\/([A-Z]+)\.json$/, (route) => {
    const team = /teams\/([A-Z]+)\.json$/.exec(route.request().url())?.[1] ?? '';
    fetched.push(team);
    return route.fulfill({ json: guideFor(team) });
  });
  return fetched;
}

async function stubTelemetry(page: Page): Promise<Record<string, unknown>[]> {
  const events: Record<string, unknown>[] = [];
  await page.route('**/jimhillr-telemetry.*/event', async (route) => {
    events.push(JSON.parse(route.request().postData() ?? 'null'));
    await route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' } });
  });
  return events;
}

const picker = (page: Page) => page.getByLabel('Your team');
const votes = (page: Page) => page.getByRole('group', { name: 'Does this verdict look right?' });

test.describe('Any team', () => {
  test('@smoke picking a team shows its guide, in the link and remembered', async ({ page }, info) => {
    const events = await stubTelemetry(page);
    const fetched = await stubTeams(page);
    await page.goto('/?arm=a');
    await expect(page.locator('.bears h2')).toContainText('Chicago Bears');
    await expect(picker(page)).toHaveValue('CHI');
    await expect(picker(page).locator('option')).toHaveCount(32);
    await expect.poll(() => events.length).toBe(1);

    await picker(page).selectOption({ label: 'Kansas City Chiefs' });
    await expect(page.locator('.bears h2')).toContainText('Kansas City Chiefs');
    await expect(page).toHaveURL(/\?arm=a&team=KC$/);
    expect(fetched).toEqual(['KC']);
    await expect(page.locator('.tagline')).toHaveText('Which NFL games to root for this week, for Chiefs fans.');
    await expect(page.getByRole('button', { name: 'Chiefs Super Bowl' })).toBeVisible();
    await expect(page.locator('.explain')).toContainText("move the Chiefs' chance of winning the Super Bowl");
    // The NFC North goal is for NFC North teams only.
    await expect(page.getByRole('button', { name: 'NFC North' })).toHaveCount(0);
    await expect(page.locator('dt', { hasText: 'All four NFC North teams' })).toHaveCount(0);
    // Votes are for the Bears' guide only, so the A/B test isn't mixed.
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect(votes(page)).toHaveCount(0);
    await expect(page.locator('.privacy')).toHaveCount(0);
    await page.getByRole('button', { name: 'Make the playoffs' }).click();
    await expect(page.locator('.explain')).toContainText("move the Chiefs' chance of making the playoffs");
    await info.attach('chiefs', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });
    expect(events).toHaveLength(1);

    // The link opens on the team, and so does a fresh visit in this browser.
    await page.reload();
    await expect(page.locator('.bears h2')).toContainText('Kansas City Chiefs');
    await page.goto('/');
    await expect(page.locator('.bears h2')).toContainText('Kansas City Chiefs');
    await expect(picker(page)).toHaveValue('KC');

    // Back to the Bears: no team in the link, and votes come back.
    await picker(page).selectOption({ label: 'Chicago Bears' });
    await expect(page.locator('.bears h2')).toContainText('Chicago Bears');
    await expect(page).toHaveURL(/\/$/);
    await expect(votes(page).first()).toBeVisible();
    await expect(page.getByRole('button', { name: 'Bears Super Bowl' })).toBeVisible();
  });

  test('a shared link opens on its team, and an NFC North team keeps that goal', async ({ page }) => {
    await stubTelemetry(page);
    const fetched = await stubTeams(page);
    await page.goto('/?team=det');
    await expect(page.locator('.bears h2')).toContainText('Detroit Lions');
    expect(fetched).toEqual(['DET']);
    await expect(picker(page)).toHaveValue('DET');
    await page.getByRole('button', { name: 'NFC North' }).click();
    await expect(page.locator('.explain')).toContainText('Bears, Lions, Packers and Vikings');
    await expect(page.locator('dt', { hasText: 'All four NFC North teams' })).toBeVisible();
    // Switching to a team outside the NFC North falls back to the Super Bowl goal.
    await picker(page).selectOption({ label: 'Buffalo Bills' });
    await expect(page.locator('.bears h2')).toContainText('Buffalo Bills');
    await expect(page.getByRole('button', { name: 'Bills Super Bowl' })).toHaveAttribute('aria-pressed', 'true');
  });

  test('an unknown team in the link shows the Bears', async ({ page }) => {
    await stubTelemetry(page);
    const fetched = await stubTeams(page);
    await page.goto('/?team=XYZ');
    await expect(page.locator('.bears h2')).toContainText('Chicago Bears');
    expect(fetched).toEqual([]);
  });

  test('the past-weeks picker is for the Bears only', async ({ page }) => {
    await stubTelemetry(page);
    await stubTeams(page);
    await page.route('**/archive/index.json', (route) =>
      route.fulfill({ json: { weeks: [{ season: 2026, week: 11 }] } }),
    );
    await page.goto('/');
    await expect(page.getByLabel('See what the guide says about an earlier week')).toBeVisible();
    await picker(page).selectOption({ label: 'Green Bay Packers' });
    await expect(page.locator('.bears h2')).toContainText('Green Bay Packers');
    await expect(page.getByLabel('See what the guide says about an earlier week')).toHaveCount(0);
  });
});
