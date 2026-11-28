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

/**
 * Waits until the app has rendered the page: its `#content h1`, which every
 * route has (home, recipe, plan, error and not-found pages). Use it after
 * `page.goto` or `page.reload` instead of `waitForLoadState('networkidle')`,
 * which Playwright discourages and which can outlast the test timeout while
 * the Auth0 discovery request is in flight. It then waits for the page's
 * stylesheets and colour transitions (`animationsDone`).
 */
export async function pageReady(page: Page): Promise<void> {
  await expect(page.locator('#content h1').first()).toBeVisible();
  await animationsDone(page);
}

/**
 * Waits for the page's styles to settle: every stylesheet link loaded, then
 * no finite animation or transition still running. The app adds its
 * stylesheets after it renders, and the header and cards fade their colours
 * in (`transition: all`), so an accessibility scan or a style check made
 * before then reads the wrong colours.
 */
export async function animationsDone(page: Page): Promise<void> {
  await page.waitForFunction(() =>
    Array.from(document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')).every(
      (link) => link.sheet !== null,
    ),
  );
  await expect
    .poll(() =>
      page.evaluate(async () => {
        const running = () =>
          document
            .getAnimations()
            .filter((a) => a.effect?.getComputedTiming().endTime !== Infinity);
        await Promise.all(running().map((a) => a.finished.catch(() => undefined)));
        // A style applied by the last frame can start a new transition.
        await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        return running().length;
      }),
    )
    .toBe(0);
}
