# Claude AI Assistant

This document provides guidance on using Claude AI assistant with this monorepo.

## Overview

Claude is an AI assistant that can help with various development tasks in this repository, including:

- Code review and suggestions
- Documentation generation
- Bug investigation and debugging
- Refactoring assistance
- Test creation
- Architecture discussions

## Getting Started

When working with Claude on this monorepo:

1. Provide context about the specific project you're working on (e.g., `projects/cast_cli`, `projects/ciam`, `projects/blueeel`)
2. Share relevant file paths and code snippets
3. Be specific about what you need help with

## Best Practices

- **Be Specific**: Clearly describe the problem or task you need help with
- **Provide Context**: Include relevant code, error messages, or documentation
- **Iterative Approach**: Work incrementally, testing suggestions before moving forward
- **Review Suggestions**: Always review and understand AI-generated code before applying it

## Common Use Cases

### Code Review
Ask Claude to review your changes for:
- Code quality and best practices
- Potential bugs or edge cases
- Performance improvements
- Security considerations

### Documentation
Claude can help:
- Write or improve README files
- Generate inline code documentation
- Create API documentation
- Write user guides

### Debugging
Share error messages and relevant code for help with:
- Root cause analysis
- Suggested fixes
- Test case creation

## Repository-Specific Context

This is a Rust-based monorepo with multiple projects. Key information:

- **Primary Language**: Rust
- **Project Management**: Uses `cast_cli` tool for project operations
- **Session Management**: Start work sessions with `cast session start`
- **Project Creation**: Create new projects with `cast project new`
- **Issue Tracking**: Issues are tracked as TODO comments in the codebase

### GitHub Actions Workflows

- **start-a-new-task.yml**: Automatically triggers a new agent task when a PR from `copilot-swe-agent[bot]` is merged. Uses `gh agent-task create` to directly create agent tasks without creating GitHub issues.
- **pages.yml**: Deploys documentation from the `docs/` folder to GitHub Pages.

### Cast Session System

The Cast tooling includes a session tracking system:
- Sessions are stored in `.cast/sessions/` directory
- Each session creates a log file named with UUID v7 (time-based) and optional name: `{uuid}-{name}.log` or `{uuid}.log`
- UUID v7 ensures sessions are chronologically ordered when sorted alphabetically
- VSCode extension (`cast_vscode`) displays elapsed time for the most recent session
- To work with sessions: always read the **last** file in the sessions directory to get the most recent session

## Resources

- [Repository README](./README.md)
- [Issues](./ISSUES.md)
- [Documentation](./docs/)
