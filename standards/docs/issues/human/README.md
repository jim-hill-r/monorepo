# Human issues

Tasks that only a person can do: anything in a vendor dashboard, an account,
a secret or a payment setting, and sign-offs an agent may not give itself.
Agents never work these. A weekly scheduled task (the `human-issues-digest`
skill) brings the open ones to Jim's attention in the Claude project. The
process is in
[standards/docs/issue-management.md](../../issue-management.md#human-issues).

Each project keeps its own human tasks in its `docs/issues/human/`; this
folder holds the repo-wide ones (settings checks, GitHub account settings).
Pane's Human issues page and `mise run //:issues -- --human` gather them
from every project. Each task is one file, with three sections: the requirement, why it
can't be automated, and a link to its runbook in
[standards/docs/runbooks/](../../runbooks/README.md). Its
front matter says when it was added, what it unblocks and which runbook it
follows. Delete the file (in a PR)
once the task is done; any agent will do that when told it is done.
