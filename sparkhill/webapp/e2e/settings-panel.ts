import { expect, type Page } from '@playwright/test';

/** What to type or pick in the settings panel; anything left out is kept. */
export type SettingsChoice = { student?: string; sequence?: string; technique?: string };

/**
 * Opens the settings panel behind the top-right button, fills in `choice`
 * and saves it (src/components/settings_panel.rs).
 */
export async function saveSettings(page: Page, choice: SettingsChoice): Promise<void> {
  await page.getByRole('button', { name: 'Settings' }).click();
  const panel = page.getByRole('dialog', { name: 'Settings' });
  await expect(panel).toBeVisible();
  if (choice.student !== undefined) await panel.getByLabel('Student').fill(choice.student);
  if (choice.sequence !== undefined) await panel.getByLabel('Sequence').fill(choice.sequence);
  if (choice.technique !== undefined) await panel.getByLabel('Technique').selectOption(choice.technique);
  await panel.getByRole('button', { name: 'Save' }).click();
  await expect(panel).toBeHidden();
}
