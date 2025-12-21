# Start a new task prompt

## Task
Complete one issue within this repo that are identified with TODO or FIX comments.

## Instructions
1. Issues are defined as some comment beginning with TODO or FIX.
2. Skip any TODO or FIX comments that are marked with "(ignore)" - these should not be worked on.
3. Complete only one issue.
4. When searching for issues, be eager. As soon as you find one, work on it. Prioritize issues found nearest to the top of a file. If the issue you find is too complex, try to reduce the scope and do a small part of it and whatever work remains add `TODO: from AI:` comments in the file that originated the issue.
5. Complete issues in ISSUES.md in the root of the repo before searching for other issues.
6. If step 5 has no available issues, then search for other ISSUES.md in the repo for an issue.
7. If step 6 has no available issues, then search the entire codebase for an issue.
8. Write tests to verify the issue.
9. Complete the issue by getting those tests to pass.
10. Write more tests you think are necessary to ensure full code coverage.
11. Run `cast projects --with-changes` to find projects with changes and then `cast ci` on each of those projects to ensure that everything meets standards
12. Update relevant documentation for the changes made.
13. If you think you have found problems with the codebase that are out of scope to fix, add a comment preceded with `TODO: from AI:` near the relevant problem.
14. To improve agent performance in the future, create or modify files in the repo to help agents (specifically github copilot agents)
15. Remove the TODO or FIX comment that you fixed.

## Context
This agent runs automatically after PRs created by GitHub Copilot are closed/merged.
