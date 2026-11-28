# The Rust strict profile

What `chores quality` measures each project with (QUAL-402 and QUAL-501 in
[code-quality.md](../docs/code-quality.md), decision 0015):

- [profile.toml](profile.toml): the lint levels passed to clippy, the file
  length limit and the generated paths every check leaves out.
- [clippy.toml](clippy.toml): the thresholds, which clippy reads through
  `CLIPPY_CONF_DIR`. A project with its own `clippy.toml` (cahokia) keeps
  its own, so it copies these values.

Each project's counts live in its `docs/quality.toml`, written by
`chores quality --update` ([chores/README.md](../../chores/README.md)).
Changing either file here moves every project's counts, so do it only
together with the standard, and update the baselines with
`--update --accept-rise` in the same PR.
