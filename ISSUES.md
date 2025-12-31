
# Priority Issues

- TODO: Add three parameters to the `cast ci` command. The should be `cast ci --check`, `cast ci --fix`, and `cast ci --release`. The --check parameter is for use during a PR merge. It only does all the standard checks currently done by ci. The --fix parameter is for use from a local machine and will fix any issues the --check would fix that can be automatically fixed. The --release parameter does all the checks against build release and then publishes an artifact if everything passes. This is intended to run after a merge to master.
- TODO: Add `git lfs` automated install to `cast install` as a default tool. It can likely be downloaded via https://git-lfs.com/ for the specific detected platform.
- TODO: Add a 'cast uninstall' command to remove cast tooling

# Backlog

# Priority Projects
- cast
- cookbook

# On Hold Projects
- luggage