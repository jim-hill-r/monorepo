# Issue Management Standards

This document defines the standards for managing issues, tasks, and TODOs within this monorepo.

## Issue Tracking Locations

Issues in this monorepo are tracked in multiple locations:

1. **ISSUES.md files** - Organized task lists in markdown format
2. **TODO comments** - In-code markers for specific issues
3. **FIX comments** - In-code markers for bugs or problems

## Finding Issues

When looking for work to do, check these locations in priority order:

1. **Root `/ISSUES.md`** - Check for priority issues and priority projects list
2. **Priority project ISSUES.md** - Check ISSUES.md files in priority projects (as listed in root ISSUES.md)
3. **Other ISSUES.md files** - Search for other ISSUES.md files throughout the repository
4. **TODO/FIX comments in code** - Search the codebase for TODO and FIX comments (skip items marked with `(agent-ignore)`)

### Note on Priority Issues with Subsections

Issues may be organized under subsections (using `##` or `###` headers) for complex epics. Look for TODO/FIX items within these subsections as well.

## ISSUES.md File Format

All ISSUES.md files should follow this structure:

```markdown
# Priority Issues

[List priority issues here]

# Backlog

[List backlog issues here]

# Priority Projects
- [project-name]
- [another-project]

# On Hold Projects
- [project-name]
```

**Important**: ISSUES.md files should have at least a `# Priority Issues` header even when empty.

## TODO Comment Conventions

When adding TODO comments in code, follow these conventions:

### Agent-Generated TODOs

Mark TODOs with `(agent-generated)` if created by an automated agent:

```rust
// TODO (agent-generated): Add validation for email format
```

### Agent-Ignore TODOs

Mark TODOs with `(agent-ignore)` if they should not be worked on by automated agents:

```rust
// TODO (agent-ignore): This requires manual architecture review
```

### Human TODOs

Tasks only a person can do go in the root `HUMAN_ISSUES.md` as
`TODO (human): ...` lines, never in a project's `ISSUES.md`. See
[Human issues](#human-issues).

### Removing Completed TODOs

**Always remove** TODO and FIX comments when the work is completed. Completed work should not leave behind TODO markers.

## Searching for References

When documenting files that need updates or searching for related code:

### For Code Files

Use `grep` to search non-markdown files, excluding build artifacts:

```bash
grep -r "pattern" --exclude-dir=.git --exclude-dir=target
```

### For Documentation

Use `grep` to search markdown files specifically:

```bash
grep -r "pattern" --include="*.md"
```

### For Dependencies

Find Cargo.toml files that reference a specific dependency:

```bash
find . -name "Cargo.toml" -exec grep -l "pattern" {} \;
```

**Important**: Always verify if metadata (like a `[package.metadata.*]` table) is just configuration vs actual dependencies.

## Creating New Issues

When creating issues that are too complex to tackle immediately:

1. Break the issue into smaller, manageable tasks
2. Document the tasks in the appropriate ISSUES.md file
3. Mark new TODOs with `(agent-generated)` if created by an agent
4. Use a flat list of TODO items under `# Priority Issues` in the root ISSUES.md
5. Only use subsections (`##`, `###`) if documenting a multi-phase epic in a project-specific ISSUES.md

### Example: Flat List Format (Root ISSUES.md)

```markdown
# Priority Issues
- TODO: Implement user authentication
- TODO: Add error handling to API endpoints
- TODO: Update documentation for new features
```

### Example: Epic Format (Project-Specific ISSUES.md)

```markdown
# Priority Issues

## Authentication System Epic

### Phase 1: Basic Auth
- TODO: Implement login endpoint
- TODO: Add JWT token generation

### Phase 2: OAuth Integration
- TODO: Add OAuth2 provider support
- TODO: Implement token refresh
```

## Working on Issues

When working on an issue:

1. **Check On Hold status** - Do not work on issues in projects listed under `# On Hold Projects`
2. **Start with tests** - Use Test-Driven Development when possible
3. **Make minimal changes** - Only change what's necessary to fix the issue
4. **Update documentation** - Update relevant docs for your changes
5. **Add out-of-scope TODOs** - If you find improvements that are out of scope, add a `TODO (agent-generated)` comment
6. **Remove completed TODOs** - Always remove the TODO/FIX comment when the work is done

## Best Practices

1. **Be specific** - Write clear, actionable TODO comments that explain what needs to be done
2. **Link to context** - Reference related issues, PRs, or documentation when helpful
3. **Prioritize appropriately** - Use Priority Issues for important work, Backlog for nice-to-haves
4. **Keep issues up-to-date** - Mark completed items, remove obsolete issues
5. **Use consistent formatting** - Follow the ISSUES.md format conventions

## Human issues

Some tasks only a person can do: anything in a vendor dashboard (Auth0,
Cloudflare, PostHog), creating an account or a secret, payment and billing
checks, and sign-offs an agent may not give itself. Agents can't do these, so
they must reach Jim rather than sit in an `ISSUES.md` or a chat thread
(decision [0002](decisions/0002-human-issues.md)).

### Where they live

- **One file.** Every such task is a line in the root
  [`HUMAN_ISSUES.md`](../../HUMAN_ISSUES.md) under `# Open`:

  ```markdown
  - TODO (human): In the Auth0 dashboard, add ... (Added 2026-11-25. Unblocks ...)
  ```

  Say what to do, where, and why, then when it was added and what it
  unblocks. `mise run //:audit` fails (HUM-001) if a `TODO (human)` line
  appears in any `ISSUES.md`.
- **PRs that need a person** get the `needs-human` label instead (see
  [pull-requests.md](pull-requests.md)). They are not copied into the file;
  the digest reads the label.
- **Questions in chat.** An agent that is blocked on something only Jim can
  do adds a `TODO (human)` line in its PR as well as asking in its thread, so
  the task survives the thread. A plain question Jim can answer in words
  stays in the thread.

### How they reach Jim

A weekly scheduled task in the Claude project runs the
[`human-issues-digest`](../../.agents/skills/human-issues-digest/SKILL.md)
skill. It posts one message listing the open `HUMAN_ISSUES.md` lines and the
open `needs-human` PRs, oldest first, with what each one unblocks and what
was added since the last digest.

### Closing one

Do the task, then remove its line in a PR (an agent will do this when told a
task is done). If the task unblocks a `TODO (agent-generated)` item, that item
becomes workable as usual.
