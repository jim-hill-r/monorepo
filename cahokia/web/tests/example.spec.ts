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
