
# Priority Issues

- TODO: In the cast project, `cast ci` should run `cast publish` if all of the other steps are successful.
- TODO: In the cast project, `cast publish` should be running a release build for the given cast configuration and then copying it to the artifacts directory. Check that this is happening.
- TODO: In the cast project, `cast publish` should be zipping up the files from the release build to store as a single artifact if there is more than one file for the project_type. For example, dixous needs all the files zipped up.
- TODO: In the cast project, `cast ci` should commit the published artifact to the artifacts directory and ensure its using gitlfs for commiting large files.
- TODO: In the cast project, `cast toolchain install` improperly determined playwright as installed. The chromium drivers were not installed. Ensure chromium headless is installed when this command is run.
- TODO: In the cast project, `cast toolchain install` should always install gitlfs.
- TODO: Update codeQL GitHub workflows to only scan directories with changes. Leverage 'cast' to find directories with changes.

# Backlog

# Priority Projects
- cast
- cookbook

# On Hold Projects
- luggage