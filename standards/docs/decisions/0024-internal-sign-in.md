# 0024. Internal sign-in: Cloudflare Access with GitHub login for internal tools, Auth0 only for products

- Status: Accepted
- Decided by: Jim (chose "Access with GitHub login" on the decision card, 2026-11-28)
- Date: 2026-11-28
- Asked by: Jim (deck sign-off thread, 2026-11-28)
- Plan: [pane deck sign-off](../../../pane/docs/plans/2026-11-deck-sign-off.md)
- Changes: [0020](0020-user-feedback-and-support.md) item 7 and the user
  feedback plan's staff sign-in; builds on
  [0010](0010-credential-second-approval.md) (Access policies are credential
  changes) and [0018](0018-feature-flags.md) (app managers on Auth0)

## Context

Pane needs a sign-in so a reviewer can sign off on a review deck. The first
proposal put pane on Auth0 with a reviewer role, alongside the products.
Jim: "The risk is that our end users would be in the same auth domain as our
internal use cases. That is not my favorite idea. We are already sharing a
single auth0 tenant across products which is non ideal. Let's really be sure
this is the pattern we want."

Nothing used Cloudflare Access yet; it appeared only in plans (pane,
the eval engine's feedback sign-in, decisions 0010 and 0014). Auth0 is live
in cookbook and cahokia, and decision 0018 uses it for app managers. Auth0's
free plan allows one tenant per account.

## Decision

1. **Two sign-in systems, split by who signs in.**
   - **Internal people** (Jim, reviewers, engineers, product owners) sign in
     to **internal tools** (pane, and platform management in pane) through
     **Cloudflare Access**, with **GitHub** as the login method. Access
     allows an Access group Jim keeps by hand in the Cloudflare dashboard,
     since emails can't be in the public repo.
   - **Products' people** (end users and each product's app managers) sign
     in through **Auth0** with `auth_sdk`, as today.
   - Nobody internal needs an account in a product's user pool, and no
     product user can reach an internal tool.
2. **Workers check the Access token and fail closed.** An internal Worker
   verifies `Cf-Access-Jwt-Assertion` against the Access team's public keys
   (issuer, the application's audience, expiry) on every write, so a
   misconfigured Access policy can't let anyone through.
3. **No email reaches the repo.** Where a record names an internal person
   (a deck sign-off), the Worker maps the Access email to a public handle
   from a Worker secret Jim sets, and writes only the handle.
4. **Staff feedback comes through the extension, to pane.** The product
   owner and engineering roles of decision 0020 item 7 are internal, so
   they no longer sign in through Auth0 on a product's page. The staff modes
   (pins, screenshots, console errors, the build) live in the browser
   extension (the user feedback plan's Phase 8), which signs in through
   pane's Access login and sends to pane. No product build ships staff code,
   so SPRK-006 holds and no product needs an Access application of its own.
   App managers stay on Auth0 and their product's own inbox.
5. **Each product's own Auth0 later.** Under "products are separate
   companies", each product should eventually have its own free Auth0
   account rather than share one tenant. That is a later decision; this one
   doesn't need it.

## Alternatives considered

| Option | Money cost | Free-tier headroom | Build effort | Why not |
| --- | --- | --- | --- | --- |
| **Access with GitHub login** (chosen) | $0 | Free up to 50 users; we'd use about 2 | About 2 PRs for deck sign-off | |
| Auth0 for internal people too, same tenant | $0 | 25,000 monthly users; a handful | About 3 PRs | Internal and end users in one pool, which Jim rejected |
| A second Auth0 tenant for internal people | $0 | Free plan allows 1 tenant per account, so a second account | About 3 PRs | A second vendor account to run, for what Access does |
| Our own GitHub OAuth in the Worker | $0 | GitHub's OAuth apps are free | About 3 PRs | More code, and a client secret to store and rotate |
| Staff sign-in through Access on each product's origin | $0 | One Access application per product, of the 50-user plan | About 1 PR per product | Every product carries staff code and its own Access app; the extension needs neither |

## Consequences

- Pane's Access setup (`pane/issues/put-pane-behind-cloudflare-access-allowing.md`)
  is the first Access application, a credential change that needs Jim's
  second approval (decision 0010). It needs a GitHub OAuth app for the login
  method, made by Jim, with its client secret stored only in Cloudflare.
- The user feedback plan's staff sign-in and its Phase 8 extension follow
  item 4.
- Cost: $0. No new vendor: Cloudflare and GitHub are approved.
