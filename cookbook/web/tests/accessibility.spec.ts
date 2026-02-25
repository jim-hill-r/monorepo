import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility tests for the Cookbook web application.
 *
 * These tests use axe-core to check for WCAG compliance violations
 * on key pages of the application.
 *
 * Before running these tests, start the Dioxus dev server:
 *   cast run
 *
 * Run tests with:
 *   npm test
 */

test.describe('Accessibility', () => {
  test('home page should have no critical accessibility violations', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');

    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();

    expect(results.violations).toEqual([]);
  });

  test('recipe page should have no critical accessibility violations', async ({ page }) => {
    await page.goto('/recipe/1');
    await page.waitForLoadState('networkidle');

    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();

    expect(results.violations).toEqual([]);
  });

  test('plan page should have no critical accessibility violations', async ({ page }) => {
    await page.goto('/plan/1');
    await page.waitForLoadState('networkidle');

    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();

    expect(results.violations).toEqual([]);
  });

  test('404 page should have no critical accessibility violations', async ({ page }) => {
    await page.goto('/invalid-route');
    await page.waitForLoadState('networkidle');

    const results = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();

    expect(results.violations).toEqual([]);
  });
});
