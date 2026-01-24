
# Priority Issues

- TODO: Stop trying to run CodeQL workflow for rust. It never finishes. Either remove it or make it so that it only runs against projects that have changed.
- TODO: Running `cast ci` should not produce artifacts unless the `--release` flag is enabled. `cast ci` should use `--check` by default. `cast ci --release` should put the build artifacts in the correct subdirectory in artifacts for the build target they were built for. If artifacts are built, they should be committed via git lfs.

# Backlog

# Priority Projects
- cast
- cookbook

# On Hold Projects
- luggage
- starcraft