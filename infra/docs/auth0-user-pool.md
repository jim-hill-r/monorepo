# The shared Auth0 user pool (`auth0-tenant`)

The jimhillr account: one Auth0 tenant whose users sign in to every
product ([decision 0027](../../standards/docs/decisions/0027-one-jimhillr-account.md),
[plan](../../standards/docs/plans/2026-11-jimhillr-account.md)). `infra`
owns it; the auth0 stack manages its applications, and, from phase 3, its
branding, prompt text and connections.

## What is shared

- The tenant `dev-jdadpn4pckxevrv5.us.auth0.com`, with its users, its
  connections (the password database and any social sign-in) and its
  single sign-on session.
- Its hosted sign-in page, branded as the jimhillr account.
- Its free plan: 25,000 monthly active users and 1,000 machine-to-machine
  tokens a month, across every product together.

Not shared: each product's Auth0 application (client ID, callback URLs),
its roles (`{app}-manager`), and every product's own data, which lives in
the product's own storage keyed by `sub`.

## Why it is shared

Jim, 2026-11-28: every product should make clear you're signing in to a
jimhillr product, and every product shares a user pool. One account means
one sign-up, single sign-on between products, and one free plan.

## Who depends on it

| Product | Application | Since |
| --- | --- | --- |
| cookbook (web) | `cookbook` | before 2026-11-26 |
| cahokia (web, desktop, mobile) | `cahokia` and its native app | before 2026-11-26 |

A product that adds sign-in adds its row here, and its own
`docs/isolation.md` section for `auth0-tenant`, in the same PR.

## Migration guide: moving one product to its own tenant

What moves is the product's users and its application; everything else
stays. About 2 Claude Code PRs and 30 minutes of Jim's time.

1. **New tenant.** Jim creates a free tenant for the product (the
   [auth0-tenant runbook](../../standards/docs/runbooks/auth0-tenant.md),
   with the product's name) and its IaC applications
   ([auth0-iac-credentials](../../standards/docs/runbooks/auth0-iac-credentials.md)).
2. **New stack.** Add an auth0 stack for the new tenant in `infra`, with
   the product's application, its connections, its roles, and its own
   branding. Apply it (a credential change, so Jim approves the run).
3. **Export its users.** From the product's own list of `sub`s (rule 2 of
   decision 0027), run an Auth0 user export job on the shared tenant for
   just those users, through the Management API in a workflow. The file
   goes straight to the import step as a workflow artifact and is deleted
   after; it's personal data, so it's never committed.
4. **Import them** with a user import job on the new tenant, keeping each
   user's `user_id`, so every `sub` stays the same and the product's data
   needs no change. Users who sign in with Google or another social
   provider keep their `sub` on their own.
5. **Passwords.** Auth0 doesn't export password hashes on the free plan,
   so password users get a reset email from the new tenant on their first
   sign-in. Say so in the product's user guide and release notes.
6. **Point the product at the new tenant**: its `AUTH0_DOMAIN`, client ID
   and the Content-Security-Policy `connect-src`, then deploy.
7. **Leave the shared pool**: remove the product's application from the
   shared stack (retire it in `standards/resources.toml`, Jim approves),
   and remove its row above and its `auth0-tenant` section in its
   `docs/isolation.md`.

**Cost after separating:** $0; the new tenant has its own free plan, so
the product's users stop counting against the shared 25,000.

**How to verify:** sign in to the product with a test account and check
the ID token's `iss` is the new tenant and `sub` is unchanged; the
product's evals pass; the shared tenant's logs show no sign-ins for the
old application after the switch.

**What the user notices:** a sign-in page with the product's own name,
a password reset for password users, and no single sign-on with other
jimhillr products any more.
