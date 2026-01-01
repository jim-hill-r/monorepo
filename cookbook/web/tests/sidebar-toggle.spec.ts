import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook sidebar toggle functionality.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Sidebar Toggle', () => {
  // Configure viewport for consistent desktop testing
  test.use({ 
    viewport: { width: 1280, height: 720 }
  });

  test('should display hamburger button in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that hamburger button exists in header
    const hamburger = page.locator('#header .hamburger-btn');
    await expect(hamburger).toBeVisible();
  });

  test('should display sidebar by default on desktop', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that sidebar is visible initially on desktop
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
    await expect(sidebar).not.toHaveClass(/hidden/);
  });

  test('should hide sidebar when hamburger is clicked', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Sidebar should be visible initially
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
    
    // Click hamburger button
    const hamburger = page.locator('#header .hamburger-btn');
    await hamburger.click();
    
    // Wait for animation to complete
    await page.waitForTimeout(400);
    
    // Sidebar should be hidden
    await expect(sidebar).toHaveClass(/hidden/);
  });

  test('should show sidebar when hamburger is clicked again', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Click to hide
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(sidebar).toHaveClass(/hidden/);
    
    // Click to show
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(sidebar).not.toHaveClass(/hidden/);
  });

  test('should adjust content margin when sidebar is hidden', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const content = page.locator('#content');
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Get initial margin
    const initialMargin = await content.evaluate((el) => {
      return window.getComputedStyle(el).marginLeft;
    });
    
    // Click to hide sidebar
    await hamburger.click();
    await page.waitForTimeout(400);
    
    // Get new margin (should be smaller or 0)
    const newMargin = await content.evaluate((el) => {
      return window.getComputedStyle(el).marginLeft;
    });
    
    // Margin should change when sidebar is hidden
    expect(newMargin).not.toBe(initialMargin);
  });

  test('should persist sidebar state across navigation', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Hide sidebar
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(sidebar).toHaveClass(/hidden/);
    
    // Navigate to another page
    await page.locator('#sidebar a', { hasText: 'Day 1' }).click();
    await page.waitForLoadState('networkidle');
    
    // Sidebar should still be hidden
    await expect(sidebar).toHaveClass(/hidden/);
  });

  test('hamburger button should be accessible', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Check that button has aria-label
    await expect(hamburger).toHaveAttribute('aria-label');
    
    // Check that button has aria-expanded attribute
    await expect(hamburger).toHaveAttribute('aria-expanded');
  });

  test('hamburger button aria-expanded should toggle', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const hamburger = page.locator('#header .hamburger-btn');
    
    // Initially should be expanded
    await expect(hamburger).toHaveAttribute('aria-expanded', 'true');
    
    // Click to hide
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(hamburger).toHaveAttribute('aria-expanded', 'false');
    
    // Click to show
    await hamburger.click();
    await page.waitForTimeout(400);
    await expect(hamburger).toHaveAttribute('aria-expanded', 'true');
  });
});
