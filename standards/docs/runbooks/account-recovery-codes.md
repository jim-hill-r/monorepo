# Save the account recovery codes offline

- **Kind:** one-time, then checked every 365 days (settings check `account-recovery-codes`)
- **Status:** open (issues/human/save-account-recovery-codes.md)
- **Time:** about 45 minutes
- **Cost:** $0

## What it's for

Every backup in [decision 0030](../decisions/0030-backups-and-disaster-recovery.md)
sits inside an account. If a phone is lost or a sign-in is locked, these
codes are how you get the account back. Without them, losing the GitHub
account loses the repo's home, and losing Cloudflare loses the backups and
every site.

## Why a person

Recovery codes are shown only to the signed-in owner, and they must be kept
where no agent, workflow or vendor account can reach them, or they don't
survive losing that account.

## Before you start

- Your password manager open, and paper and a pen (or a printer).
- Your phone or security key for each account's 2FA.

## Steps

For each account below, sign in, open the page, make sure two-factor
authentication is on, and save the recovery codes (or register a second
passkey or security key and note where it is kept). Save each in your
password manager with the account's name, then write or print them all on
one sheet kept somewhere safe at home, away from your computer.

1. **GitHub `jimhillr`**: github.com > Settings > Password and
   authentication > Recovery codes > Download. Also add a second passkey
   if you have only one.
2. **GitHub `jimhillr-agt`** and **`jimhillr-reviewer`**: the same page,
   signed in as each.
3. **Cloudflare**: dash.cloudflare.com > My Profile > Authentication >
   Two-Factor Authentication > Backup codes.
4. **Auth0**: manage.auth0.com > your avatar > Your Profile > Multi-factor
   authentication > Recovery code.
5. **PostHog**: us.posthog.com > Settings > User > Two-factor
   authentication > Backup codes.
6. **Squarespace**: account.squarespace.com > Account settings > Security >
   Two-step verification > Backup codes.
7. **Claude**: claude.ai > Settings > Account: check how you sign in (an
   email link or Google). Make sure you can reach that email account
   without this computer, and save its own recovery codes too.
8. **Your email account** (where every vendor sends resets): save its
   recovery codes and check its recovery phone number.

## Where secrets go

Only your password manager and the paper copy. Never in the repo, a GitHub
or vendor secret, a chat or a note an agent can read.

## Verify

Tell any thread "recovery codes saved" with the list of accounts done (no
codes). It deletes the issue file and records the settings check.

## Undo or rotate

If a code is used or the sheet is lost, make new codes on the same page
(that voids the old ones), and update the password manager and the paper
copy. Do this yearly when the settings check asks.
