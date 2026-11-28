# Turn off GitHub email notifications on the three jimhillr accounts

- **Kind:** one-time per account (do it again for any new jimhillr
  account), then a settings check every 180 days
- **Status:** open (standards/docs/issues/human/github-email-notifications-off.md)
- **Time:** about 5 minutes per account, 15 in all
- **Cost:** $0 (account settings on GitHub's Free plan)
- **Asked for:** Jim, 2026-11-28: "remove all email notifications from all
  three of the jimhillr-** users."

## What it's for

Agents open, push, review and merge many PRs a day as `jimhillr-agt`,
`jimhillr-reviewer` reviews them, and Actions runs on every push, so each
account gets a flood of GitHub email. This turns all of it off. Nothing
is lost: every notification still reaches the GitHub web inbox
(github.com/notifications) and GitHub Mobile, the weekly human issues
digest still lists what waits on Jim, and the thread that needs Jim asks
him in the project chat.

GitHub always sends some email that no setting turns off: sign-in and
security alerts, two-factor and verification codes, password resets,
billing receipts and changes to its terms. Leave those alone; they are
how GitHub reaches you if an account is attacked.

## Why a person

GitHub has no API for an account's notification or email settings. They
are only on each account's own signed-in settings pages, and agents never
hold a password or session for these accounts (decision 0010).

## Before you start

- Signed in to GitHub as each account in turn: `jimhillr`, `jimhillr-agt`
  and `jimhillr-reviewer`. A separate browser profile (or a private
  window) per account saves signing out and in.
- `jimhillr-reviewer` may not exist yet. If not, skip it here; step 6 of
  [its own runbook](jimhillr-reviewer-account.md) points back to this one.

## Steps

Do steps 1 to 4 once for each of the three accounts.

1. **Notifications page.** Open https://github.com/settings/notifications
   (avatar > Settings > Notifications).
   - **Subscriptions > Watching:** click **Notify me** and untick
     **Email**. Keep **On GitHub** (and **CLI** if shown) ticked.
   - **Subscriptions > Participating, @mentions and custom:** click
     **Notify me** and untick **Email**. Keep **On GitHub** ticked.
   - **Customize email updates:** untick everything (Reviews, Pushes,
     Comments on Issues and Pull Requests, Includes your own updates).
     These only matter while email is on, so this is belt and braces.
   - **System > Actions:** click **Notify me** and untick **Email**.
     Keep **On GitHub** and, if you like, **Only notify for failed
     workflows**.
   - **System > Dependabot alerts: New vulnerabilities:** untick
     **Email**; keep **On GitHub**. Set **Dependabot alerts: Email weekly
     digest** to **Don't send**.
   - **System > Security campaign emails** and **Deploy key alert email**
     (if shown): turn them off.
   - Each box saves as you change it; there is no Save button.
2. **Watched repositories.** Open https://github.com/watching. For
   `jimhillr/monorepo` (and any other repository listed) the Watch level
   can stay as it is: with email off in step 1 it only feeds the web
   inbox. If a repository shows **Custom** with email ticked, set it to
   **Participating and @mentions**.
3. **Emails page.** Open https://github.com/settings/emails
   (Settings > Emails).
   - **Email preferences:** choose **Only receive account related
     emails, and those I subscribe to**, then **Save email preferences**.
     This stops GitHub's product and marketing email.
   - Click **Manage subscriptions** (it opens on github.com) and
     unsubscribe from every newsletter and product update listed.
4. **Organizations (only if the account is in one).** Settings >
   Notifications > **Custom routing** shows where each organization's
   email would go; with Email unticked in step 1 there is nothing to
   change. Skip it if the list is empty.
5. **GitHub Mobile (once, on your phone).** In GitHub Mobile, Profile >
   Settings > Notifications: make sure push is on for **Direct mentions**,
   **Review requests** and **Assigned**, for `jimhillr` (and any other
   account you add to the app). This is how urgent things now reach you
   instead of email.

## Where secrets go

None: no credential is made or changed.

## Verify

1. Each account's Notifications page shows no **Email** tick under
   Watching, Participating, Actions or Dependabot.
2. Over the next day, no GitHub email arrives for any of the three
   accounts except account security or billing mail, while
   github.com/notifications still fills up.
3. Tell any agent "done": it deletes
   [the human task](../issues/human/github-email-notifications-off.md).
   The settings check `github-email-notifications-off` re-asks every 180
   days, because GitHub sometimes adds new email types turned on.

## Undo or rotate

Tick **Email** again on the boxes you want under Settings >
Notifications, or choose **Receive all emails** on the Emails page. No
rotation: nothing here expires.
