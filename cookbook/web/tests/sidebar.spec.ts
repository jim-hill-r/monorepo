import { test, expect } from '@playwright/test';

/**
 * Tests for the Cookbook sidebar navigation.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cast run
 * 
 * Run tests with:
 *   npm test
 */

test.describe('Sidebar Navigation', () => {
  test('should display sidebar on home page', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that sidebar exists
    const sidebar = page.locator('#sidebar');
    
    // On mobile viewports (width <= 768), sidebar is hidden by default
    if (viewport && viewport.width <= 768) {
      await expect(sidebar).toHaveClass(/hidden/);
    } else {
      await expect(sidebar).toBeVisible();
    }
  });

  test('should display sidebar title', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile viewports, sidebar is hidden so title won't be visible
    const sidebar = page.locator('#sidebar');
    if (viewport && viewport.width <= 768) {
      // Just verify sidebar exists even if hidden
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    
    // Check for sidebar title
    const sidebarTitle = page.locator('#sidebar h2');
    // Be tolerant: ensure the sidebar title is visible and contains expected text
    await expect(sidebarTitle).toBeVisible();
    await expect(sidebarTitle).toContainText('Quick Navigation');
  });

  test('should display recipe quick links section', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for recipes section in sidebar
    const recipesSection = page.locator('#sidebar .sidebar-section').first();
    await expect(recipesSection.locator('h3')).toHaveText('Daily Recipes');
    
    // Check that there are recipe links (should be 7 links for current week, or fewer near year end)
    const recipeLinks = recipesSection.locator('a');
    const linkCount = await recipeLinks.count();
    expect(linkCount).toBeGreaterThanOrEqual(1);
    expect(linkCount).toBeLessThanOrEqual(7);
    
    // Verify the links have proper date format (e.g., "Thu, 1-Jan")
    const firstLink = recipeLinks.first();
    const firstLinkText = await firstLink.textContent();
    expect(firstLinkText).toMatch(/\w{3}, \d{1,2}-\w{3}/); // Format: "Day, DD-Mon"
  });

  test('should display plan quick links section', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check for plans section in sidebar
    const plansSection = page.locator('#sidebar .sidebar-section').last();
    await expect(plansSection.locator('h3')).toHaveText('Weekly Plans');
    
    // Check that there are plan links (should be 4 links)
    const planLinks = plansSection.locator('a');
    await expect(planLinks).toHaveCount(4);
    
    // Verify the links have proper week format
    const firstLink = planLinks.first();
    await expect(firstLink).toContainText('Week');
  });

  test('should navigate to recipe from sidebar link', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile, sidebar is hidden by default, so we need to show it first or skip the click
    if (viewport && viewport.width <= 768) {
      // On mobile, the sidebar is hidden, so verify it exists but don't test navigation
      const sidebar = page.locator('#sidebar');
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    
    // Click on the first recipe link in sidebar
    const recipeLink = page.locator('#sidebar .sidebar-section').first().locator('a').first();
    await recipeLink.click();
    await page.waitForLoadState('networkidle');
    
    // Should navigate to a recipe page (check URL pattern)
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    
    // Should have a recipe title (h1 in content area, not header)
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toBeVisible();
    await expect(contentH1).not.toBeEmpty();
  });

  test('should navigate to plan from sidebar link', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile, sidebar is hidden by default
    if (viewport && viewport.width <= 768) {
      const sidebar = page.locator('#sidebar');
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    
    // Click on the first plan link in sidebar
    const planLink = page.locator('#sidebar .sidebar-section').last().locator('a').first();
    const planText = await planLink.textContent();
    const weekNumber = planText?.match(/Week (\d+)/)?.[1];
    
    await planLink.click();
    await page.waitForLoadState('networkidle');
    
    // Should navigate to a plan page (check URL pattern)
    await expect(page).toHaveURL(/\/plan\/\d+/);
    
    // Should have a plan title matching the week number
    const contentH1 = page.locator('#content h1');
    await expect(contentH1).toHaveText(`Meal Plan for Week ${weekNumber}`);
  });

  test('should display sidebar on recipe pages', async ({ page, viewport }) => {
    await page.goto('/recipe/50');
    await page.waitForLoadState('networkidle');
    
    // Verify sidebar exists on recipe page
    const sidebar = page.locator('#sidebar');
    
    // On mobile, sidebar is hidden by default
    if (viewport && viewport.width <= 768) {
      await expect(sidebar).toHaveClass(/hidden/);
    } else {
      await expect(sidebar).toBeVisible();
    }
  });

  test('should display sidebar on plan pages', async ({ page, viewport }) => {
    await page.goto('/plan/25');
    await page.waitForLoadState('networkidle');
    
    // Verify sidebar exists on plan page
    const sidebar = page.locator('#sidebar');
    
    // On mobile, sidebar is hidden by default
    if (viewport && viewport.width <= 768) {
      await expect(sidebar).toHaveClass(/hidden/);
    } else {
      await expect(sidebar).toBeVisible();
    }
  });

  test('should allow navigation between multiple recipe links', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile, sidebar is hidden and links aren't clickable
    if (viewport && viewport.width <= 768) {
      const sidebar = page.locator('#sidebar');
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Get recipe links from sidebar
    const recipeSection = page.locator('#sidebar .sidebar-section').first();
    const firstLink = recipeSection.locator('a').first();
    const secondLink = recipeSection.locator('a').nth(1);
    
    // Click first link
    await firstLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    const firstUrl = page.url();
    
    // Click second link from sidebar
    await secondLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/recipe\/\d+/);
    const secondUrl = page.url();
    
    // URLs should be different
    expect(firstUrl).not.toBe(secondUrl);
  });

  test('should allow navigation between multiple plan links', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // On mobile, sidebar is hidden and links aren't clickable
    if (viewport && viewport.width <= 768) {
      const sidebar = page.locator('#sidebar');
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
        
    // Get plan links from sidebar
    const planSection = page.locator('#sidebar .sidebar-section').last();
    const firstLink = planSection.locator('a').first();
    const secondLink = planSection.locator('a').nth(1);
    
    const firstLinkText = await firstLink.textContent();
    const firstWeek = firstLinkText?.match(/Week (\d+)/)?.[1];
    
    // Click first link
    await firstLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/plan\/\d+/);
    await expect(page.locator('#content h1')).toHaveText(`Meal Plan for Week ${firstWeek}`);
    
    // Click second link from sidebar
    const secondLinkText = await secondLink.textContent();
    const secondWeek = secondLinkText?.match(/Week (\d+)/)?.[1];
    
    await secondLink.click();
    await page.waitForLoadState('networkidle');
    await expect(page).toHaveURL(/\/plan\/\d+/);
    await expect(page.locator('#content h1')).toHaveText(`Meal Plan for Week ${secondWeek}`);
  });
});
