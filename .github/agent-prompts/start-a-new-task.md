# Start a new task prompt

## Task
Complete one issue within this repo that are identified with TODO or FIX comments.

## Instructions
1. Issues are defined as some comment beginning with TODO or FIX.
2. Complete only one issue.
3. When searching for issues, be eager. As soon as you find one, work on it. Prioritize issues found nearest to the top of a file. If the issue you find is too complex, try to reduce the scope and do a small part of it and whatever work remains add `TODO: from AI:` comments in the file that originated the issue.
4. Complete issues in ISSUES.md in the root of the repo before searching for other issues.
5. If step 4 has no available issues, then search for other ISSUES.md in the repo for an issue.
6. If step 5 has no available issues, then search the entire codebase for an issue.
4. Write tests to verify the issue.
5. Complete the issue by getting those tests to pass.
6. Write more tests you think are necessary to ensure full code coverage.
7. Run `cast projects --with-changes` to find projects with changes and then `cast ci` on each of those projects to ensure that everything meets standards
8. Update relevant documentation for the changes made.
9. If you think you have found problems with the codebase that are out of scope to fix, add a comment preceded with `TODO: from AI:` near the relevant problem.
10. To improve agent performance in the future, create or modify files in the repo to help agents (specifically github copilot agents)
11. Remove the TODO or FIX comment that you fixed.
12. Set the PR to ready for review. Don't leave it as draft.

## Context
This agent runs automatically after PRs created by GitHub Copilot are closed/merged.
