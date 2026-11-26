# Security policy

## Reporting a vulnerability

Please report security problems privately through GitHub's private
vulnerability reporting: open the repository's **Security** tab and choose
**Report a vulnerability**. Don't open a public issue or pull request for it.

Include what is affected (a project, a site URL or a workflow), how to
reproduce it, and what an attacker could do with it. If you found a leaked
secret, say where it is, but don't paste the secret itself.

This is a one-person hobby monorepo, so there is no bounty and no fixed
response time, but every report is read and answered.

## Scope

Everything in this repository, and the sites and Workers it deploys to
Cloudflare. Third-party services it uses (Auth0, PostHog, RapidAPI, GitHub,
Cloudflare) have their own reporting programs.

## How the repo protects itself

The rules and checks (secret scanning on every commit and pull request,
Dependabot on every lockfile, ignored secret files) are in
[standards/docs/security.md](standards/docs/security.md).
