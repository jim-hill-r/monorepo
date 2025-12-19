# Post-PR Review Agent Prompt

## Task
After a PR is merged, review the changes and identify any follow-up tasks or improvements that should be made.

## Instructions
1. Review the merged PR changes
2. Check for:
   - Any TODOs added in the code
   - Potential improvements or optimizations
   - Missing tests or documentation
   - Related files that might need updates
3. If any follow-up work is identified, create a new issue or update the ISSUES.md file

## Context
This agent runs automatically after PRs created by GitHub Copilot are closed/merged.
