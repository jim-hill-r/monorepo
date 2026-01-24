# Priority Issues

- TODO: Convert the `Cast CI` workflow to be called `Pull Request CI` such that it only runs on PRs and then add another duplicate workflow called `Trunk CI` that runs on any commit to Main. The PR workflow should use `cast ci --only-changed --check --recursive 2` and the trunk workflow should use `cast ci --only-changed --release --recursive 2`. Also, the trunk workflow should not run if the only change is a new file in a subdirectory of any artifacts folder.
- TODO: Convert the `Cast CD` workflow to be called `CD`. It should only run when an new linux build appears in an artifacts directory. It should run `cast cd` on the parent project of that artifacts directory.
- TODO: Running `cast ci --only-changed --recursive 2` is pretty slow. Change the implementation to memoize the git diff s/t it should traverse the directories really fast when no changes are found.
- TODO: `cast ci --release` for project_type dixous should produce a zip file containing everything needed for a cloudflare static deploy. Since the target is typically wasm, use that for the subdirectory.
- TODO: `cast ci --recursive` should not fail the entire process if one project fails. It should note an error and then continue, providing a summary at the end of what failed.

# Backlog
