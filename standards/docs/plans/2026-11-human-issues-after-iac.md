# Human issues after infrastructure as code

Jim asked (2026-11-27) for every file in `issues/human/` to be re-checked now
that IaC exists (decisions 0007, 0009 and 0010). His view: the only setup
left for a person should be the `infra` environments and the bootstrap
secrets that let automation make every other token.

## Closed in this PR

| Issue | Why |
| --- | --- |
| `auth0-dashboard-cloudflare-pages-preview-domains` | The `auth0` stack sets the preview callback, origin and logout URLs for cookbook and cahokia (`infra/out/auth0/main.tf.json`). |
| `auth0-dashboard-open-application-cahokia-uses` | The same stack sets the `jimhillr-cahokia` and localhost URLs and the Single Page Application type. Cahokia sign-in on production waits for the first `auth0` apply. |
| `cloudflare-dashboard-give-cloudflare-api-token` | Workers Scripts: Read is covered by Workers Scripts: Edit in `cloudflare-api-token-workers-d1-edit`. |

`delete-old-pages-projects-after-jimhillr-rename` keeps only the Pages deletes:
the `auth0` apply removes the old `cahokia-web` URLs, and IaC never deletes
(decision 0007, item 8).

## Minted by a workflow (Jim chose this on 2026-11-27)

OpenTofu can't mint them under the rule "never commit secrets to git or store
them in tf state": an Auth0 client secret or a Cloudflare token value comes
back from the vendor API as a computed attribute and lands in state, and
IAC-002 fails that plan. A workflow outside OpenTofu can: in the
`infra-credentials` environment (each run waits for Jim's approval), it calls
the vendor API with a bootstrap token and pipes the new value straight into
`gh secret set`, masked in the log, never on disk, in git or in state.

These now fold into the Infra Mint workflow (`infra/scripts/mint.sh`), and the files are deleted:

| Issue | What the workflow does instead |
| --- | --- |
| `tofu-state-passwords` | Generates both passwords and the passphrase, sets the GitHub secrets |
| `tofu-state-origin-variable` (PR #726) | Sets `TOFU_STATE_ORIGIN` from the account's workers.dev subdomain |
| `auth0-iac-credentials` | Creates the infra-read and infra-apply M2M apps and their grants; the M2M budget is still Jim's number |
| `cloudflare-iac-credentials` | Creates the infra-read and infra-apply tokens |
| `cloudflare-api-token-workers-d1-edit` | Mints `CLOUDFLARE_API_TOKEN` as the account token `jimhillr-deploy` with Pages, Workers Scripts and D1 Write |
| `cloudflare-dashboard-turn-web-analytics-production` | Gives `jimhillr-deploy` Account Analytics Read |
| `optional-create-android-release-key-controlled` | Generates the keystore in the job and stores it as secrets |

`infra-credentials-environment` is now the one setup task: the two
environments and three bootstrap credentials (an Auth0 app that can create
apps and grants, a Cloudflare token with Account API Tokens: Edit, and the
infra GitHub App from `github-app-infra-credentials`, which Jim asked for in
place of a PAT), all only in `infra-credentials`.

The `github` stack (2026-11-27) replaced
`github-settings-rules-rulesets-default-ruleset` with a `CI` ruleset and
took over all of `github-settings-code-security-turn-secret` but private
vulnerability reporting, which the provider can't set. Its plans need a
read-only GitHub App, `github-app-infra-read`: GitHub has no API to make
one.

## Staying with a person

| Issue | Why |
| --- | --- |
| `infra-credentials-environment` | Someone has to make the first tokens and be the second approver (decision 0010). |
| `check-there-still-no-payment-method` | Billing pages have no read API for these vendors; it's a recurring check by eye. |
| `create-posthog-cloud-free-plan-no` | Signing up for an account, and PostHog personal API keys can't be made by API without one. |
| `delete-old-pages-projects-after-jimhillr-rename` | A delete, which IaC never does. |
| `open-terrars-upstream-prs` | Agents can only reach this repository. Optional: nothing here is blocked on it. |
