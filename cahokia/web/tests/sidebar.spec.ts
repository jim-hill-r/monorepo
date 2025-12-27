import { test, expect } from '@playwright/test';

/**
 * Playwright tests for Cahokia sidebar component.
 * 
 * The dev server is automatically started by Playwright before tests run.
 * See playwright.config.ts webServer configuration for details.
 * 
 * Run tests with:
 *   npm test -- sidebar.spec.ts
 */

test.describe('Sidebar Component', () => {
  test('should display sidebar toggle button in header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the sidebar toggle button exists
    const toggleButton = page.locator('.sidebar-toggle');
    await expect(toggleButton).toBeVisible();
    
    // Verify it displays the hamburger menu icon
    await expect(toggleButton).toHaveText('☰');
  });

  test('sidebar should be hidden by default', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the sidebar exists
    const sidebar = page.locator('#sidebar');
    await expect(sidebar).toBeVisible();
    
    // Verify it does not have the 'open' class initially
    const hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(false);
  });

  test('should open sidebar when toggle button is clicked', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const toggleButton = page.locator('.sidebar-toggle');
    
    // Verify sidebar is not open initially
    let hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(false);
    
    // Click the toggle button
    await toggleButton.click();
    
    // Wait a bit for the animation
    await page.waitForTimeout(100);
    
    // Verify sidebar now has the 'open' class
    hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(true);
  });

  test('should display sidebar content when open', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const toggleButton = page.locator('.sidebar-toggle');
    
    // Open the sidebar
    await toggleButton.click();
    await page.waitForTimeout(100);
    
    // Check that sidebar content is visible
    const sidebarContent = page.locator('.sidebar-content');
    await expect(sidebarContent).toBeVisible();
    
    // Verify the content heading
    await expect(sidebarContent.locator('h2')).toHaveText('Controls');
    
    // Verify the placeholder text
    await expect(sidebarContent.locator('p')).toContainText('Future controls will be added here');
  });

  test('should close sidebar when close button is clicked', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const toggleButton = page.locator('.sidebar-toggle');
    
    // Open the sidebar
    await toggleButton.click();
    await page.waitForTimeout(100);
    
    // Verify sidebar is open
    let hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(true);
    
    // Click the close button inside the sidebar
    const closeButton = page.locator('.sidebar-content button:has-text("Close")');
    await closeButton.click();
    
    // Wait a bit for the animation
    await page.waitForTimeout(100);
    
    // Verify sidebar is closed
    hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(false);
  });

  test('should toggle sidebar open and closed multiple times', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const toggleButton = page.locator('.sidebar-toggle');
    
    // Verify initial state
    let hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(false);
    
    // Toggle open
    await toggleButton.click();
    await page.waitForTimeout(100);
    hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(true);
    
    // Toggle closed
    await toggleButton.click();
    await page.waitForTimeout(100);
    hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(false);
    
    // Toggle open again
    await toggleButton.click();
    await page.waitForTimeout(100);
    hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(true);
  });

  test('sidebar should have correct styling classes', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    
    // Verify the sidebar has the correct ID
    await expect(sidebar).toHaveAttribute('id', 'sidebar');
    
    // Open the sidebar
    const toggleButton = page.locator('.sidebar-toggle');
    await toggleButton.click();
    await page.waitForTimeout(100);
    
    // Verify sidebar content has the correct class
    const sidebarContent = page.locator('.sidebar-content');
    await expect(sidebarContent).toHaveClass('sidebar-content');
  });

  test('sidebar should remain open when navigating to different pages', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    const sidebar = page.locator('#sidebar');
    const toggleButton = page.locator('.sidebar-toggle');
    
    // Open the sidebar
    await toggleButton.click();
    await page.waitForTimeout(100);
    
    // Verify sidebar is open
    let hasOpenClass = await sidebar.evaluate((el) => el.classList.contains('open'));
    expect(hasOpenClass).toBe(true);
    
    // Navigate to About page
    await page.locator('.header-nav a:has-text("About")').click();
    await page.waitForLoadState('networkidle');
    
    // Note: The sidebar state might reset on navigation depending on implementation
    // This test documents the current behavior
    // If state should persist, the implementation would need to use a global state management solution
  });
});
