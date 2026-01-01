import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook sidebar mobile responsiveness.
 * 
 * These tests verify that the sidebar is closed by default on mobile devices
 * and can be opened/closed using the hamburger menu.
 */

test.describe('Sidebar Mobile Responsiveness', () => {
  // Configure these tests to run only on mobile viewports
  test.use({ 
    viewport: { width: 375, height: 667 } // iPhone SE size
  });

  test('should hide sidebar by default on mobile', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile, sidebar should be hidden by default
    const sidebar = page.locator('#sidebar');
    // Accept either hidden class or simply not visible via CSS
    await expect(sidebar).not.toBeVisible();
  });

  test('should show sidebar when hamburger is clicked on mobile', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Sidebar should be hidden initially
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).not.toBeVisible();
    
    // Click hamburger button to show sidebar
    const hamburger = page.locator('#header .hamburger-btn');
    await hamburger.click();
    await page.waitForTimeout(400); // Wait for animation
    
    // Sidebar should now be visible
    await expect(sidebar).toBeVisible();
  });

  test('should hide sidebar when hamburger is clicked again on mobile', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Initially hidden on mobile
    await expect(sidebar).not.toBeVisible();
    
    // Click to show
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(sidebar).toBeVisible();
    
    // Click to hide again
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(sidebar).not.toBeVisible();
  });

  test('hamburger button aria-expanded should be false initially on mobile', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const hamburger = page.locator('#header .hamburger-btn');
    
    // On mobile, should start as collapsed (false)
    await expect(hamburger).toHaveAttribute('aria-expanded', 'false');
  });

  test('should allow navigation with sidebar open on mobile', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Open sidebar
    const hamburger = page.locator('#header .hamburger-btn');
    await hamburger.click();
    await page.waitForTimeout(400);
    
    // Navigate using sidebar link
    const recipeLink = page.locator('#sidebar .sidebar-section').first().locator('a').first();
    await recipeLink.click();
    await page.waitForLoadState('networkidle');
    
    // Should navigate to recipe page
    await expect(page).toHaveURL(/\/recipe\/\d+/);
  });
});

test.describe('Sidebar Desktop Behavior', () => {
  // Configure these tests to run only on desktop viewports
  test.use({ 
    viewport: { width: 1280, height: 720 }
  });

  test('should show sidebar by default on desktop', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On desktop, sidebar should be visible by default
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).not.toHaveClass(/hidden/);
  });

  test('hamburger button aria-expanded should be true initially on desktop', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const hamburger = page.locator('#header .hamburger-btn');
    
    // On desktop, should start as expanded (true)
    await expect(hamburger).toHaveAttribute('aria-expanded', 'true');
  });
});
