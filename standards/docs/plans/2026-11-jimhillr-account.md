# One jimhillr account: a clear, shared sign-in for every product

- Asked by: Jim (project thread, 2026-11-28): "make sure that every login
  in every app makes it clear you are logging into a jimhillr product and
  that every one of these products shares a user pool."
- Decision: [0027](../decisions/0027-one-jimhillr-account.md) (accepted)
- Migration guide for the shared pool:
  [infra/docs/auth0-user-pool.md](../../../infra/docs/auth0-user-pool.md)
- Scope: every end user and app manager who signs in through Auth0 today
  (cookbook, cahokia web, desktop and mobile) and every product that adds
  sign-in later through `auth_sdk`. Staff sign-in to internal tools such as
  pane moves to Cloudflare Access (PR 926) and is out of scope.

## Review: today's sign-in

What a person sees today, from the code on `main` (2026-11-28) and Auth0's
defaults. The sandbox can't reach `*.auth0.com` (the proxy refuses it), so
the hosted page is described from the tenant's settings in the repo and
Auth0's documented defaults, not from a screenshot; phase 2's read-only
plan confirms the live settings.

### In the apps

| Finding | Where | Effect |
| --- | --- | --- |
| The button says only **Login**, with no product family named | `cookbook/web/src/auth.rs` `render_auth_section`, `cahokia/web/src/main.rs` `WebNavbar` | Nothing says this is a jimhillr account, or that it works anywhere else |
| The button says **Login** even after you sign in; no name, no sign out | both apps | You can't tell you're signed in. `WebAuthProvider::logout` is `todo!()` ([issue](../../../auth_sdk/issues/web-provider-todo-panics.md)), so a sign-out button would panic |
| Each app writes its own sign-in UI and error strings | cookbook has `format_auth_error`, cahokia prints `Authentication Error: {err}` with the raw error | Two wordings now, a third with every new product; cahokia shows technical text to users |
| "Loading authentication..." placeholder | both apps | Engineering wording in the header on every page load |

### On Auth0's hosted page (Universal Login)

| Finding | Effect |
| --- | --- |
| The address bar shows `dev-jdadpn4pckxevrv5.us.auth0.com` | A random vendor host, with nothing linking it to jimhillr. A custom domain would fix it, but Auth0 asks for a card to add one, which the free-tier rule forbids ([auth0-tenant runbook](../runbooks/auth0-tenant.md)) |
| No tenant friendly name or logo is set in the repo; Auth0's default heading is "Log in to *tenant name* to continue to *app name*" | The page names the tenant slug, or nothing, instead of jimhillr |
| The application names are lowercase `cookbook` and `cahokia` (`infra/src/constructs.rs`) | "continue to cookbook" reads like an internal id |
| Auth0's default logo and colors | Nothing ties the page to our design system |
| Nothing says one account works across products | People may sign up twice, or think each app is a separate account |
| Which connections (password database, Google) each app may use is set by hand in the dashboard, not in IaC | The pool is shared only as long as every app has the same connections turned on. Nothing checks it, so a new app could quietly get its own pool |
| Verification and password-reset emails use Auth0's default templates | Custom templates need our own email provider; the emails show the tenant's friendly name once it's set, which is the most we get for $0 |

What already works: both apps use the same tenant, so a person who signed
in to cookbook gets straight into cahokia in the same browser (Auth0's
single sign-on session), and desktop and mobile sign-in open the same
hosted page in the system browser, so branding there covers native apps too.

## Design

### Words

The account is called a **jimhillr account** everywhere: in the apps, on
the hosted page, in user guides. Products keep their own names (Cookbook,
Cahokia), and the account sits under them.

