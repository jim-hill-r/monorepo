---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The Rust crate in `learning/computer_science/algorithms/neet_code_150/` is
never built, linted or tested. It has no `mise.toml` of its own, so it isn't
in the root `config_roots`, and `athenaeum/mise.toml` uses the docs-only tasks
(`standards/tasks/docs.toml`), whose `ci` just echoes. Its `CONTRIBUTING.md`
still tells you to run `mise install` in a folder with no `mise.toml`.

On 2026-11-27 `cargo test` passed (7 tests), but `cargo clippy --all-targets`
reported 45 warnings: 19 needless `return`, 12 `unwrap()` (the crate warns on
`unwrap_used`), 4 `&Vec` arguments, dead code (`Tree`, `search_upper_bound`)
and `module_inception` in `binary_search`. `invert_binary_tree.rs` also sits
under `binary_search/` though it is a tree problem.

Give the crate a `mise.toml` that includes `standards/tasks/rust.toml`, add
it to `config_roots` in the root `mise.toml`, fix what `mise run ci` reports
(move `invert_binary_tree` to a `trees` module), and update its
`CONTRIBUTING.md`.
