# Human issues

Tasks that only a person can do: anything in a vendor dashboard, an account,
a secret or a payment setting, and sign-offs an agent may not give itself.
Agents never work these. A weekly scheduled task (the `human-issues-digest`
skill) brings the open ones to Jim's attention in the Claude project. The
process is in
[standards/docs/issue-management.md](../../standards/docs/issue-management.md#human-issues).

Each task is one file here, with three sections: the requirement, why it
can't be automated, and a link to its runbook in
[standards/docs/runbooks/](../../standards/docs/runbooks/README.md). Its
front matter says when it was added, what it unblocks and which runbook it
follows. Delete the file (in a PR)
once the task is done; any agent will do that when told it is done.
