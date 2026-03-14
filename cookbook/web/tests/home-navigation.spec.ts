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
    const recipesLink = page.locator('a[href^="/recipe"], button:has-text("Recipes"), .recipe-card a, .navigation-card a:has-text("Recipe")').first();
    // Ensure a recipes navigation target is present and visible
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
    await page.waitForLoadState('domcontentloaded');
    
    // Find and click the recipes card link - "Browse Recipes"
    const recipesLink = page.locator('.recipe-card a:has-text("Browse Recipes")');
    await recipesLink.waitFor({ state: 'visible' });
    // Scroll into view first to ensure it's accessible
    await recipesLink.scrollIntoViewIfNeeded();
    // Use force click to bypass overlapping elements
    await recipesLink.click({ force: true });
    await page.waitForLoadState('domcontentloaded');
    
    // Verify we navigated to a recipe page
    await expect(page).toHaveURL(/\/recipe\/\d+/);
  });

  test('should navigate to plans when clicking the plans card', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Find and click the plans card link - "View Meal Plans"
    const plansLink = page.locator('.plan-card a:has-text("View Meal Plans")');
    // Use force:true for mobile viewports where elements may overlap
    await plansLink.click({ force: true });
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
