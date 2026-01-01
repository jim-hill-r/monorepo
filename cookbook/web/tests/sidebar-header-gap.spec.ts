import { test, expect } from '@playwright/test';

// Tolerance values for sub-pixel rendering and browser differences
const PIXEL_TOLERANCE = 1;
const SCROLLBAR_TOLERANCE = 5;

/**
 * Tests for the gap between sidebar and header.
 *
 * This test verifies that the sidebar starts exactly where the header ends,
 * with no visible gap between them.
 */

test.describe('Sidebar and Header Alignment', () => {
  test('sidebar should touch header with no gap', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Get header element
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Get sidebar element
    const sidebar = page.locator('#sidebar');
    
    // On mobile, sidebar is hidden by default, so skip alignment check
    if (viewport && viewport.width <= 768) {
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    
    await expect(sidebar).toBeVisible();
    
    // Get the bounding boxes
    const headerBox = await header.boundingBox();
    const sidebarBox = await sidebar.boundingBox();
    
    expect(headerBox).not.toBeNull();
    expect(sidebarBox).not.toBeNull();
    
    if (headerBox && sidebarBox) {
      // The sidebar top should equal header bottom (header.y + header.height)
      const headerBottom = headerBox.y + headerBox.height;
      const sidebarTop = sidebarBox.y;
      
      // Allow small tolerance for sub-pixel rendering differences
      const gap = Math.abs(sidebarTop - headerBottom);
      expect(gap).toBeLessThanOrEqual(PIXEL_TOLERANCE);
    }
  });
  
  test('sidebar height should extend to bottom of viewport', async ({ page, viewport }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Get sidebar element
    const sidebar = page.locator('#sidebar');
    
    // On mobile, sidebar is hidden by default, so skip height check
    if (viewport && viewport.width <= 768) {
      await expect(sidebar).toHaveClass(/hidden/);
      return;
    }
    await expect(sidebar).toBeVisible();
    
    // Get the bounding box
    const sidebarBox = await sidebar.boundingBox();
    const viewportSize = page.viewportSize();
    
    expect(sidebarBox).not.toBeNull();
    expect(viewportSize).not.toBeNull();
    
    if (sidebarBox && viewportSize) {
      // The sidebar should extend from header bottom to viewport bottom
      const sidebarBottom = sidebarBox.y + sidebarBox.height;
      
      // Allow tolerance for scrollbars and rounding
      expect(sidebarBottom).toBeGreaterThanOrEqual(viewportSize.height - SCROLLBAR_TOLERANCE);
    }
  });
  
  test('CSS custom property should be used for header height', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the CSS custom property is defined
    const headerHeightVar = await page.evaluate(() => {
      return getComputedStyle(document.documentElement).getPropertyValue('--header-height');
    });
    
    expect(headerHeightVar).toBeTruthy();
    expect(headerHeightVar.trim()).toBe('60px');
  });
  
  test('header should have consistent height', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Get header element
    const header = page.locator('#header');
    await expect(header).toBeVisible();
    
    // Check computed height
    const headerHeight = await header.evaluate((el) => {
      return window.getComputedStyle(el).height;
    });
    
    // Should be 60px as defined in CSS custom property
    expect(headerHeight).toBe('60px');
  });
});
