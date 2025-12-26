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
    
    // Check for title in header
    const headerTitle = page.locator('#header .header-title h1');
    await expect(headerTitle).toHaveText('Cookbook');
  });

  test('should display navigation links in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for navigation links
    const nav = page.locator('#header .header-nav');
    await expect(nav).toBeVisible();
    
    // Check for Home link
    const homeLink = nav.locator('a', { hasText: 'Home' });
    await expect(homeLink).toBeVisible();
    
    // Check for Recipes link
    const recipesLink = nav.locator('a', { hasText: 'Recipes' });
    await expect(recipesLink).toBeVisible();
    
    // Check for Plans link
    const plansLink = nav.locator('a', { hasText: 'Plans' });
    await expect(plansLink).toBeVisible();
  });

  test('should navigate to home from header', async ({ page }) => {
    // Start on a recipe page
    await page.goto('/recipe/10');
    await page.waitForLoadState('networkidle');
    
    // Click Home link in header
    const homeLink = page.locator('#header .header-nav a', { hasText: 'Home' });
    await homeLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on home page
    await expect(page.locator('h1').first()).toHaveText('Cookbook');
    await expect(page.locator('p').first()).toContainText('Welcome to the Cookbook application!');
  });

  test('should navigate to recipes from header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click Recipes link in header
    const recipesLink = page.locator('#header .header-nav a', { hasText: 'Recipes' });
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on recipe page (should go to recipe 1 as a default)
    await expect(page.locator('h1')).toHaveText('Recipe for Day 1');
  });

  test('should navigate to plans from header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click Plans link in header
    const plansLink = page.locator('#header .header-nav a', { hasText: 'Plans' });
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we're on plan page (should go to plan 1 as a default)
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 1');
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
      const headerTitle = page.locator('#header .header-title h1');
      await expect(headerTitle).toHaveText('Cookbook');
    }
  });

  test('should persist header during navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Navigate between pages and verify header is always present
    const recipesLink = page.locator('#header .header-nav a', { hasText: 'Recipes' });
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    let header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const plansLink = page.locator('#header .header-nav a', { hasText: 'Plans' });
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const homeLink = page.locator('#header .header-nav a', { hasText: 'Home' });
    await homeLink.click();
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
  });
});
