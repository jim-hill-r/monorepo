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
    
    // Check that the home page loaded with the home title
    await expect(page).toHaveURL('/');
    const contentH1 = page.locator('.home-container h1');
    // Ensure the page heading is visible and contains the full title
    await expect(contentH1).toBeVisible();
    await expect(contentH1).toHaveText("The Engineer's 365 Cookbook");
  });
  
  test('should have welcome text on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for intro description in the content area
    await expect(page.locator('.intro-description')).toBeVisible();
  });

  test('should have navigation information on home page', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for navigation cards with Daily Recipes and Weekly Plans
    await expect(page.locator('#content h2').first()).toContainText('Daily Recipes');
    await expect(page.locator('#content h2').last()).toContainText('Weekly Meal Plans');
  });
});

test.describe('Recipe Routes', () => {
  test('should load recipe for day 1', async ({ page }) => {
    await page.goto('/recipe/1');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded with actual recipe content
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toBeVisible();
    await expect(contentH1).not.toBeEmpty();
    
    // Should have ingredients section
    await expect(page.locator('h2', { hasText: 'Ingredients' })).toBeVisible();
  });

  test('should load recipe for day 100', async ({ page }) => {
    await page.goto('/recipe/100');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded with actual recipe content
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toBeVisible();
    await expect(contentH1).not.toBeEmpty();
    
    // Should have ingredients section
    await expect(page.locator('h2', { hasText: 'Ingredients' })).toBeVisible();
  });

  test('should load recipe for day 365', async ({ page }) => {
    await page.goto('/recipe/365');
    await page.waitForLoadState('networkidle');
    
    // Check that the recipe page loaded with actual recipe content
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toBeVisible();
    await expect(contentH1).not.toBeEmpty();
    
    // Should have ingredients section
    await expect(page.locator('h2', { hasText: 'Ingredients' })).toBeVisible();
  });

  test('should have back to home link on recipe page', async ({ page }) => {
    await page.goto('/recipe/50');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a:has-text("Back to Home")');
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    // Use force:true for mobile viewports where elements may overlap
    await backLink.click({ force: true });
    await page.waitForLoadState('networkidle');
    // Check we're on home page by looking for the home page specific content
    await expect(page).toHaveURL('/');
    await expect(page.locator('.home-container h1')).toHaveText("The Engineer's 365 Cookbook");
  });

  test('should load multiple recipe days correctly', async ({ page }) => {
    // Test a few different days to ensure routing works consistently
    const days = [7, 42, 180, 250];
    
    for (const day of days) {
      await page.goto(`/recipe/${day}`);
      // Use domcontentloaded instead of networkidle for faster, more reliable tests
      await page.waitForLoadState('domcontentloaded');
      // Verify we're on the correct recipe page by checking URL
      await expect(page).toHaveURL(`/recipe/${day}`);
      // Verify recipe content is displayed
      const contentH1 = page.locator('#content h1');
      await expect(contentH1).toBeVisible();
    }
  });
});

test.describe('Plan Routes', () => {
  test('should load plan for week 1', async ({ page }) => {
    await page.goto('/plan/1');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Meal Plan for Week 1');
    // Just verify page content is present (shopping list or recipes)
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should load plan for week 26', async ({ page }) => {
    await page.goto('/plan/26');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Meal Plan for Week 26');
    // Just verify page content is present
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should load plan for week 52', async ({ page }) => {
    await page.goto('/plan/52');
    await page.waitForLoadState('networkidle');
    
    // Check that the plan page loaded
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Meal Plan for Week 52');
    // Just verify page content is present
    await expect(page.locator('#content')).toBeVisible();
  });

  test('should have back to home link on plan page', async ({ page }) => {
    await page.goto('/plan/12');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a:has-text("Back to Home")');
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    // Use force:true for mobile viewports where elements may overlap
    await backLink.click({ force: true });
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL('/');
    await expect(page.locator('.home-container h1')).toHaveText("The Engineer's 365 Cookbook");
  });

  test('should load multiple plan weeks correctly', async ({ page }) => {
    // Test a few different weeks to ensure routing works consistently
    const weeks = [5, 13, 30, 45];
    
    for (const week of weeks) {
      await page.goto(`/plan/${week}`);
      // Use domcontentloaded instead of networkidle for faster, more reliable tests
      await page.waitForLoadState('domcontentloaded');
      const contentH1 = page.locator('#content h1');
      await expect(contentH1).toHaveText(`Meal Plan for Week ${week}`);
    }
  });
});

test.describe('404 Page', () => {
  test('should show 404 page for invalid routes', async ({ page }) => {
    await page.goto('/invalid-route');
    await page.waitForLoadState('networkidle');
    
    // Check for 404 page
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Page not found');
    await expect(page.locator('#content p')).toContainText('terribly sorry');
  });

  test('should have back to home link on 404 page', async ({ page }) => {
    await page.goto('/non-existent');
    await page.waitForLoadState('networkidle');
    
    // Check for back to home link
    const backLink = page.locator('a:has-text("Back to Home")');
    await expect(backLink).toBeVisible();
    
    // Click the link and verify navigation
    await backLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL('/');
    await expect(page.locator('.home-container h1')).toHaveText("The Engineer's 365 Cookbook");
  });
});

test.describe('Input Validation', () => {
  test('should show error for day 0', async ({ page }) => {
    await page.goto('/recipe/0');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Day');
    await expect(page.locator('#content p')).toContainText('Day 0 is not valid');
    await expect(page.locator('#content p')).toContainText('between 1 and 365');
  });

  test('should show error for day 366', async ({ page }) => {
    await page.goto('/recipe/366');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Day');
    await expect(page.locator('#content p')).toContainText('Day 366 is not valid');
    await expect(page.locator('#content p')).toContainText('between 1 and 365');
  });

  test('should show error for day 999', async ({ page }) => {
    await page.goto('/recipe/999');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid day error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Day');
    await expect(page.locator('#content p')).toContainText('Day 999 is not valid');
  });

  test('should show error for week 0', async ({ page }) => {
    await page.goto('/plan/0');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Week');
    await expect(page.locator('#content p')).toContainText('Week 0 is not valid');
    await expect(page.locator('#content p')).toContainText('between 1 and 53');
  });

  test('should show error for week 54', async ({ page }) => {
    await page.goto('/plan/54');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Week');
    await expect(page.locator('#content p')).toContainText('Week 54 is not valid');
    await expect(page.locator('#content p')).toContainText('between 1 and 53');
  });

  test('should show error for week 100', async ({ page }) => {
    await page.goto('/plan/100');
    await page.waitForLoadState('networkidle');
    
    // Check for invalid week error
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText('Invalid Week');
    await expect(page.locator('#content p')).toContainText('Week 100 is not valid');
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
    await expect(page).toHaveURL('/');
    await expect(page.locator('#content h1')).toHaveText("The Engineer's 365 Cookbook");
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
    await expect(page).toHaveURL('/');
    await expect(page.locator('#content h1')).toHaveText("The Engineer's 365 Cookbook");
  });
});
