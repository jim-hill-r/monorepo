# Get a free EIN and open a separate free account for project money

- **Kind:** one-time
- **Status:** open (cost/docs/issues/human/open-project-bank-account-and-ein.md)
- **Time:** about 60 minutes (10 for the EIN, the rest for the account)
- **Cost:** $0 (the EIN is free from the IRS; pick an account with no
  monthly fee and no minimum balance). Never pay a third-party site for an
  EIN.

## What it's for

[Decision 0016](../decisions/0016-holding-and-handling-money.md) keeps
project money in its own account, so every transaction in it belongs to the
public financial ledger and the monthly statement can be checked against
its balance without showing your personal spending. The EIN goes on the
GitHub Sponsors W-9 and the account application in place of your SSN.
Without it, Sponsors payouts (decision 0012 phase 2) land in your personal
account and the history is mixed for good.

## Why a person

The IRS and the bank need your identity, signature and identity checks.
No API does this, and no agent may hold a credential that can move money.

## Before you start

- Your SSN and address, for the IRS form (it stays with the IRS and the
  bank, never in the repo or a chat).
- The bank: your own bank or credit union, as a second free checking
  account (decision 0016, question 2).

## Steps

1. Go to irs.gov and search "Apply for an EIN online" (the free IRS
   assistant; open Monday to Friday, US daytime hours).
2. Choose **Sole proprietor**, then "Started a new business". Business
   name: your own name, or a trade name if you want one (`jimhillr` is
   fine as a trade name; some states want a DBA filing for it, so your own
   name is simplest).
3. Finish the form and **download the confirmation letter (CP 575)**. Keep
   it with your personal records, not in the repo.
4. At the bank you chose, open a **business checking account as a sole
   proprietor** with the EIN. Pick the option with no monthly fee and no
   minimum balance. Decline overdraft lines, credit cards and any paid
   add-on.
5. Turn on two-factor sign-in and statement emails (PDF statements).
6. Don't link it to any app or bank-feed service. Agents never get a
   sign-in or token for it.
7. Tell the thread "done" with only: the bank's name and the last four
   digits of the account. Nothing else.

## Where secrets go

None. The EIN letter, account and routing numbers and the bank sign-in
stay with you. None of them goes in git, OpenTofu state, a GitHub secret,
a chat or a note in the repo. The ledger refers to the account only as
`bank:checking`.

## Verify

- The account shows a $0 balance with no monthly fee in its fee schedule.
- An agent adds the `project-bank-account` settings check to
  `standards/settings.toml` (payout destination still this account, no
  overdraft or card added) and deletes the issue file.
- When you set up GitHub Sponsors (0012 phase 2), its payout account is
  this one and the W-9 uses the EIN.

## Undo or rotate

Close the account at the bank once it is empty, and tell the thread so
the settings check goes. An EIN can't be deleted, but the IRS can close the
business account for it by letter if it is never used.
