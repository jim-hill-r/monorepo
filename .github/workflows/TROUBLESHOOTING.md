# Troubleshooting GitHub Workflows

## A `ci (<project>)` job fails on `cargo fmt --all --check`

`mise run ci` checks formatting first. Run `mise run fix` (or `cargo fmt
--all`) in the project and commit the result.

## A `ci (<project>)` job fails with "dx and dioxus versions are incompatible"

`dx` must match the `dioxus` crate version. Set `"cargo:dioxus-cli"` in the
project's `mise.toml` to the dioxus version in its `Cargo.lock`.

## The `plan` job fails in "Audit standards"

`mise run //:audit` prints each error with its rule id (for example
`error[STD-004]`). The rules are listed at the top of
`standards/scripts/audit.sh`. The usual cause is a new project directory that
is missing its `mise.toml`, or one that is not listed in `config_roots` in the
root `mise.toml`.

## A deploy fails with an authentication error

`push-main.yml` deploys the cloudflare projects with `wrangler pages deploy`,
which needs the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` repository
secrets.
