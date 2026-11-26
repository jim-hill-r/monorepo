---
added: 2026-11-26
unblocks: "sign-in on Cahokia production (URL updated for the jimhillr- rename), see cahokia/docs/plans/2026-11-prod-oidc-error.md"
---

In the Auth0 dashboard, open the application Cahokia uses (client ID `6CHDECRfCsyYdCFq1hwqKNwCHxxmum3E`): add `https://jimhillr-cahokia.pages.dev/` and `http://localhost:8080/` to Allowed Callback URLs, add `https://jimhillr-cahokia.pages.dev` and `http://localhost:8080` to Allowed Web Origins and Allowed Logout URLs, and check the application type is Single Page Application.

Infrastructure as code phase 1 (decision 0007) will set these URLs from code once `issues/human/auth0-iac-credentials.md` is done, so this can be skipped if sign-in on previews can wait for that.
