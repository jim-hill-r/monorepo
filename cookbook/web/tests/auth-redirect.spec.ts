import { test, expect, type Page } from '@playwright/test';

/**
 * Sign-in redirect URI (cookbook/issues/stable-auth0-redirect-uri.md).
 *
 * Auth0 accepts only an exact Allowed Callback URL, so Login must send the
 * site root as `redirect_uri` from every page, never the page's own URL.
 * These talk to the real Auth0 tenant for OIDC discovery; only the authorize
 * redirect is stubbed.
 */

const AUTH0_AUTHORIZE = 'https://dev-jdadpn4pckxevrv5.us.auth0.com/authorize**';

/** Wait until the header has finished loading auth (Login or an error). */
async function waitForAuth(page: Page) {
  const header = page.locator('#header');
  await expect(header).toBeVisible();
  await expect(header.getByText('Loading authentication...')).toHaveCount(0, {
    timeout: 30_000,
  });
}

test.describe('Sign-in redirect', () => {
  for (const path of ['/recipe/100?from=plan', '/plan/12']) {
    test(`login from ${path} sends the site root as redirect_uri`, async ({
      page,
      baseURL,
    }) => {
      let authorizeUrl: URL | undefined;
      await page.route(AUTH0_AUTHORIZE, async (route) => {
        authorizeUrl = new URL(route.request().url());
        await route.fulfill({ status: 200, contentType: 'text/html', body: 'stub' });
      });

      await page.goto(path);
      await waitForAuth(page);
      await page.locator('#header').getByRole('button', { name: 'Login' }).click();

      await expect.poll(() => authorizeUrl?.toString()).toBeTruthy();
      const origin = new URL(baseURL ?? 'http://localhost:8080').origin;
      expect(authorizeUrl?.searchParams.get('redirect_uri')).toBe(`${origin}/`);
    });
  }
});
