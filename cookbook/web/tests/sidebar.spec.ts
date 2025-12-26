import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook sidebar navigation.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Sidebar Navigation', () => {
  test('should display sidebar on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that sidebar exists
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
  });

  test('should display sidebar title', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for sidebar title
    const sidebarTitle = page.locator('#sidebar h2');
    await expect(sidebarTitle).toHaveText('Quick Navigation');
  });

  test('should display recipe quick links section', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for recipes section in sidebar
    const recipesSection = page.locator('#sidebar .sidebar-section').first();
    await expect(recipesSection.locator('h3')).toHaveText('Daily Recipes');
    
    // Check for some recipe range links
    await expect(recipesSection.locator('a', { hasText: 'Days 1-10' })).toBeVisible();
    await expect(recipesSection.locator('a', { hasText: 'Days 11-20' })).toBeVisible();
  });

  test('should display plan quick links section', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for plans section in sidebar
    const plansSection = page.locator('#sidebar .sidebar-section').last();
    await expect(plansSection.locator('h3')).toHaveText('Weekly Plans');
    
    // Check for some plan range links
    await expect(plansSection.locator('a', { hasText: 'Weeks 1-4' })).toBeVisible();
    await expect(plansSection.locator('a', { hasText: 'Weeks 5-8' })).toBeVisible();
  });

  test('should navigate to recipe from sidebar link', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click on a recipe range link
    const recipeLink = page.locator('#sidebar a', { hasText: 'Days 1-10' });
    await recipeLink.click();
    await page.waitForLoadState('networkidle');
    
    // Should navigate to first day in range
    await expect(page.locator('h1')).toHaveText('Recipe for Day 1');
  });

  test('should navigate to plan from sidebar link', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click on a plan range link
    const planLink = page.locator('#sidebar a', { hasText: 'Weeks 1-4' });
    await planLink.click();
    await page.waitForLoadState('networkidle');
    
    // Should navigate to first week in range
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 1');
  });

  test('should display sidebar on recipe pages', async ({ page }) => {
    await page.goto('/recipe/50');
    await page.waitForLoadState('networkidle');
    
    // Verify sidebar exists on recipe page
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
  });

  test('should display sidebar on plan pages', async ({ page }) => {
    await page.goto('/plan/25');
    await page.waitForLoadState('networkidle');
    
    // Verify sidebar exists on plan page
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
  });

  test('should allow navigation between multiple recipe ranges', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click first range
    await page.locator('#sidebar a', { hasText: 'Days 1-10' }).click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Recipe for Day 1');
    
    // Click another range from sidebar
    await page.locator('#sidebar a', { hasText: 'Days 11-20' }).click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Recipe for Day 11');
  });

  test('should allow navigation between multiple plan ranges', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Click first range
    await page.locator('#sidebar a', { hasText: 'Weeks 1-4' }).click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 1');
    
    // Click another range from sidebar
    await page.locator('#sidebar a', { hasText: 'Weeks 5-8' }).click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 5');
  });
});
