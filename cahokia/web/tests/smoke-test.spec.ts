import { expect, createSSGWorkerFixture } from './fixtures/ssg-server';

/**
 * Smoke test for Cahokia web application.
 * 
 * This is a minimal, fast test that verifies the most critical functionality:
 * - The app loads successfully
 * - The home page displays correctly
 * 
 * Uses SSG (Static Site Generation) approach which doesn't require a dev server.
 * The test builds a static bundle and serves it for testing.
 * 
 * Run this test quickly to verify basic functionality:
 *   npm test -- smoke-test.spec.ts
 */

// Create a test instance with SSG server fixture
const test = createSSGWorkerFixture({
  port: 8092, // Use a unique port to avoid conflicts
  bundleTimeout: 600000, // 10 minutes for bundle creation
});

test.describe('Cahokia Smoke Test', () => {
  // Override the base URL to use the SSG server
  test.use({ baseURL: 'http://localhost:8092' });

  test('should load and display home page', async ({ page }) => {
    // Navigate to the home page
    await page.goto('/');
    
    // Wait for the page to be fully loaded
    await page.waitForLoadState('networkidle');
    
    // Verify the page has a title
    await expect(page).toHaveTitle(/.+/);
    
    // Verify the main header with "Cahokia" title is visible
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Verify the Cahokia title text is present
    await expect(header.locator('h1')).toHaveText('Cahokia');
  });
});
