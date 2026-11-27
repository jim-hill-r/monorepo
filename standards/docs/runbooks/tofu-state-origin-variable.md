# Set the TOFU_STATE_ORIGIN variable

- **Kind:** one-time
- **Status:** open (`issues/human/tofu-state-origin-variable.md`). The Infra Mint workflow (PR #729) is expected to set it instead.
- **Time:** about 2 minutes
- **Cost:** $0

## What it's for

The infra plan, apply and drift workflows read and lock OpenTofu state at this URL (decision 0009). Until it is set, they skip every stack.

## Why a person

No workflow can write repository variables yet: the GITHUB_TOKEN can't, and the infra GitHub App that can isn't set up.

## Before you start

- Accounts: GitHub as `jimhillr`.
- Permissions: admin of `jimhillr/monorepo`.
- Deploy tofu_state has run once and is green.

## Steps

1. Open the latest green run of Actions > Deploy tofu_state, and copy the Worker URL from its job summary: `https://jimhillr-tofu-state.<subdomain>.workers.dev`.
2. In jimhillr/monorepo > Settings > Secrets and variables > Actions > Variables, add a new repository variable named `TOFU_STATE_ORIGIN` with that URL as its value.

## Where secrets go

None. The URL isn't a secret, which is why it is a variable.

## Verify

The next Infra Plan or Infra Apply run gets past `tofu init` for each stack that has its secrets.

## Undo or rotate

Edit or delete the variable on the same page. If the Worker ever moves, set the new URL here.
