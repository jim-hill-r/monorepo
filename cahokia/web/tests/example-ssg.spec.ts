import { expect, createSSGWorkerFixture } from './fixtures/ssg-server';

/**
 * Example of using the SSG server fixture to run tests against static bundle.
 * 
 * This test file demonstrates how to use the SSG fixture to run existing tests
 * against the static assets built by `dx bundle --platform web --ssg` instead
 * of requiring a dev server.
 * 
 * The fixture:
 * - Builds the SSG bundle once per worker (efficient for multiple tests)
 * - Starts a static HTTP server
 * - Configures Playwright to use the static server's base URL
 * 
 * To run only these tests:
 *   npm test -- example-ssg.spec.ts
 * 
 * Prerequisites:
 * - Dioxus CLI (`dx`) must be installed: cargo install dioxus-cli
 */

// Create a test instance with SSG server fixture
// This will build the bundle and start the server once per worker
const test = createSSGWorkerFixture({
  port: 8091, // Use a different port to avoid conflicts with other tests
  bundleTimeout: 600000, // 10 minutes for bundle creation
});

test.describe('Cahokia Web Application (SSG Bundle)', () => {
  // Override the base URL to use the SSG server
  test.use({ baseURL: 'http://localhost:8091' });

  test('should load the home page', async ({ page }) => {
    // Navigate to the home page
    await page.goto('/');
    
    // Wait for the page to be fully loaded
    await page.waitForLoadState('networkidle');
    
    // Basic check that the page loaded
    await expect(page).toHaveTitle(/.+/);
  });

  test('should have a valid HTML structure', async ({ page }) => {
    await page.goto('/');
    
    // Check that basic HTML structure exists
    await expect(page.locator('html')).toBeVisible();
    await expect(page.locator('body')).toBeVisible();
  });

  test('should display header with Cahokia title', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the header exists
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Check that the Cahokia title is displayed
    const title = header.locator('h1');
    await expect(title).toHaveText('Cahokia');
  });

  test('should have navigation links in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that navigation links exist
    const nav = page.locator('.header-nav');
    await expect(nav).toBeVisible();
    
    // Check for all navigation links
    await expect(nav.locator('a:has-text("Home")')).toBeVisible();
    await expect(nav.locator('a:has-text("About")')).toBeVisible();
    await expect(nav.locator('a:has-text("History")')).toBeVisible();
    await expect(nav.locator('a:has-text("Explore")')).toBeVisible();
  });

  test('should navigate to About page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click the About link
    await page.locator('.header-nav a:has-text("About")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the about page
    expect(page.url()).toContain('/about');
    
    // Check that the About page content is displayed
    await expect(page.locator('h2:has-text("About Cahokia")')).toBeVisible();
  });

  test('should navigate to History page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click the History link
    await page.locator('.header-nav a:has-text("History")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the history page
    expect(page.url()).toContain('/history');
    
    // Check that the History page content is displayed
    await expect(page.locator('h2:has-text("History of Cahokia")')).toBeVisible();
  });

  test('should navigate to Explore page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click the Explore link
    await page.locator('.header-nav a:has-text("Explore")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the explore page
    expect(page.url()).toContain('/explore');
    
    // Check that the Explore page content is displayed
    await expect(page.locator('h2:has-text("Explore Cahokia")')).toBeVisible();
  });

  test('should navigate back to Home page from other pages', async ({ page }) => {
    await page.goto('/about');
    await page.waitForLoadState('networkidle');
    
    // Click the Home link
    await page.locator('.header-nav a:has-text("Home")').click();
    await page.waitForLoadState('networkidle');
    
    // Check that we're on the home page
    expect(page.url()).not.toContain('/about');
    
    // Check that the Home page content is displayed
    await expect(page.locator('h1:has-text("Cahokia")')).toBeVisible();
  });
});
