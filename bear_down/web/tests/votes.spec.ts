import { test, expect, type Page } from '@playwright/test';

/**
 * BEAR-010: fans can say when a verdict looks wrong. The telemetry Worker is
 * stubbed, and each test checks what the page would have sent.
 */
async function stubTelemetry(page: Page): Promise<unknown[]> {
  const events: unknown[] = [];
  await page.route('**/jimhillr-telemetry.*/event', async (route) => {
    const event = JSON.parse(route.request().postData() ?? 'null');
    // Exposures are covered in arms.spec.ts; keep only the votes here.
    if (event?.kind === 'feedback') events.push(event);
    await route.fulfill({ status: 202, headers: { 'access-control-allow-origin': '*' } });
  });
  return events;
}

test.describe('Verdict votes', () => {
  test('@smoke every game asks whether its verdict looks right', async ({ page }) => {
    await stubTelemetry(page);
    await page.goto('/?arm=a');
    const games = page.locator('li.game');
    await expect(games.first()).toBeVisible();
    const count = await games.count();
    await expect(page.getByRole('group', { name: 'Does this verdict look right?' })).toHaveCount(count);
    await expect(page.locator('.privacy')).toContainText('Do Not Track turns voting off');
  });

  test('@smoke a vote says thanks, takes an optional note and is remembered', async ({ page }) => {
    const events = await stubTelemetry(page);
    await page.goto('/?arm=a');
    const game = page.locator('li.game').first();
    await game.getByRole('button', { name: 'No' }).click();
    await expect(game.getByText('Thanks!')).toBeVisible();
    await expect.poll(() => events.length).toBe(1);
    expect(events[0]).toMatchObject({ kind: 'feedback', project: 'bear_down', arm: 'a', tab: 'super_bowl', vote: 'no' });

    await game.getByLabel("What's off? (optional)").fill('Felt like it mattered');
    await game.getByRole('button', { name: 'Send' }).click();
    await expect(game.getByText('Thanks for your vote.')).toBeVisible();
    await expect.poll(() => events.length).toBe(2);
    expect(events[1]).toMatchObject({ vote: 'note', note: 'Felt like it mattered' });

    // One vote per game, tab and week in this browser.
    await page.reload();
    await expect(page.locator('li.game').first().getByText('Thanks for your vote.')).toBeVisible();
    // Another tab is a separate vote.
    await page.getByRole('button', { name: 'Make the playoffs' }).click();
    await expect(page.getByRole('group', { name: 'Does this verdict look right?' }).first()).toBeVisible();
  });

  test('Do Not Track turns voting off', async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(navigator, 'doNotTrack', { get: () => '1' });
    });
    await stubTelemetry(page);
    await page.goto('/?arm=a');
    await expect(page.locator('li.game').first()).toBeVisible();
    await expect(page.getByRole('group', { name: 'Does this verdict look right?' })).toHaveCount(0);
    await expect(page.locator('.privacy')).toHaveCount(0);
  });
});
