import { expect, type Locator, type Page } from '@playwright/test';

/** Widest viewport (px) at which the header's links live in the sidebar. */
export const MOBILE_MAX_WIDTH = 768;

/** Whether the page's viewport is narrow enough for the phone layout. */
export function isPhone(page: Page): boolean {
  const viewport = page.viewportSize();
  return viewport !== null && viewport.width <= MOBILE_MAX_WIDTH;
}

/**
 * The main Home / Recipes / Plans link named `name`, wherever it is shown:
 * in the header on wide screens, or in the sidebar (opened first if it is
 * closed) on phones.
 */
export async function mainNavLink(page: Page, name: string): Promise<Locator> {
  if (isPhone(page)) {
    const hamburger = page.locator('#header .hamburger-btn');
    if ((await hamburger.getAttribute('aria-expanded')) !== 'true') {
      await hamburger.click();
    }
    const link = page.locator('#sidebar .sidebar-nav a', { hasText: name });
    await expect(link).toBeVisible();
    return link;
  }
  return page.locator('#header .header-nav a', { hasText: name });
}
