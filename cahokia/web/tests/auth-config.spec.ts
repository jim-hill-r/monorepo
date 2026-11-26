import { test, expect, type Page } from '@playwright/test';

/**
 * Guards for the Auth0 sign-in config (cahokia/docs/plans/2026-11-prod-oidc-error.md).
 *
 * These talk to the real Auth0 tenant for OIDC discovery, so a broken issuer,
 * client or CSP fails CI instead of showing up on the live site.
 */

const AUTH0_AUTHORIZE = 'https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize**';

/** Wait until the navbar has finished loading auth (Login or an error). */
async function waitForAuth(page: Page) {
  const navbar = page.locator('#navbar');
  await expect(navbar).toBeVisible();
  await expect(navbar.getByText('Loading authentication...')).toHaveCount(0, {
    timeout: 30_000,
  });
}

test.describe('Auth0 sign-in config', () => {
  // AUTH-003: the home page must never render "Authentication Error".
  test('home page shows Login, never an authentication error', async ({ page }) => {
    await page.goto('/');
    await waitForAuth(page);

    await expect(page.getByText('Authentication Error')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Login' })).toBeVisible();
  });

  // AUTH-002: the redirect URI is the site root from every page, so it
  // matches Auth0's Allowed Callback URLs exactly.
  test('login from another page sends the site root as redirect_uri', async ({
    page,
    baseURL,
  }) => {
    let authorizeUrl: URL | undefined;
    await page.route(AUTH0_AUTHORIZE, async (route) => {
      authorizeUrl = new URL(route.request().url());
      await route.fulfill({ status: 200, contentType: 'text/html', body: 'stub' });
    });

    await page.goto('/history?tab=timeline');
    await waitForAuth(page);
    await page.getByRole('button', { name: 'Login' }).click();

    await expect.poll(() => authorizeUrl?.toString()).toBeTruthy();
    const origin = new URL(baseURL ?? 'http://localhost:8080').origin;
    expect(authorizeUrl?.searchParams.get('redirect_uri')).toBe(`${origin}/`);
  });
});
