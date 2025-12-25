import { test, expect } from '@playwright/test';

/**
 * Playwright tests for Cahokia earthy color scheme.
 * 
 * These tests verify that the app uses earthy colors inspired by ancient burial mounds.
 * 
 * Before running these tests, start the Dioxus dev server:
 *   cd cahokia/web
 *   dx serve --port 8080
 * 
 * Run tests with:
 *   npm test -- earthy-colors.spec.ts
 */

test.describe('Earthy Color Scheme', () => {
  test('should have earthy background color on body', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the body has an earthy background color
    const bodyBgColor = await page.locator('body').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    
    // The background should be an earthy brown tone
    // #3d2f1f converts to rgb(61, 47, 31)
    expect(bodyBgColor).toBe('rgb(61, 47, 31)');
  });

  test('should have earthy cream text color on body', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that the body text is an earthy cream color
    const bodyTextColor = await page.locator('body').evaluate((el) => {
      return window.getComputedStyle(el).color;
    });
    
    // The text should be cream/beige
    // #f5e6d3 converts to rgb(245, 230, 211)
    expect(bodyTextColor).toBe('rgb(245, 230, 211)');
  });

  test('should have earthy brown header background', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check header background color
    const headerBgColor = await page.locator('#header').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    
    // Header should have brown color #8b7355
    expect(headerBgColor).toBe('rgb(139, 115, 85)');
  });

  test('should have earthy brown border on header', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check header border color
    const headerBorderColor = await page.locator('#header').evaluate((el) => {
      return window.getComputedStyle(el).borderBottomColor;
    });
    
    // Header border should be darker brown #654321
    expect(headerBorderColor).toBe('rgb(101, 67, 33)');
  });

  test('should have earthy colors in hero section', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check that hero headings exist and are visible
    const heroH1 = page.locator('#hero h1');
    const heroH2 = page.locator('#hero h2');
    
    await expect(heroH1).toBeVisible();
    await expect(heroH2).toBeVisible();
    
    // Verify h1 has earthy golden color
    const h1Color = await heroH1.evaluate((el) => {
      return window.getComputedStyle(el).color;
    });
    
    // #d4a574 converts to rgb(212, 165, 116)
    expect(h1Color).toBe('rgb(212, 165, 116)');
  });

  test('should have earthy sidebar colors', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Check sidebar background color
    const sidebarBgColor = await page.locator('#sidebar').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    
    // Sidebar should have dark brown #654321
    expect(sidebarBgColor).toBe('rgb(101, 67, 33)');
  });

  test('should have consistent earthy color palette throughout', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    
    // Navigate to different pages and verify colors are consistent
    await page.locator('.header-nav a:has-text("About")').click();
    await page.waitForLoadState('networkidle');
    
    let bodyBgColor = await page.locator('body').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    expect(bodyBgColor).toBe('rgb(61, 47, 31)');
    
    // Navigate to History
    await page.locator('.header-nav a:has-text("History")').click();
    await page.waitForLoadState('networkidle');
    
    bodyBgColor = await page.locator('body').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    expect(bodyBgColor).toBe('rgb(61, 47, 31)');
    
    // Navigate to Explore
    await page.locator('.header-nav a:has-text("Explore")').click();
    await page.waitForLoadState('networkidle');
    
    bodyBgColor = await page.locator('body').evaluate((el) => {
      return window.getComputedStyle(el).backgroundColor;
    });
    expect(bodyBgColor).toBe('rgb(61, 47, 31)');
  });

  test('page content headings should have golden ochre color', async ({ page }) => {
    await page.goto('/about');
    await page.waitForLoadState('networkidle');
    
    // Check h2 color in page content
    const h2Color = await page.locator('.page-content h2').evaluate((el) => {
      return window.getComputedStyle(el).color;
    });
    
    // #d4a574 converts to rgb(212, 165, 116)
    expect(h2Color).toBe('rgb(212, 165, 116)');
  });
});
