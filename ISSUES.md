
# Priority Issues

- TODO: Add a `cast ci --recursive 2` option that after running `cast ci` on the current directory it will look 2 levels below the current directory for other cast projects and run `cast ci --recursive 2` on them as well.
- TODO: In command `cast test`, if a project contains an npm `package.json`, ensure that `cast test` also runs `npm test`.
- TODO: Add `cast ci --only-changed` option that only runs the ci if the project contains git diffs back to the origin's default branch. Otherwise, it outputs that no diffs were found and doesn't run.
- TODO: Ensure that `cast ci` runs the `install` logic for each project before any other steps.

# Backlog

# Priority Projects
- cast
- cookbook

# On Hold Projects
- luggage