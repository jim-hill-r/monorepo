import { test, expect, type Page } from '@playwright/test';

/**
 * The A/B test of the "doesn't matter" line (fan feedback plan, step 4).
 * Arm A uses the published 0.05-point line, arm B 0.02 points. The
 * telemetry Worker is stubbed.
 */
async function stubTelemetry(page: Page): Promise<Record<string, unknown>[]> {
  const events: Record<string, unknown>[] = [];
  await page.route('**/jimhillr-telemetry.*/event', async (route) => {
    events.push(JSON.parse(route.request().postData() ?? 'null'));
    await route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' } });
  });
  return events;
}

async function picks(page: Page): Promise<number> {
  await expect(page.locator('li.game').first()).toBeVisible();
  return page.locator('li.game:not(.neutral)').count();
}

for (const arm of ['a', 'b']) {
  test(`@smoke arm ${arm} shows its verdicts and reports one exposure`, async ({ page }, info) => {
    const events = await stubTelemetry(page);
    await page.goto(`/?arm=${arm}`);
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect.poll(() => events.filter((e) => e.kind === 'exposure').length).toBe(1);
    expect(events[0]).toMatchObject({ kind: 'exposure', arm, tab: 'super_bowl' });
    // UAT: what each arm sees, in the PR's report.
    await info.attach(`arm-${arm}`, { body: await page.screenshot({ fullPage: true }), contentType: 'image/png' });

    // Switching tabs reports that tab once per visit.
    await page.getByRole('button', { name: 'Make the playoffs' }).click();
    await page.getByRole('button', { name: 'Bears Super Bowl' }).click();
    await expect.poll(() => events.filter((e) => e.kind === 'exposure').length).toBe(2);
    expect(events[1]).toMatchObject({ arm, tab: 'playoffs' });
  });
}

test('arm B never has fewer picks than arm A', async ({ page }) => {
  await stubTelemetry(page);
  await page.goto('/?arm=a');
  const a = await picks(page);
  await page.goto('/?arm=b');
  const b = await picks(page);
  expect(b).toBeGreaterThanOrEqual(a);
});

test('without ?arm= the arm comes from the id kept in this browser', async ({ page }) => {
  await page.addInitScript(() => {
    if (!localStorage.getItem('bear_down.ab_id')) localStorage.setItem('bear_down.ab_id', '00000000000ab');
  });
  const events = await stubTelemetry(page);
  await page.goto('/');
  await expect.poll(() => events.length).toBe(1);
  expect(events[0]).toMatchObject({ kind: 'exposure', arm: 'b' });
});
