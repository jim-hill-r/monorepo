# observability user guide

## Who it's for

Jim, who gets paged when a production site or Worker breaks, and the
agents who build and fix the apps. It tells you when something in
production is down or getting worse, so it is fixed in minutes rather
than at the next nightly run.

Today this is planned work (decision 0023); the steps below describe how
it will work once phases 1 and 2 ship.

## Getting started

1. Install GitHub Mobile and sign in with your own GitHub account.
2. Turn on push notifications for mentions and assignments, and watch
   `jimhillr/monorepo` as "Participating and @mentions", so only
   high-urgency alerts push to your phone.
3. If you want night pages, let GitHub Mobile through your phone's Focus
   or Do Not Disturb settings.

Phase 2 adds a runbook with these steps and a test page to prove them.

## How to handle an alert

1. Open the push. It is a GitHub issue labelled `incident`, titled with
   the project and check that failed, such as `cookbook/home`.
2. Read what failed: the URL, the status or error count, and when it
   started.
3. Fix it, or ask an agent to, and record any follow-up work as an issue
   file in the project's `issues/` folder.
4. Leave the incident issue alone: the watchdog closes it when the check
   passes again.

## Getting help

- No push came for an incident you can see on GitHub: check GitHub
  Mobile's notification settings and your watch level, then your phone's
  Focus settings. The same notification also goes to your email.
- Too many alerts: ask for the rule to move to low urgency or be tuned;
  that change needs the `observability-reviewer`.
- Something else: ask in the project chat.
