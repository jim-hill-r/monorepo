# Start a new task prompt

## Task
Complete one issue within this repo.

## Instructions
1. Find one issue per the Find Issue section below.
2. Complete just that one issue per the Fix Issue section below.

## Find Issue
1. Be eager. While following these steps, just work the first issue you find.
2. Issues are defined as some comment beginning with TODO or FIX.
3. Skip any TODO or FIX comments that are marked with `(agent-ignore)` - these should not be worked on.
4. Read ISSUES.md in the root of the repo.
5. If there are issues in the `# Priority Issues` sections of the root ISSUES.md, prioritize the first issues found in the list.
6. If there are projects listed in the `# Priority Projects` sections, then prioritize issues found in the associated ISSUES.md for that project. Prioritize the first project found in the list.
7. If steps 5 and 6 don't have issues, then search for other ISSUES.md in the repo for an issue.
8. If steps 5, 6, and 7 don't have issues, then search then entire codebase for an issue.
9. If no issues are found, do any investigation and find some issues and add them to an appropriate ISSUES.md for future consideration.

## Fix Issue
1. If the issue to be worked is too complex, then just try to break up the issue into several smaller issues and issue a PR with changes to any relevant ISSUES.md. Mark these comments with `TODO (agent-generated)`.
2. Otherwise do the following:
3. Use Test Driven Development as much as possible. Start by writing tests for the issue you are trying to fix.
4. Implement code that will make the tests pass and fix this issue.
5. Analyze the new code to determine if more tests are needed and add them.
6. Run `cast ci` against any projects that have been changed.
7. Update relevant documentation for the changes made.
8. While do this work, if you find improvements to the codebase that are out of scope for this task, then add a comment preceded with `TODO: (agent-generated)` near the relevant problem.
9. Add information to `copilot-instructions` or any other relevant file that will help future agents to work in this repo more easily.
10. Remove the TODO or FIX comment that you fixed.

## Context
This agent runs automatically after PRs created by GitHub Copilot are closed/merged.
