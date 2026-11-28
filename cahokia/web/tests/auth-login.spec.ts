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

const SIGN_IN = 'Sign in with jimhillr';

test.describe('Authentication and sign-in', () => {
  test('should display the jimhillr sign-in button', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for the navbar to be present
    const navbar = page.locator('#navbar');
    await expect(navbar).toBeVisible();
    
    // Check that either the login button or loading state is present
    // The auth provider may still be loading or may show error
    // The button shows while loading too (disabled), so it is always there.
    await expect(navbar.getByRole('button', { name: SIGN_IN })).toBeVisible();
  });

  test('sign-in button should be clickable without crashing app', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Wait for the auth provider to load
    await page.waitForTimeout(1000);
    
    // Check if login button is present
    const loginButton = page.getByRole('button', { name: SIGN_IN });
    
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
      // An error is a short message for people, with no technical detail.
      const errorText = (await errorDiv.textContent()) ?? '';
      expect(errorText.length).toBeLessThan(60);
      expect(errorText).not.toMatch(/https?:|oidc|discovery/i);
    }
    // The sign-in button stays, so the person can try again.
    await expect(page.getByRole('button', { name: SIGN_IN })).toBeVisible();
  });
});
