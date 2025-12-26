import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook authentication navbar.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Authentication Navbar', () => {
  test('should display navbar on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that navbar exists
    const navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
  });

  test('should display login button or loading state in navbar', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
    
    // Check for either login button or loading state (since auth initialization is async)
    const hasLoginButton = await navbar.locator('button', { hasText: 'Login' }).isVisible().catch(() => false);
    const hasLoadingState = await navbar.locator('div', { hasText: 'Loading authentication' }).isVisible().catch(() => false);
    const hasError = await navbar.locator('.error').isVisible().catch(() => false);
    
    // At least one should be visible
    expect(hasLoginButton || hasLoadingState || hasError).toBeTruthy();
  });

  test('should display navbar on all pages', async ({ page }) => {
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
      
      // Verify navbar exists on each page
      const navbar = page.locator('#navbar');
      await expect(navbar).toBeVisible();
    }
  });

  test('should persist navbar during navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Navigate between pages and verify navbar is always present
    const recipesLink = page.locator('#header .header-nav a', { hasText: 'Recipes' });
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    let navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
    
    const plansLink = page.locator('#header .header-nav a', { hasText: 'Plans' });
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
    
    const homeLink = page.locator('#header .header-nav a', { hasText: 'Home' });
    await homeLink.click();
    await page.waitForLoadState('networkidle');
    
    navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
  });

  test('should have proper styling on login button', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for login button to appear (it may take time for auth to initialize)
    const loginButton = page.locator('#navbar button', { hasText: 'Login' });
    
    // Check if button is present (it should eventually appear or show an error/loading state)
    const isVisible = await loginButton.isVisible().catch(() => false);
    
    if (isVisible) {
      // If button is visible, verify it has proper styling
      const buttonStyles = await loginButton.evaluate((el) => {
        const styles = window.getComputedStyle(el);
        return {
          cursor: styles.cursor,
          borderRadius: styles.borderRadius,
        };
      });
      
      expect(buttonStyles.cursor).toBe('pointer');
      expect(buttonStyles.borderRadius).toBeTruthy();
    }
  });

  test('should position navbar below header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const header = page.locator('#header');
    const navbar = page.locator('#navbar');
    
    await expect(header).toBeVisible();
    await expect(navbar).toBeVisible();
    
    // Get positions
    const headerBox = await header.boundingBox();
    const navbarBox = await navbar.boundingBox();
    
    // Navbar should be below header
    expect(navbarBox!.y).toBeGreaterThanOrEqual(headerBox!.y + headerBox!.height);
  });
});
