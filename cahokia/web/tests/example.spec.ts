import { test, expect } from '@playwright/test';

/**
 * Example Playwright test for Cahokia web application.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cahokia/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Cahokia Web Application', () => {
  test('should load the home page', async ({ page }) => {
    // Navigate to the home page
    await page.goto('/');
    
    // Wait for the page to be fully loaded
    await page.waitForLoadState('networkidle');
    
    // Basic check that the page loaded
    // TODO: Replace with actual selectors from your app
    await expect(page).toHaveTitle(/.+/);
  });

  test('should have a valid HTML structure', async ({ page }) => {
    await page.goto('/');
    
    // Check that basic HTML structure exists
    await expect(page.locator('html')).toBeVisible();
    await expect(page.locator('body')).toBeVisible();
  });
});
