# Priority Issues

- TODO: Running `cast ci --only-changed --recursive 2` is pretty slow. Change the implementation to memoize the git diff s/t it should traverse the directories really fast when no changes are found.
- TODO: `cast ci --release` for project_type dixous should produce a zip file containing everything needed for a cloudflare static deploy. Since the target is typically wasm, use that for the subdirectory.
- TODO: `cast ci --recursive` should not fail the entire process if one project fails. It should note an error and then continue, providing a summary at the end of what failed.

# Backlog
