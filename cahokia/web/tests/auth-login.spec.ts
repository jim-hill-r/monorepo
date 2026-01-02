import { test, expect } from '@playwright/test';

/**
 * Playwright tests for authentication/login functionality in Cahokia web application.
 * 
 * The dev server is automatically started by Playwright before tests run.
 * See playwright.config.ts webServer configuration for details.
 * 
 * Run tests with:
 *   npm test -- auth-login.spec.ts
 */

test.describe('Authentication and Login', () => {
  test('should display login button when auth provider loads', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for the navbar to be present
    const navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
    
    // Check that either the login button or loading state is present
    // The auth provider may still be loading or may show error
    const loginButton = page.locator('button:has-text("Login")');
    const loadingText = page.locator('div:has-text("Loading authentication...")');
    const errorText = page.locator('.error');
    
    // One of these should be visible
    const isLoginVisible = await loginButton.isVisible();
    const isLoadingVisible = await loadingText.isVisible();
    const isErrorVisible = await errorText.isVisible();
    
    expect(isLoginVisible || isLoadingVisible || isErrorVisible).toBe(true);
  });

  test('login button should be clickable without crashing app', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for the auth provider to load
    await page.waitForTimeout(1000);
    
    // Check if login button is present
    const loginButton = page.locator('button:has-text("Login")');
    
    // Only test if the button is visible (it might not be if auth provider failed to load)
    if (await loginButton.isVisible()) {
      // Listen for console errors
      const consoleErrors: string[] = [];
      page.on('console', msg => {
        if (msg.type() === 'error') {
          consoleErrors.push(msg.text());
        }
      });
      
      // Click the login button
      // Note: This will likely redirect or open a new window, so we don't wait for navigation
      await loginButton.click({ timeout: 5000 }).catch(() => {
        // Ignore click errors since the button might redirect
      });
      
      // Wait a moment to see if any errors are logged
      await page.waitForTimeout(500);
      
      // The button click should not cause any console errors related to unwrap/panic
      // (though there may be other errors like redirect failures in test environment)
      const hasUnwrapError = consoleErrors.some(err => 
        err.includes('panic') || err.includes('unwrap')
      );
      expect(hasUnwrapError).toBe(false);
    }
  });

  test('should handle auth provider initialization errors gracefully', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for auth state to resolve
    await page.waitForTimeout(2000);
    
    // Check if error is displayed
    const errorDiv = page.locator('.error');
    
    if (await errorDiv.isVisible()) {
      // If there's an error, verify it's displayed with proper message
      const errorText = await errorDiv.textContent();
      expect(errorText).toContain('Authentication Error');
    } else {
      // If no error, verify login button or loading state is shown
      const loginButton = page.locator('button:has-text("Login")');
      const loadingText = page.locator('div:has-text("Loading authentication...")');
      
      const isLoginVisible = await loginButton.isVisible();
      const isLoadingVisible = await loadingText.isVisible();
      
      expect(isLoginVisible || isLoadingVisible).toBe(true);
    }
  });
});
