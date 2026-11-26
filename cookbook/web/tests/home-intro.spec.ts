import { test, expect } from '@playwright/test';

/**
 * The home page renders content/intro.md as Markdown: one <p> per paragraph,
 * the book title in <em>, and no Markdown markers left as literal text.
 */
test.describe('Home page intro', () => {
  test('renders the intro as separate paragraphs', async ({ page }) => {
    await page.goto('/');
    const paragraphs = page.locator('.intro-description p');
    await expect(paragraphs).toHaveCount(3);
    await expect(paragraphs.first()).toContainText('by Jim R. Hill');
  });

  test('shows the book title as emphasis without Markdown markers', async ({ page }) => {
    await page.goto('/');
    const intro = page.locator('.intro-description');
    await expect(intro.locator('em').first()).toHaveText("The Engineer's 365 Cookbook");
    await expect(intro).not.toContainText('*');
  });
});
