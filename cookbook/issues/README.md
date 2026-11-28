# Cookbook issues

One file per issue; the format is in
[standards/docs/issue-management.md](../../standards/docs/issue-management.md).

# Epic notes

All three epics come from the [cookbook roadmap](../docs/roadmap.md)
(2026-11-28). The content is the product and will become a book, so work
the epics in this order, and the phases of each in order.

## Content curation

Make the content model fit a book: a written recipe format with a status
for every recipe, then weekly themes and staples. Issues that change
`web/src/main.rs` wait for
[split-web-src-main-rs-1](split-web-src-main-rs-1.md) to merge.

## Management app

The private app where Jim and his wife, the cookbook's curators, log what
they cooked and edit recipes and plans. Named and built as a project's
management app in
[decision 0018](../../standards/docs/decisions/0018-feature-flags.md).
Phase 1 is a credential change (decision 0010). Phases 4 and 5 store data
where Jim's open storage decision in the roadmap says; until he answers,
build on the recommended option.

## Book

Progress toward the book and the manuscript export. Needs the management
app's status data.
