# 0027. One jimhillr account: every product signs in to one shared Auth0 user pool, branded jimhillr

- Status: Accepted
- Decided by: Jim (decision card in the project thread, 2026-11-28)
- Date: 2026-11-28
- Asked by: Jim (project thread, 2026-11-28)
- Plan: [2026-11-jimhillr-account.md](../plans/2026-11-jimhillr-account.md)
- Migration guide: [infra/docs/auth0-user-pool.md](../../../infra/docs/auth0-user-pool.md)
- Builds on: [0010](0010-credential-second-approval.md) (credential
  changes), [0018](0018-feature-flags.md) (app managers sign in with
  Auth0), [0021](0021-tenant-scoped-product-data.md) (products as separate
  companies, shared infrastructure only when a product can move out)

## Context

Jim, 2026-11-28: "make sure that every login in every app makes it clear
you are logging into a jimhillr product and that every one of these
products shares a user pool."

Jim also said the same day: "Assume products are owned by different
companies during architecture." A shared user pool is the opposite of
that for one resource: every product's users live in one Auth0 tenant.
So the pool needs to be a choice written down, with rules that keep each
product separable and a guide for moving one out.

Today cookbook and cahokia already use one tenant
(`dev-jdadpn4pckxevrv5.us.auth0.com`), but nothing says so to users, and
which connections each app may use is set by hand, so the pool is shared
by accident rather than by design. The review is in the plan.

## Options

| Option | Money | Free-tier use | Build effort | One account across products | Separate companies |
| --- | --- | --- | --- | --- | --- |
| **A. One shared tenant, branded "jimhillr account", connections managed in IaC** | **$0** | **One tenant; 25,000 monthly active users shared, a person counted once** | **About 6 PRs** | **Yes, with single sign-on between products** | **Shared on purpose, with the rules below and a migration guide** |
| B. One tenant per product | $0 (each tenant on its own free plan) | 25,000 per tenant | About 4 PRs plus a human task per product to create its tenant, and a move for today's users | No: a new account per product | Fully separate |
| C. One tenant, a separate database connection per product | $0 | One tenant | About 4 PRs | No: one login page, but separate accounts | Separate pools, shared vendor account |
| D. Leave it as it is | $0 | Same as A | None | Yes, by accident | Not written down |

A is what Jim asked for. B and C give each product its own users, which
contradicts "shares a user pool". D leaves the pool unchecked.

## Decision

1. **One jimhillr account.** Every product that signs people in (end users
   and app managers) uses the one shared Auth0 tenant. Staff sign-in to
   internal tools uses Cloudflare Access instead (PR 926).
2. **Say it everywhere.** Apps use `design_system_dioxus::AccountButton`
   ("Sign in with jimhillr", "One jimhillr account works in every jimhillr
   app"), and the hosted page's name, logo and words say "jimhillr
   account" and name the product through Auth0's `${clientName}`.
3. **The tenant is owned by `infra`.** Its branding, prompt text and
   which connections each client may use are IaC in the auth0 stack, and a
   test fails any product with sign-in that isn't on every shared
   connection. `auth0_tenant`, `auth0_branding`, `auth0_prompt*` and
   `auth0_connection_clients` are credential changes: Jim approves each
   run in `infra-credentials`.
4. **Rules that keep products separable:**
   1. Each product has its own Auth0 application and its own roles.
   2. A product keys its data by `sub`, never by email, and keeps its own
      list of the `sub`s that used it.
   3. Products never share `app_metadata` or other per-user data through
      Auth0; any key a product writes starts with its name.
   4. Nothing a product's requests pass through is shared besides Auth0's
      own endpoints.
5. **Moving a product out** follows the
   [migration guide](../../../infra/docs/auth0-user-pool.md). Each product
   that signs in lists the tenant in its `docs/isolation.md` under the
   shared resource `auth0-tenant`.

## Consequences

- Money: $0. Everything used is on Auth0's free plan and GitHub.
- Free tier: one tenant's 25,000 monthly active users cover every product
  together, so a product that grows fast uses up the others' headroom.
  The cost check watches the total (`cost-tracking-phase-3-auth0-usage`).
- A person signed in to one product is signed in to the others in the same
  browser, which is the point, and also means one stolen session reaches
  every product. Short sessions and multi-factor for managers (decision
  0018) limit that.
- The address bar still shows Auth0's host until a custom domain is
  allowed; adding one asks for a card, which needs Jim's call on the
  free-tier rule.
- Moving a product out later means its users with passwords set a new
  one (Auth0 won't export password hashes on the free plan); people who
  sign in with Google don't notice.
