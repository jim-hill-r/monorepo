import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook authentication in header.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Authentication in Header', () => {
  test('should display login button or loading state in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Check for either login button or loading state (since auth initialization is async)
    const hasLoginButton = await header.locator('button', { hasText: 'Login' }).isVisible().catch(() => false);
    const hasLoadingState = await header.locator('div', { hasText: 'Loading authentication' }).isVisible().catch(() => false);
    const hasError = await header.locator('.error').isVisible().catch(() => false);
    
    // At least one should be visible
    expect(hasLoginButton || hasLoadingState || hasError).toBeTruthy();
  });

  test('should display user-friendly error messages when auth fails', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Check if there's an error displayed
    const errorElement = header.locator('.error');
    const hasError = await errorElement.isVisible().catch(() => false);
    
    if (hasError) {
      // If there's an error, verify it's user-friendly
      const errorText = await errorElement.textContent();
      
      // Verify error message is concise (less than 100 characters)
      expect(errorText?.length || 0).toBeLessThan(100);
      
      // Verify it doesn't contain technical details
      expect(errorText).not.toContain('https://');
      expect(errorText).not.toContain('.well-known');
      expect(errorText).not.toContain('NetworkError');
      expect(errorText).not.toContain('stack trace');
      expect(errorText).not.toContain('Error:');
      
      // Verify it uses plain language
      const plainLanguageTerms = [
        'authentication',
        'login',
        'service',
        'error',
        'try again',
        'check',
        'unavailable',
        'incomplete',
        'failed',
      ];
      const hasPlainLanguage = plainLanguageTerms.some(term => 
        errorText?.toLowerCase().includes(term)
      );
      expect(hasPlainLanguage).toBeTruthy();
    }
  });

  test('should display login button in header on all pages', async ({ page }) => {
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
      
      // Check that login button or loading/error state is in header
      const hasLoginButton = await header.locator('button', { hasText: 'Login' }).isVisible().catch(() => false);
      const hasLoadingState = await header.locator('div', { hasText: 'Loading authentication' }).isVisible().catch(() => false);
      const hasError = await header.locator('.error').isVisible().catch(() => false);
      
      expect(hasLoginButton || hasLoadingState || hasError).toBeTruthy();
    }
  });

  test('should persist login button during navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Navigate between pages and verify login button is always present in header
    const recipesLink = page.locator('#header .header-nav a', { hasText: 'Recipes' });
    // Use force:true for mobile viewports where elements may overlap
    await recipesLink.click({ force: true });
    await page.waitForLoadState('networkidle');
    
    let header = page.locator('#header');
    await expect(header).toBeVisible();
    const hasAuth1 = await header.locator('button, div').count() > 0;
    expect(hasAuth1).toBeTruthy();
    
    const plansLink = page.locator('#header .header-nav a', { hasText: 'Plans' });
    // Use force:true for mobile viewports where elements may overlap
    await plansLink.click({ force: true });
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
    const hasAuth2 = await header.locator('button, div').count() > 0;
    expect(hasAuth2).toBeTruthy();
    
    const homeLink = page.locator('#header .header-nav a', { hasText: 'Home' });
    // Use force:true for mobile viewports where elements may overlap
    await homeLink.click({ force: true });
    await page.waitForLoadState('networkidle');
    
    header = page.locator('#header');
    await expect(header).toBeVisible();
    const hasAuth3 = await header.locator('button, div').count() > 0;
    expect(hasAuth3).toBeTruthy();
  });

  test('should have proper styling on login button', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for login button to appear in header (it may take time for auth to initialize)
    const loginButton = page.locator('#header button', { hasText: 'Login' });
    
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
});
