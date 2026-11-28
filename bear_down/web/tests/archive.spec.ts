import { test, expect, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * The archive of past weeks (docs/plans/2026-11-archive.md, phase 2). The
 * deploy builds archive/ (phase 3), so these tests stub it with this
 * week's sample guide relabelled as earlier weeks, and stub the telemetry
 * Worker to check that a past week sends nothing.
 */
const sample = JSON.parse(readFileSync(join(__dirname, '..', 'public', 'rooting.json'), 'utf8'));

function pastWeek(week: number) {
  return { ...sample, week, generated_at: '2026-11-27T06:00:00Z', banner: null };
}

async function stubArchive(page: Page, weeks: number[]): Promise<void> {
  await page.route('**/archive/index.json', (route) =>
    route.fulfill({ json: { weeks: weeks.map((week) => ({ season: 2026, week })) } }),
  );
  await page.route(/\/archive\/2026-week-(\d\d)\.json$/, (route) => {
    const week = Number(/week-(\d\d)/.exec(route.request().url())?.[1]);
    return route.fulfill({ json: pastWeek(week) });
  });
}

async function stubTelemetry(page: Page): Promise<Record<string, unknown>[]> {
  const events: Record<string, unknown>[] = [];
  await page.route('**/jimhillr-telemetry.*/event', async (route) => {
    events.push(JSON.parse(route.request().postData() ?? 'null'));
    await route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' } });
  });
  return events;
}

const picker = (page: Page) => page.getByLabel('See what the guide says about an earlier week');

test.describe('Past weeks', () => {
  test('@smoke the picker shows a past week without votes, and goes back', async ({ page }, info) => {
    const events = await stubTelemetry(page);
    await stubArchive(page, [10, 11, 12]);
    await page.goto('/?arm=a');
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect.poll(() => events.length).toBe(1);

    // Newest first, without this week (12), which the live guide shows.
    await expect(picker(page).locator('option')).toHaveText(['This week', 'Week 11, 2026 season', 'Week 10, 2026 season']);
    await info.attach('picker-phone', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });

    await picker(page).selectOption({ label: 'Week 11, 2026 season' });
    await expect(page.getByRole('note')).toContainText("You're looking at Week 11, 2026 season, rebuilt with today's model");
    await expect(page.locator('.bears .week')).toHaveText('Week 11, 2026 season');
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect(page.getByRole('group', { name: 'Does this verdict look right?' })).toHaveCount(0);
    await expect(page.locator('.banner')).toHaveCount(0);
    await expect(page.locator('.privacy')).toHaveCount(0);
    await expect(page.getByText(/^Rebuilt 2026-11-27 06:00 UTC/)).toBeVisible();
    // Switching tabs on a past week reports no exposure.
    await page.getByRole('button', { name: 'Make the playoffs' }).click();
    await expect(page.locator('.explain')).toContainText('making the playoffs');
    await info.attach('past-week-phone', { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });

    await page.getByRole('button', { name: 'Back to this week' }).click();
    await expect(page.locator('.bears .week')).toHaveText('Week 12, 2026 season');
    await expect(page.getByRole('group', { name: 'Does this verdict look right?' }).first()).toBeVisible();
    await expect(picker(page)).toHaveValue('');
    expect(events.filter((e) => e.kind === 'exposure')).toHaveLength(events.length);
    expect(events.every((e) => e.week === 12)).toBe(true);
  });

  test('another past week replaces the one showing', async ({ page }) => {
    await stubTelemetry(page);
    await stubArchive(page, [10, 11]);
    await page.goto('/?arm=a');
    await picker(page).selectOption({ label: 'Week 11, 2026 season' });
    await expect(page.locator('.bears .week')).toHaveText('Week 11, 2026 season');
    await picker(page).selectOption({ label: 'Week 10, 2026 season' });
    await expect(page.locator('.bears .week')).toHaveText('Week 10, 2026 season');
    await picker(page).selectOption({ label: 'This week' });
    await expect(page.locator('.bears .week')).toHaveText('Week 12, 2026 season');
  });

  test('a past week that fails to load says so', async ({ page }) => {
    await stubTelemetry(page);
    await stubArchive(page, [11]);
    await page.route('**/archive/2026-week-11.json', (route) => route.fulfill({ status: 404, body: 'Not found' }));
    await page.goto('/?arm=a');
    await picker(page).selectOption({ label: 'Week 11, 2026 season' });
    await expect(page.locator('.status.error')).toContainText("Couldn't load the guide for Week 11, 2026 season.");
    await page.getByRole('button', { name: 'Back to this week' }).click();
    await expect(page.locator('li.game').first()).toBeVisible();
  });

  test('without an archive there is no picker', async ({ page }) => {
    await stubTelemetry(page);
    await page.route('**/archive/index.json', (route) => route.fulfill({ status: 404, body: 'Not found' }));
    await page.goto('/?arm=a');
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Past weeks' })).toHaveCount(0);
  });
});
