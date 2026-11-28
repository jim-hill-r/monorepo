# 0007 note: the terrars spike

- Date: 2026-11-26
- For: [decision 0007](0007-infrastructure-as-code.md), item 2
- Result: **adopt `terrars` (A1)**, generating only the resource types we use.

## What was tried

The `infra` project generates typed bindings for `auth0/auth0` 1.63.0 with
`terrars-generate` 0.4.0 and OpenTofu 1.13.10, builds the `auth0` stack from
them, and checks the output with `tofu validate`. The same was done for the
whole Auth0 provider and the whole Cloudflare v5 provider (5.29.0), to see
how far the approach scales.

Measured in a 4-core cloud sandbox (debug builds, dependencies already
compiled unless noted):

| Bindings | Generated Rust | Generate | `cargo build` | `cargo clippy` |
| --- | --- | --- | --- | --- |
| Auth0, `client` only (what `infra` uses) | 8,000 lines | under 1 s | 30 s for the whole crate from clean, 1 s incremental | 1 s |
| Auth0, whole provider | 123,000 lines | 2 s | 4 s | - |
| Cloudflare v5, whole provider (790 resources and data sources) | 453,000 lines | 7 s | 59 s | 98 s |

`tofu validate` accepts the synthesized `out/auth0/main.tf.json`. GitHub CI
times for the `infra` job are in the PR that added this note.

So the whole Cloudflare provider would add one to two minutes to every
`infra` CI run, mostly clippy. Generating only what we use (the `include`
list in `infra/terrars_<provider>.json`) keeps each provider to seconds, and
adding a resource type is one line and a regenerate.

## Bugs found, fixed upstream

Jim's rule is that `terrars` bugs are fixed upstream (decision 0007, item 2).
The fixes are in `infra/upstream/` as `git format-patch` files, ready to open
as pull requests on `andrewbaxter/terrars`. Agents can't reach that
repository, so opening them is a human task
(`infra/docs/issues/human/open-terrars-upstream-prs.md`).

1. **`terrars-generate` fails with OpenTofu.** With `"binary": "tofu"`, the
   schema is keyed `registry.opentofu.org/<vendor>/<name>`, but the generator
   only looked for `registry.terraform.io/...` and stopped with "Missing
   provider schema for listed provider". Fix:
   `terrars-generate-opentofu-schema-key.patch`. This only affects generating
   bindings, which is done by hand when a provider version changes, so `infra`
   doesn't need a git dependency for it: the committed bindings were
   generated with the patched generator.
2. **No `import` blocks.** `Stack` has `add_removed` but no way to write the
   `import` blocks every existing resource needs (decision 0007, item 11).
   Fix: `terrars-import-blocks.patch`, which adds `Stack::add_import`. Until
   it is released, `infra/src/synth.rs` adds the blocks to the JSON after
   `Stack::serialize`, in one small function marked for removal.

Not a bug, but noisy: generated resources serialize every dynamic block as
`null` and the full `lifecycle` block. OpenTofu ignores them, so it is left
alone for now.

## Choices made during the spike

- **One directory per stack**, `infra/out/<stack>/main.tf.json`, instead of
  `infra/out/<stack>.tf.json`. OpenTofu treats a directory as one module with
  one backend, so stacks in one directory would share a state and need every
  vendor's credentials for any plan. One directory each keeps a PR plan to the
  credentials of the stack it changes.
- **The provider lock file is committed** next to each stack
  (`out/<stack>/.terraform.lock.hcl`), with the `zh:` hashes of every
  platform from the provider's release checksums and the `h1:` hash for
  `linux_amd64`, so CI runs `tofu init -lockfile=readonly`.
- **The generated bindings are committed** under `infra/src/tfschema/`, as
  the terrars README recommends, so building `infra` needs no OpenTofu or
  provider download. They are source, not build output.

## What the spike couldn't check

The sandbox has no Auth0 credentials (`issues/human/auth0-iac-credentials.md`
is open), so no plan has run against the real tenant. The first PR plan, in
the workflows issue, must show only the intended changes. Two things to look
for there:

- The application names in `infra/src/stacks/mod.rs` (`cookbook`, `cahokia`)
  are guesses; a rename in the plan means the code should use the real name.
- Attributes we don't set keep their imported values only when the provider
  marks them computed. Any other change in the first plan gets set in code to
  the current value.
