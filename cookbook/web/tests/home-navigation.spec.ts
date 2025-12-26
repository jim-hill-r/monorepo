import { test, expect } from '@playwright/test';

/**
 * Tests for home page navigation UI elements.
 * 
 * These tests verify that the home page has clickable UI elements
 * (buttons/cards) that allow users to navigate to recipes and plans.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Home Page Navigation UI', () => {
  test('should have a recipes navigation card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Look for a link or button that navigates to recipes
    // Using a flexible selector that works with different UI implementations
    const recipesLink = page.locator('a[href="/recipe/1"], button:has-text("Recipes"), .recipe-card a, .navigation-card a:has-text("Recipe")').first();
    await expect(recipesLink).toBeVisible();
  });

  test('should have a plans navigation card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Look for a link or button that navigates to plans
    const plansLink = page.locator('a[href="/plan/1"], button:has-text("Plans"), .plan-card a, .navigation-card a:has-text("Plan")').first();
    await expect(plansLink).toBeVisible();
  });

  test('should navigate to recipes when clicking the recipes card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Find and click the recipes navigation element
    const recipesLink = page.locator('a[href="/recipe/1"], .recipe-card a, .navigation-card a:has-text("Recipe")').first();
    await recipesLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we navigated to a recipe page
    await expect(page).toHaveURL(/\/recipe\/\d+/);
  });

  test('should navigate to plans when clicking the plans card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Find and click the plans navigation element
    const plansLink = page.locator('a[href="/plan/1"], .plan-card a, .navigation-card a:has-text("Plan")').first();
    await plansLink.click();
    await page.waitForLoadState('networkidle');
    
    // Verify we navigated to a plan page
    await expect(page).toHaveURL(/\/plan\/\d+/);
  });

  test('should have descriptive text for navigation cards', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that there's helpful text describing what recipes are
    const pageContent = await page.textContent('body');
    expect(pageContent).toMatch(/recipe|Recipe|daily/i);
    expect(pageContent).toMatch(/plan|Plan|weekly|meal/i);
  });
});
