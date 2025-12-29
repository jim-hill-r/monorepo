# Priority Issues

## Cast Workspace Restructuring (Epic)
This is a complex restructuring task that needs to be broken down into sequential steps. Each step must be completed and tested before moving to the next.

### Phase 1: Preparation and Planning (agent-generated)
- ✓ COMPLETED: Document all current dependencies and references to cast, cast_cli, and cast_vscode projects (See WORKSPACE_RESTRUCTURING_DEPENDENCIES.md)
- ✓ COMPLETED: Create a detailed migration plan with rollback steps (See WORKSPACE_RESTRUCTURING_MIGRATION_PLAN.md)
- ✓ COMPLETED: Identify all files that need updates (workflows, dependabot, tests, docs, etc.) - See comprehensive checklist in WORKSPACE_RESTRUCTURING_MIGRATION_PLAN.md Phase 1.3

### Phase 2: Create Workspace Structure (agent-generated)
- ✓ COMPLETED: Create new cast workspace root with Cargo.toml containing [workspace] configuration
- ✓ COMPLETED: Add workspace-level Cast.toml, README.md, and .gitignore
- ✓ COMPLETED: Add workspace ISSUES.md (move current issues to workspace root)

### Phase 3: Rename and Move cast to cast_core (agent-generated)
- ✓ COMPLETED: Rename cast package to cast_core in its Cargo.toml
- ✓ COMPLETED: Create cast_workspace/core directory structure
- ✓ COMPLETED: Move cast_core files to cast_workspace/core directory
- ✓ COMPLETED: Update cast_cli dependency to point to cast_core at new path
- ✓ COMPLETED: Test that cast_cli still builds with cast_core

### Phase 4: Move cast_cli to Workspace (agent-generated)
- ✓ COMPLETED: Create cast_workspace/cli directory structure
- ✓ COMPLETED: Move cast_cli files to cast_workspace/cli directory
- ✓ COMPLETED: Update GitHub workflows to build from new location (cast_workspace/cli)
- ✓ COMPLETED: Update workflow tests to use new paths
- ✓ COMPLETED: Test that workflows can still build the CLI

### Phase 5: Move cast_vscode to Workspace (agent-generated)
- ✓ COMPLETED: Create cast_workspace/vscode_ext directory structure
- ✓ COMPLETED: Move cast_vscode files to cast_workspace/vscode_ext directory
- ✓ COMPLETED: Update any references in documentation

### Phase 6: Update Configuration Files (agent-generated)
- ✓ COMPLETED: Update dependabot.yml to reference new workspace structure
- ✓ COMPLETED: Update REPOSITORY_STRUCTURE.md
- ✓ COMPLETED: Update workflow documentation
- ✓ COMPLETED: Update cast workspace README.md with new structure

### Phase 7: Testing and Validation (agent-generated)
- ✓ COMPLETED: Run `cast ci` on all moved projects (cast_workspace/core and cast_workspace/cli both pass)
- TODO (agent-generated): Test GitHub workflows in a test PR (will be tested when this PR is created)
- ✓ COMPLETED: Verify all tests pass (all tests passing for cast_core and cast_cli)
- ✓ COMPLETED: Update copilot-instructions with workspace structure patterns

### Phase 8: Cleanup (agent-generated)
- TODO (agent-generated): Remove old cast, cast_cli, cast_vscode directories
- TODO (agent-generated): Remove these TODO items from ISSUES.md once complete

# Backlog
TODO: Refactor commands to using executor command pattern per [blog post.](https://medium.com/@robjsliwa_71070/crafting-cli-with-oauth-2-0-authentication-multi-tenant-todo-server-in-rust-series-eaa0af452a56)