| Place | Text |
| --- | --- |
| App header, signed out | Button: **Sign in with jimhillr** |
| App header, signed in | The person's name (or email), and **Sign out** |
| Hint under the button, or its tooltip on small screens | "One jimhillr account works in every jimhillr app." |
| Hosted page title | **Sign in to your jimhillr account** |
| Hosted page description | "Continue to ${clientName}. The same account works in every jimhillr app." |
| Sign-up title | **Create your jimhillr account** |
| Sign-up description | "One account for ${clientName} and every other jimhillr app." |
| Application names in Auth0 | **Cookbook**, **Cahokia** (what `${clientName}` shows) |
| Tenant friendly name | **jimhillr** (shown in emails and the page's fallback text) |
| Errors | One short, non-technical message per `AuthError`, shared by every app |

`${clientName}` is Auth0's own variable for the application's name, so one
tenant-wide text works for every product.

### In the apps: one shared component

A `design_system_dioxus::AccountButton` component draws the signed-out
button, the hint, the signed-in name and sign out, and the error message,
with the design system's tokens. It takes plain values and callbacks, so
the design system doesn't depend on `auth_sdk` and stays usable with any
provider; each app passes its `WebAuthProvider` state in a few lines. The
words live in the component, so a new product gets them by using it. The
`AuthError` wording moves from cookbook's `format_auth_error` into
`auth_sdk` (`AuthError::user_message`) so web, desktop and mobile share it.

`WebAuthProvider` gets working `is_authenticated` and `logout` (the open
[auth_sdk issue](../../../auth_sdk/issues/web-provider-todo-panics.md)),
since a signed-in state needs both.

### On the hosted page: branding as IaC

All of these are on Auth0's free plan and set in the `infra` auth0 stack,
never in the dashboard:

- `auth0_tenant`: friendly name `jimhillr`, picture URL to the jimhillr mark.
- `auth0_branding`: the jimhillr mark as logo, primary color and page
  background from `design_system` tokens (the neutral theme, not a
  product's theme, since the page belongs to the account, not one product).
- `auth0_prompt`: the New Universal Login experience.
- `auth0_prompt_custom_text`: the titles and descriptions above, for the
  `login`, `login-id`, `login-password` and `signup` prompts, in English.
- `auth0_client`: display names `Cookbook` and `Cahokia`, and `logo_uri`
  to each product's own icon (shown on consent screens).
- `auth0_connection_clients`: every product's client is enabled on exactly
  the same connections. A test in `infra` fails a product with sign-in
  that isn't on all of them, so the pool can't split by accident.

The jimhillr mark is a small SVG and PNG in `design_system/brand/`, served
from `raw.githubusercontent.com` (GitHub, approved, $0) until the design
system has a site of its own. Auth0's custom page templates need a custom
domain, so the layout itself stays Auth0's.

### Shared pool rules (decision 0027)

The shared tenant is one of the few things products share at runtime, so
it is written down as an intentional shared resource with its own
[migration guide](../../../infra/docs/auth0-user-pool.md). To keep
products separable (decision 0021's spirit):

1. Each product has its own Auth0 application, callback URLs and roles
   (`{app}-manager`); no product signs in with another's client ID.
2. A product keys its data by the user's `sub`, never by email, and keeps
   its own list of the `sub`s that have used it, in its own storage.
3. No product reads or writes another product's data through Auth0: no
   shared `app_metadata` keys, only keys prefixed with the product's name.
4. The tenant's settings (branding, text, connections) are owned by
   `infra`; a product only adds its own client.

## Phases

| Phase | What | Credential change? | Size |
| --- | --- | --- | --- |
| 1 | `AccountButton` in `design_system_dioxus`, `AuthError::user_message` and working sign-in state and sign out in `auth_sdk`, both apps switched over, Playwright tests on the new words | No | 2 to 3 PRs |
| 2 | `infra`: generate bindings for the five Auth0 types, add them to the guard's allow list (none can bill), classify `auth0_tenant`, `auth0_branding`, `auth0_prompt*` and `auth0_connection_clients` as credential changes, import the tenant's live settings and read them in a plan | Classification only | 1 to 2 PRs |
| 3 | `infra`: set the branding, text, names and connections above. The run waits for Jim in `infra-credentials` | Yes, Jim approves the run | 1 PR |
| 4 | An eval that the hosted page and each app's button say "jimhillr", user guide page "Your jimhillr account" linked from every signed-in product's guide, isolation sections per product | No | 1 PR |

Phase 1 changes nothing outside our own code. Jim accepted decision 0027
on 2026-11-28, so phases 2 and 3 can follow; phase 3's apply still waits
for his approval of the run in `infra-credentials`.

## Cost

$0 in money. Universal Login branding, custom text, prompts and connection
settings are on Auth0's free plan; the hosted logo is a file in a public
GitHub repo. Monthly active users stay inside the free 25,000 (the shared
pool counts a person once across products, which helps). Build effort is
about 6 Claude Code PRs over the phases, a small share of the $25 a month
budget. Jim's time: accepting the decision and approving one
`infra-credentials` run.

## Risks

- A wrong connection setting could stop people signing in to a product.
  Phase 2 reads the live settings first, and phase 3's plan shows the
  exact change before Jim approves it.
- Changing the hosted page's look can make an existing user wary. The
  page keeps Auth0's layout; only the name, logo and words change.
- The vendor host in the address bar stays until a custom domain is
  allowed, which needs a card and so Jim's call on the free-tier rule.
