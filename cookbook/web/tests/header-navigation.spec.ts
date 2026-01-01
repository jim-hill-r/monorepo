import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook header navigation bar.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Header Navigation', () => {
  test('should display header on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that header exists
    const header = page.locator('#header');
    await expect(header).toBeVisible();
  });

  test('should display Cookbook title in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for title in header - the h1 with class "header-title" is directly in the header
    const headerTitle = page.locator('#header h1.header-title');
    // Be tolerant: ensure title is visible and contains the expected word
    await expect(headerTitle).toBeVisible();
    await expect(headerTitle).toContainText('Cookbook');
  });

  test('should display navigation links in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for navigation links
    const nav = page.locator('#header .header-nav');
    await expect(nav).toBeVisible();
    
    // Check for Home link
    const homeLink = nav.locator('a:has-text("Home")');
    await expect(homeLink).toBeVisible();
    
    // Check for Recipes link
    const recipesLink = nav.locator('a:has-text("Recipes")');
    await expect(recipesLink).toBeVisible();
    
    // Check for Plans link
    const plansLink = nav.locator('a:has-text("Plans")');
    await expect(plansLink).toBeVisible();
  });

  test('should navigate to home from header', async ({ page }) => {
    // Start on a recipe page
    await page.goto('/recipe/10');
    await page.waitForLoadState('networkidle');
    
    // Click Home link in header
    const homeLink = page.locator('#header .header-nav a:has-text("Home")');
    await homeLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on home page by checking URL and content sections
    await expect(page).toHaveURL('/');
    // Home page has recipe and plan sections
    await expect(page.locator('#content')).toContainText('Daily Recipes');
    await expect(page.locator('#content')).toContainText('Weekly Meal Plans');
  });

  test('should navigate to recipes from header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click Recipes link in header
    const recipesLink = page.locator('#header .header-nav a:has-text("Recipes")');
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on a recipe page (check URL pattern)
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    // Recipe pages have the date heading
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should navigate to plans from header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click Plans link in header
    const plansLink = page.locator('#header .header-nav a:has-text("Plans")');
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on a plan page (check URL pattern)
    await expect(page).toHaveURL(/\/plan\/\d+/);
    // Plan pages have content visible
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should display header on all pages', async ({ page }) => {
    const pages = [
      '/',
      '/recipe/1',
      '/recipe/100',
      '/plan/1',
      '/plan/26'
    ];
    
    for (const path of pages) {
      await page.goto(path);
      await page.waitForLoadState('networkidle');
      
      // Verify header exists on each page
      const header = page.locator('#header');
      await expect(header).toBeVisible();
      
      // Verify title is present
      const headerTitle = page.locator('#header h1.header-title');
      await expect(headerTitle).toBeVisible();
      await expect(headerTitle).toContainText('Cookbook');
    }
  });

  test('should persist header during navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Navigate between pages and verify header is always present
    const recipesLink = page.locator('#header .header-nav a:has-text("Recipes")');
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    let header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const plansLink = page.locator('#header .header-nav a:has-text("Plans")');
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const homeLink = page.locator('#header .header-nav a:has-text("Home")');
    await homeLink.click();
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
  });
});
