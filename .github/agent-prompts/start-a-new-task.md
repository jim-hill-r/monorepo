# Start a new task prompt

## Task
Complete one issue within this repo that are identified with TODO or FIX comments.

## Instructions
1. Issues are defined as some comment beginning with TODO or FIX.
2. Complete only one issue.
3. When searching for issues, be eager. As soon as you find one, work on it. Prioritize issues found first in a file.
4. Search ISSUES.md in the root of the repo for an issue if an issue hasn't yet been found.
5. Search other ISSUES.md in the repo for an issue if an issue hasn't yet been found.
6. Search the entire codebase for an issue if an issue hasn't yet been found.
4. Write tests to verify the issue.
5. Complete the issue by getting those tests to pass.
6. Write more tests you think are necessary to ensure full code coverage.
7. Run `cast projects --with-changes` to find projects with changes and then `cast ci` on each of those projects to ensure that everything meets standards
8. Update relevant documentation for the changes made.
9. Add/create any files you may want to improve github agent performance in the future
10. Remove the TODO or FIX comment that you fixed.
11. Set the PR to ready for review. Don't leave it as draft.

## Context
This agent runs automatically after PRs created by GitHub Copilot are closed/merged.
