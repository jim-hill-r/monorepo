import { test, expect } from '@playwright/test';
import { isPhone, mainNavLink, pageReady } from './nav';

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

test.describe('Header Navigation', { tag: ['@req:COOK-006', '@smoke'] }, () => {
  test('should display header on home page', async ({ page }) => {
    await page.goto('/');
    await pageReady(page);
    
    // Check that header exists
    const header = page.locator('#header');
    await expect(header).toBeVisible();
  });

  test('should display Cookbook title in header', async ({ page }) => {
    await page.goto('/');
    await pageReady(page);
    
    // Check for title in header - the h1 with class "header-title" is directly in the header
    const headerTitle = page.locator('#header h1.header-title');
    // Be tolerant: ensure title is visible and contains the expected word
    await expect(headerTitle).toBeVisible();
    await expect(headerTitle).toContainText('Cookbook');
  });

  test('should display navigation links in header', async ({ page }) => {
    await page.goto('/');

    // Phones show these links in the sidebar instead (mobile-header.spec.ts).
    const nav = page.locator('#header .header-nav');
    if (isPhone(page)) {
      await expect(nav).toBeHidden();
    } else {
      await expect(nav).toBeVisible();
    }

    for (const name of ['Home', 'Recipes', 'Plans']) {
      await expect(await mainNavLink(page, name)).toBeVisible();
    }
  });

  test('should navigate to home from header', async ({ page }) => {
    // Start on a recipe page
    await page.goto('/recipe/10');
    await pageReady(page);
    
    // Click Home link in header
    const homeLink = await mainNavLink(page, 'Home');
    await homeLink.click();
    
    // Verify we're on home page by checking URL and content sections
    await expect(page).toHaveURL('/');
    // Home page has recipe and plan sections
    await expect(page.locator('#content')).toContainText('Daily Recipes');
    await expect(page.locator('#content')).toContainText('Weekly Meal Plans');
  });

  test('should navigate to recipes from header', async ({ page }) => {
    await page.goto('/');
    await pageReady(page);
    
    // Click Recipes link in header
    const recipesLink = await mainNavLink(page, 'Recipes');
    await recipesLink.click();
    
    // Verify we're on a recipe page (check URL pattern)
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    // Recipe pages have the date heading
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should navigate to plans from header', async ({ page }) => {
    await page.goto('/');
    await pageReady(page);
    
    // Click Plans link in header
    const plansLink = await mainNavLink(page, 'Plans');
    await plansLink.click();
    
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
      // Use domcontentloaded instead of networkidle for faster, more reliable tests
      await page.waitForLoadState('domcontentloaded');
      
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
    await pageReady(page);
    
    // Navigate between pages and verify header is always present
    const recipesLink = await mainNavLink(page, 'Recipes');
    await recipesLink.click();
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    
    let header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const plansLink = await mainNavLink(page, 'Plans');
    await plansLink.click();
    await expect(page).toHaveURL(/\/plan\/\d+/);
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
    
    const homeLink = await mainNavLink(page, 'Home');
    await homeLink.click();
    await expect(page).toHaveURL('/');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
  });
});
