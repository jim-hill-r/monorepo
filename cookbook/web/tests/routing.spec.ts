import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook web application routing.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cookbook/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Cookbook Web Application', () => {
  test('should load the home page', async ({ page }) => {
    // Navigate to the home page
    await page.goto('/');
    
    // Wait for the page to be fully loaded
    await page.waitForLoadState('networkidle');
    
    // Check that the page loaded with Cookbook title
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });

  test('should have welcome text on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for welcome message
    await expect(page.locator('p').first()).toContainText('Welcome to the Cookbook application!');
  });

  test('should have navigation information on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for daily recipes section
    await expect(page.locator('h2').first()).toContainText('Daily Recipes');
    
    // Check for weekly plans section
    await expect(page.locator('h2').last()).toContainText('Weekly Plans');
  });
});

test.describe('Recipe Routes', () => {
  test('should load recipe for day 1', async ({ page }) => {
    await page.goto('/recipe/1');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded
    await expect(page.locator('h1')).toHaveText('Recipe for Day 1');
    await expect(page.locator('p')).toContainText('placeholder recipe for day 1');
  });

  test('should load recipe for day 100', async ({ page }) => {
    await page.goto('/recipe/100');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded
    await expect(page.locator('h1')).toHaveText('Recipe for Day 100');
    await expect(page.locator('p')).toContainText('placeholder recipe for day 100');
  });

  test('should load recipe for day 365', async ({ page }) => {
    await page.goto('/recipe/365');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded
    await expect(page.locator('h1')).toHaveText('Recipe for Day 365');
    await expect(page.locator('p')).toContainText('placeholder recipe for day 365');
  });

  test('should have back to home link on recipe page', async ({ page }) => {
    await page.goto('/recipe/50');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a', { hasText: 'Back to Home' });
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });

  test('should load multiple recipe days correctly', async ({ page }) => {
    // Test a few different days to ensure routing works consistently
    const days = [7, 42, 180, 250];
    
    for (const day of days) {
      await page.goto(`/recipe/${day}`);
      await page.waitForLoadState('networkidle');
      await expect(page.locator('h1')).toHaveText(`Recipe for Day ${day}`);
    }
  });
});

test.describe('Plan Routes', () => {
  test('should load plan for week 1', async ({ page }) => {
    await page.goto('/plan/1');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 1');
    await expect(page.locator('p')).toContainText('placeholder meal plan for week 1');
  });

  test('should load plan for week 26', async ({ page }) => {
    await page.goto('/plan/26');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 26');
    await expect(page.locator('p')).toContainText('placeholder meal plan for week 26');
  });

  test('should load plan for week 52', async ({ page }) => {
    await page.goto('/plan/52');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    await expect(page.locator('h1')).toHaveText('Meal Plan for Week 52');
    await expect(page.locator('p')).toContainText('placeholder meal plan for week 52');
  });

  test('should have back to home link on plan page', async ({ page }) => {
    await page.goto('/plan/12');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a', { hasText: 'Back to Home' });
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });

  test('should load multiple plan weeks correctly', async ({ page }) => {
    // Test a few different weeks to ensure routing works consistently
    const weeks = [5, 13, 30, 45];
    
    for (const week of weeks) {
      await page.goto(`/plan/${week}`);
      await page.waitForLoadState('networkidle');
      await expect(page.locator('h1')).toHaveText(`Meal Plan for Week ${week}`);
    }
  });
});

test.describe('404 Page', () => {
  test('should show 404 page for invalid routes', async ({ page }) => {
    await page.goto('/invalid-route');
    await page.waitForLoadState('networkidle');
    
    // Check for 404 page
    await expect(page.locator('h1')).toHaveText('Page not found');
    await expect(page.locator('p')).toContainText('terribly sorry');
  });

  test('should have back to home link on 404 page', async ({ page }) => {
    await page.goto('/non-existent');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a', { hasText: 'Back to Home' });
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });
});

test.describe('Input Validation', () => {
  test('should show error for day 0', async ({ page }) => {
    await page.goto('/recipe/0');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    await expect(page.locator('h1')).toHaveText('Invalid Day');
    await expect(page.locator('p')).toContainText('Day 0 is not valid');
    await expect(page.locator('p')).toContainText('between 1 and 365');
  });

  test('should show error for day 366', async ({ page }) => {
    await page.goto('/recipe/366');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    await expect(page.locator('h1')).toHaveText('Invalid Day');
    await expect(page.locator('p')).toContainText('Day 366 is not valid');
    await expect(page.locator('p')).toContainText('between 1 and 365');
  });

  test('should show error for day 999', async ({ page }) => {
    await page.goto('/recipe/999');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    await expect(page.locator('h1')).toHaveText('Invalid Day');
    await expect(page.locator('p')).toContainText('Day 999 is not valid');
  });

  test('should show error for week 0', async ({ page }) => {
    await page.goto('/plan/0');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    await expect(page.locator('h1')).toHaveText('Invalid Week');
    await expect(page.locator('p')).toContainText('Week 0 is not valid');
    await expect(page.locator('p')).toContainText('between 1 and 52');
  });

  test('should show error for week 53', async ({ page }) => {
    await page.goto('/plan/53');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    await expect(page.locator('h1')).toHaveText('Invalid Week');
    await expect(page.locator('p')).toContainText('Week 53 is not valid');
    await expect(page.locator('p')).toContainText('between 1 and 52');
  });

  test('should show error for week 100', async ({ page }) => {
    await page.goto('/plan/100');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    await expect(page.locator('h1')).toHaveText('Invalid Week');
    await expect(page.locator('p')).toContainText('Week 100 is not valid');
  });

  test('should have back to home link on invalid day page', async ({ page }) => {
    await page.goto('/recipe/500');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a', { hasText: 'Back to Home' });
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });

  test('should have back to home link on invalid week page', async ({ page }) => {
    await page.goto('/plan/75');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a', { hasText: 'Back to Home' });
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page.locator('h1')).toHaveText('Cookbook');
  });
});
