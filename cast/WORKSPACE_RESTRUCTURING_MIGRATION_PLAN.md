# Cast Workspace Restructuring - Detailed Migration Plan with Rollback Steps

This document provides detailed step-by-step instructions for migrating the cast, cast_cli, and cast_vscode projects into a unified workspace structure (a Cargo workspace containing all Cast-related projects under a single directory with shared configuration and dependencies), including rollback procedures for each phase.

**Created**: 2025-12-27  
**Status**: Planning Phase  
**Related Documents**: 
- `WORKSPACE_RESTRUCTURING_DEPENDENCIES.md` - Current state documentation
- `ISSUES.md` - Epic tracking and task breakdown

## Prerequisites

Before beginning any phase:
1. Ensure all changes are committed and pushed
2. Create a backup branch: `git checkout -b backup/pre-workspace-restructure`
3. Verify all existing tests pass: `cast ci` on cast and cast_cli
4. Verify GitHub workflows are passing
5. Document current commit SHA for rollback reference

## Migration Phases

Each phase includes:
- **Objective**: What the phase accomplishes
- **Prerequisites**: What must be true before starting
- **Steps**: Detailed execution steps
- **Validation**: How to verify success
- **Rollback**: How to undo if problems occur

---

## Phase 1: Preparation and Planning

### Phase 1.1: Document Current Dependencies ✅ COMPLETED
See `WORKSPACE_RESTRUCTURING_DEPENDENCIES.md`

### Phase 1.2: Create Migration Plan ✅ COMPLETED
This document.

### Phase 1.3: Identify All Files Requiring Updates

**Objective**: Create a comprehensive checklist of all files that will need updates during the migration.

**Prerequisites**:
- WORKSPACE_RESTRUCTURING_DEPENDENCIES.md exists
- This migration plan exists

**Steps**:
1. Review WORKSPACE_RESTRUCTURING_DEPENDENCIES.md "Summary of Files Requiring Updates"
2. Search codebase for additional references:
   ```bash
   # Search for cast_cli references
   grep -r "cast_cli" --exclude-dir=.git --exclude-dir=target
   
   # Search for cast_vscode references
   grep -r "cast_vscode" --exclude-dir=.git --exclude-dir=target
   
   # Search for "../cast" path references
   grep -r "\.\./cast" --exclude-dir=.git --exclude-dir=target
   ```
3. Document any additional references found
4. Create a tracking checklist in this document

**Validation**:
- All references are documented
- No surprises during later phases

**Rollback**: N/A (documentation only)

**Search Results Completed**: 2025-12-27

#### Comprehensive File Update Checklist

Based on thorough codebase search, the following files will need updates during the migration:

##### Non-Markdown Code and Configuration Files (11 files)

1. **cast_cli/Cargo.toml** (Phase 3)
   - Update: `cast = { path = "../cast" }` → `cast_core = { path = "../cast_workspace/core" }`
   - Status: ⏳ Pending

2. **.github/workflows/cast-ci.yml** (Phase 4)
   - Line ~38-41: Build step `cd cast_cli` → `cd cast_workspace/cli`
   - Line ~75: `CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"` → `cast_workspace/cli/target/release/cast`
   - Line ~106: `CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"` → `cast_workspace/cli/target/release/cast`
   - Status: ⏳ Pending

3. **.github/workflows/cast-cd.yml** (Phase 4)
   - Line ~31-33: Build step `cd cast_cli` → `cd cast_workspace/cli`
   - Line ~67: `CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"` → `cast_workspace/cli/target/release/cast`
   - Line ~98: `CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"` → `cast_workspace/cli/target/release/cast`
   - Status: ⏳ Pending

4. **.github/dependabot.yml** (Phase 6)
   - Lines 6-20: `directory: "/projects/cast_cli"` → `directory: "/cast_workspace"`
     - Note: Currently incorrect path (should be `/cast_cli` but will become `/cast_workspace`)
   - Lines 22-36: `directory: "/projects/cast"` → Remove (consolidated into workspace)
     - Note: Currently incorrect path (should be `/cast` but will be removed)
   - Labels: Update from separate "cast_cli" and "cast" to "cast_workspace"
   - Status: ⏳ Pending

5. **monorepo/workflow_tests/src/lib.rs** (Phase 4)
   - Function: `get_cast_cli_cargo_path()` 
   - Update: `get_repo_root().join("cast_cli/Cargo.toml")` → `"cast_workspace/cli/Cargo.toml"`
   - Status: ⏳ Pending

6. **monorepo/workflow_tests/tests/cast_ci_workflow_tests.rs** (Phase 4)
   - Test assertions checking for "cast_cli" string in workflow
   - Update test expectations to look for "cast_workspace/cli"
   - Functions affected:
     - `test_workflow_uses_cast_cli_to_detect_changes()`
     - `test_workflow_builds_cast_cli()`
     - `test_cast_cli_project_exists()`
   - Status: ⏳ Pending

7. **monorepo/workflow_tests/tests/cast_cd_workflow_tests.rs** (Phase 4)
   - Test assertions checking for "cast_cli" string in workflow
   - Update test expectations to look for "cast_workspace/cli"
   - Functions affected:
     - `test_workflow_uses_cast_cli_to_detect_changes()`
     - `test_workflow_builds_cast_cli()`
   - Status: ⏳ Pending

8. **cast_cli/Cargo.lock** (Phase 4)
   - Will be auto-updated when Cargo.toml is changed
   - Status: 🤖 Auto-updated

9. **cast/Cargo.toml** (Phase 3)
   - Package name: `name = "cast"` → `name = "cast_core"`
   - Status: ⏳ Pending

10. **cast/Cargo.lock** (Phase 3)
    - Will be auto-updated when package name changes
    - Status: 🤖 Auto-updated

##### Files Requiring No Changes (2 files)

11. **cookbook/web/Cargo.toml**
    - Contains: `[package.metadata.cast]` - Just metadata, not a dependency
    - Status: ✅ No action needed

12. **cahokia/web/Cargo.toml**
    - Contains: `[package.metadata.cast]` - Just metadata, not a dependency
    - Status: ✅ No action needed

##### Markdown Documentation Files (5 files)

13. **README.md** (Phase 5)
    - Line ~12: `cargo install --path ./cast_cli` → `./cast_workspace/cli`
    - Line ~12: `./cast_vscode/cast.vsix` → `./cast_workspace/vscode_ext/cast.vsix`
    - Status: ⏳ Pending

14. **REPOSITORY_STRUCTURE.md** (Phase 6)
    - Line ~21: Update structure diagram for cast_workspace
    - Line ~22: Remove separate cast_cli entry
    - Lines 71-80: Update Cast CLI section with new paths
    - Status: ⏳ Pending

15. **.github/workflows/README.md** (Phase 6)
    - Line ~18: Update description "Builds the `cast` CLI from `cast_cli`"
    - Line ~30: Update "The `cast_cli` project must be buildable"
    - Line ~83: Update description (duplicate)
    - Line ~95: Update requirement (duplicate)
    - Status: ⏳ Pending

16. **.github/workflows/TROUBLESHOOTING.md** (Phase 6)
    - Line ~34: Update example path `cast_cli/src/bin/cast.rs` → `cast_workspace/cli/src/bin/cast.rs`
    - Status: ⏳ Pending

17. **macos/README.md** (Phase 5)
    - Line ~3: `cargo install --path ./cast_cli` → `./cast_workspace/cli`
    - Status: ⏳ Pending

18. **cast/ISSUES.md** (Phase 8)
    - Mark all Phase 1-8 TODOs as complete
    - Remove completed TODO items
    - Status: ⏳ Pending

##### Summary Statistics

- **Total Files Requiring Updates**: 18 files
  - Code/Config files needing manual updates: 7 files
  - Auto-updated files (Cargo.lock): 2 files
  - Documentation files: 5 files
  - Workspace migration docs: 1 file (cast/ISSUES.md)
  - No action needed: 2 files (just metadata)
  - Excludes the workspace restructuring documentation itself
  
- **Files by Phase**:
  - Phase 3 (cast → cast_core): 2 files + 1 auto-update
  - Phase 4 (cast_cli → workspace/cli): 5 files + 1 auto-update
  - Phase 5 (cast_vscode → workspace/vscode_ext): 2 files
  - Phase 6 (Configuration/Documentation): 4 files
  - Phase 8 (Cleanup): 1 file

- **Critical Path Files** (workflows that must work):
  - .github/workflows/cast-ci.yml
  - .github/workflows/cast-cd.yml
  - monorepo/workflow_tests/* (3 files)

##### Additional Notes

1. **No script files found** containing cast_cli or cast_vscode references
2. **No VSCode settings files** (.vscode/) contain cast references
3. **No CODEOWNERS or PR templates** contain cast references
4. **.github/WORKFLOW_CONVENTIONS.md** does not contain cast_cli or cast_vscode references
5. **dependabot.yml current state**: Already has incorrect paths ("/projects/cast_cli" and "/projects/cast" instead of "/cast_cli" and "/cast")

---

## Phase 2: Create Workspace Structure

**Objective**: Create the new workspace root directory structure without moving any existing code.

**Prerequisites**:
- Phase 1 complete
- Clean git working directory
- Current tests passing

**Steps**:

1. Create workspace root directory:
   ```bash
   cd /home/runner/work/monorepo/monorepo
   mkdir -p cast_workspace
   ```

2. Create workspace Cargo.toml:
   ```bash
   cat > cast_workspace/Cargo.toml << 'EOF'
   [workspace]
   resolver = "2"
   members = [
       "core",
       "cli",
   ]
   
   [workspace.package]
   version = "0.1.0"
   edition = "2021"
   authors = ["Cast Contributors"]
   license = "MIT"
   
   [workspace.dependencies]
   # Common dependencies will be added as we migrate
   EOF
   ```

3. Create workspace Cast.toml:
   ```bash
   cat > cast_workspace/Cast.toml << 'EOF'
   [project]
   name = "cast"
   type = "workspace"
   description = "Cast monorepo tooling workspace"
   EOF
   ```

4. Create workspace README.md:
   ```bash
   cat > cast_workspace/README.md << 'EOF'
   # Cast Workspace
   
   This workspace contains the Cast monorepo tooling projects.
   
   ## Projects
   
   - **core** (`cast_core`) - Core Cast library for monorepo operations
   - **cli** (`cast_cli`) - Cast command-line interface
   - **vscode_ext** (`cast_vscode`) - VSCode extension for Cast (to be migrated)
   
   ## Building
   
   Build all workspace members:
   ```bash
   cargo build --workspace
   ```
   
   Build specific member:
   ```bash
   cargo build -p cast_core
   cargo build -p cast_cli
   ```
   
   ## Testing
   
   Test all workspace members:
   ```bash
   cargo test --workspace
   ```
   
   ## Development
   
   The Cast CLI is the primary development tool:
   ```bash
   cd cli
   cargo build --release
   ./target/release/cast --help
   ```
   EOF
   ```

5. Create workspace .gitignore:
   ```bash
   cat > cast_workspace/.gitignore << 'EOF'
   target/
   Cargo.lock
   **/*.rs.bk
   .DS_Store
   EOF
   ```

6. Create workspace ISSUES.md (copy from cast/ISSUES.md):
   ```bash
   cp cast/ISSUES.md cast_workspace/ISSUES.md
   ```

7. Create placeholder directories:
   ```bash
   mkdir -p cast_workspace/core
   mkdir -p cast_workspace/cli
   mkdir -p cast_workspace/vscode_ext
   ```

**Validation**:
- Directories exist: `cast_workspace/`, `cast_workspace/core/`, `cast_workspace/cli/`
- Files exist: workspace Cargo.toml, Cast.toml, README.md, .gitignore, ISSUES.md
- No existing projects are affected
- Git status shows new untracked files only

**Rollback**:
```bash
# Simple: just remove the workspace directory
rm -rf cast_workspace
git status  # Verify no tracked files affected
```

---

## Phase 3: Rename and Move cast to cast_core

**Objective**: Move the cast library into the workspace as cast_core.

**Prerequisites**:
- Phase 2 complete
- Workspace structure exists
- Current cast tests passing
- Clean git working directory

**Steps**:

1. Copy cast to workspace core directory:
   ```bash
   cp -r cast/* cast_workspace/core/
   # Verify copy
   ls -la cast_workspace/core/
   ```

2. Update cast_workspace/core/Cargo.toml - change package name:
   ```toml
   [package]
   name = "cast_core"  # Changed from "cast"
   version = "0.1.0"
   edition = "2021"
   # ... rest remains same
   ```

3. Remove cast_workspace/core/ISSUES.md (already in workspace root):
   ```bash
   rm cast_workspace/core/ISSUES.md
   ```

4. Test that cast_core builds:
   ```bash
   cd cast_workspace/core
   cargo build
   cargo test
   cd ../..
   ```

5. Update cast_cli to use new cast_core:
   
   Edit `cast_cli/Cargo.toml`:
   ```toml
   [dependencies]
   cast_core = { path = "../cast_workspace/core" }  # Changed from cast
   # Note: Keep old cast dependency commented for rollback
   # cast = { path = "../cast" }
   ```

6. Update cast_cli imports:
   ```bash
   # Find and replace in cast_cli/src
   find cast_cli/src -type f -name "*.rs" -exec sed -i 's/use cast::/use cast_core::/g' {} +
   find cast_cli/src -type f -name "*.rs" -exec sed -i 's/extern crate cast/extern crate cast_core/g' {} +
   ```

7. Test that cast_cli builds with cast_core:
   ```bash
   cd cast_cli
   cargo build
   cargo test
   cd ..
   ```

8. Commit the changes:
   ```bash
   git add cast_workspace/core
   git add cast_cli/Cargo.toml
   git add cast_cli/src
   git commit -m "Phase 3: Add cast_core to workspace and update cast_cli dependency"
   ```

**Validation**:
- `cast_workspace/core` contains all cast library code
- Package name in cast_workspace/core/Cargo.toml is "cast_core"
- `cast_cli` builds successfully using cast_core
- All cast_cli tests pass
- `cargo build` in cast_workspace/core succeeds
- `cargo test` in cast_workspace/core succeeds

**Rollback**:
```bash
# Revert cast_cli changes
cd cast_cli
git restore Cargo.toml
find src -type f -name "*.rs" -exec git restore {} +
cargo build  # Verify it builds with old cast

# Remove workspace core directory
cd ..
rm -rf cast_workspace/core/*

# Verify cast_cli works
cd cast_cli
cargo test
```

---

## Phase 4: Move cast_cli to Workspace

**Objective**: Move cast_cli into the workspace structure.

**Prerequisites**:
- Phase 3 complete
- cast_core in workspace and building
- cast_cli using cast_core successfully
- Clean git working directory

**Steps**:

1. Copy cast_cli to workspace cli directory:
   ```bash
   cp -r cast_cli/* cast_workspace/cli/
   ls -la cast_workspace/cli/
   ```

2. Update cast_workspace/cli/Cargo.toml - fix cast_core path:
   ```toml
   [dependencies]
   cast_core = { path = "../core" }  # Updated path for workspace
   ```

3. Test that workspace CLI builds:
   ```bash
   cd cast_workspace
   cargo build -p cast_cli
   cargo test -p cast_cli
   cd ..
   ```

4. Update GitHub workflow: cast-ci.yml
   
   Before:
   ```yaml
   - name: Build cast CLI
     run: |
       cd cast_cli
       cargo build --release
   ```
   
   After:
   ```yaml
   - name: Build cast CLI
     run: |
       cd cast_workspace/cli
       cargo build --release
   ```
   
   Update all CAST_BIN references:
   - Line ~75: `CAST_BIN="$GITHUB_WORKSPACE/cast_workspace/cli/target/release/cast"`
   - Line ~106: `CAST_BIN="$GITHUB_WORKSPACE/cast_workspace/cli/target/release/cast"`

5. Update GitHub workflow: cast-cd.yml
   
   Similar changes:
   ```yaml
   - name: Build cast CLI
     run: |
       cd cast_workspace/cli
       cargo build --release
   ```
   
   Update CAST_BIN references:
   - Line ~67: `CAST_BIN="$GITHUB_WORKSPACE/cast_workspace/cli/target/release/cast"`
   - Line ~98: `CAST_BIN="$GITHUB_WORKSPACE/cast_workspace/cli/target/release/cast"`

6. Update workflow tests: monorepo/workflow_tests/src/lib.rs
   
   Before:
   ```rust
   pub fn get_cast_cli_cargo_path() -> PathBuf {
       get_repo_root().join("cast_cli/Cargo.toml")
   }
   ```
   
   After:
   ```rust
   pub fn get_cast_cli_cargo_path() -> PathBuf {
       get_repo_root().join("cast_workspace/cli/Cargo.toml")
   }
   ```

7. Update workflow test assertions:
   - `monorepo/workflow_tests/tests/cast_ci_workflow_tests.rs`
   - `monorepo/workflow_tests/tests/cast_cd_workflow_tests.rs`
   
   Update expected paths from "cast_cli" to "cast_workspace/cli"

8. Test workflow tests:
   ```bash
   cd monorepo/workflow_tests
   cargo test
   cd ../..
   ```

9. Commit the changes:
   ```bash
   git add cast_workspace/cli
   git add .github/workflows/cast-ci.yml
   git add .github/workflows/cast-cd.yml
   git add monorepo/workflow_tests
   git commit -m "Phase 4: Move cast_cli to workspace and update workflows"
   ```

**Validation**:
- `cast_workspace/cli` contains cast_cli code
- CLI builds in workspace: `cd cast_workspace && cargo build -p cast_cli`
- Workflow files reference new paths
- Workflow tests pass: `cd monorepo/workflow_tests && cargo test`
- Original cast_cli still exists (not deleted yet)

**Rollback**:
```bash
# Revert workflow changes
git restore .github/workflows/cast-ci.yml
git restore .github/workflows/cast-cd.yml
git restore monorepo/workflow_tests

# Remove workspace cli
rm -rf cast_workspace/cli/*

# Verify original cast_cli works
cd cast_cli
cargo build
cargo test

# Test workflows would work (local simulation)
cd cast_cli
cargo build --release
test -f target/release/cast && echo "CLI binary exists"
```

---

## Phase 5: Move cast_vscode to Workspace

**Objective**: Move cast_vscode into the workspace structure.

**Prerequisites**:
- Phase 4 complete
- cast_core and cast_cli in workspace
- Workflows updated and tested
- Clean git working directory

**Steps**:

1. Copy cast_vscode to workspace vscode_ext directory:
   ```bash
   cp -r cast_vscode/* cast_workspace/vscode_ext/
   ls -la cast_workspace/vscode_ext/
   ```

2. Verify cast_vscode has no Rust dependencies to update:
   ```bash
   # Check for any Cargo.toml
   find cast_workspace/vscode_ext -name "Cargo.toml"
   # Should find none or only cast_workspace/vscode_ext/Cast.toml
   ```

3. Test VSCode extension builds (if applicable):
   ```bash
   cd cast_workspace/vscode_ext
   # Add build commands here based on package.json scripts
   # Example: npm install && npm run compile
   cd ../..
   ```

4. Update README.md installation instructions:
   
   Before:
   ```markdown
   - Cast (cli for this monorepo): Run `cargo install --path ./cast_cli` and `code --install-extension ./cast_vscode/cast.vsix`
   ```
   
   After:
   ```markdown
   - Cast (cli for this monorepo): Run `cargo install --path ./cast_workspace/cli` and `code --install-extension ./cast_workspace/vscode_ext/cast.vsix`
   ```

5. Commit the changes:
   ```bash
   git add cast_workspace/vscode_ext
   git add README.md
   git commit -m "Phase 5: Move cast_vscode to workspace"
   ```

**Validation**:
- `cast_workspace/vscode_ext` contains cast_vscode code
- README.md has updated paths
- Original cast_vscode still exists

**Rollback**:
```bash
# Revert README changes
git restore README.md

# Remove workspace vscode_ext
rm -rf cast_workspace/vscode_ext/*

# Verify original cast_vscode still exists
ls -la cast_vscode/
```

---

## Phase 6: Update Configuration Files

**Objective**: Update all remaining configuration and documentation files.

**Prerequisites**:
- Phase 5 complete
- All three projects in workspace
- Workflows and README updated
- Clean git working directory

**Steps**:

1. Update dependabot.yml:
   
   Before (currently incorrect):
   ```yaml
   - package-ecosystem: "cargo"
     directory: "/projects/cast_cli"
   ```
   
   After:
   ```yaml
   - package-ecosystem: "cargo"
     directory: "/cast_workspace"
   ```
   
   Replace both cast and cast_cli entries with single workspace entry.

2. Update REPOSITORY_STRUCTURE.md:
   
   Update structure diagram to show:
   ```
   ├── cast_workspace/          # Cast workspace (monorepo tooling)
   │   ├── core/               # cast_core library
   │   ├── cli/                # cast_cli binary
   │   └── vscode_ext/         # VSCode extension
   ```
   
   Update Cast CLI documentation section with new paths.

3. Update .github/workflows/README.md:
   
   Replace references to:
   - "cast_cli" → "cast_workspace/cli"
   - Update build paths in examples

4. Update .github/workflows/TROUBLESHOOTING.md:
   
   Update example paths:
   - "cast_cli/src/bin/cast.rs" → "cast_workspace/cli/src/bin/cast.rs"

5. Commit the changes:
   ```bash
   git add .github/dependabot.yml
   git add REPOSITORY_STRUCTURE.md
   git add .github/workflows/README.md
   git add .github/workflows/TROUBLESHOOTING.md
   git commit -m "Phase 6: Update configuration and documentation files"
   ```

**Validation**:
- All documentation has correct paths
- dependabot.yml references workspace directory
- No references to old structure remain in documentation

**Rollback**:
```bash
# Revert all documentation changes
git restore .github/dependabot.yml
git restore REPOSITORY_STRUCTURE.md
git restore .github/workflows/README.md
git restore .github/workflows/TROUBLESHOOTING.md
```

---

## Phase 7: Testing and Validation

**Objective**: Comprehensively test the new workspace structure before cleanup.

**Prerequisites**:
- All previous phases complete
- All files updated
- Clean git working directory
- All changes committed

**Steps**:

1. Test workspace builds:
   ```bash
   cd cast_workspace
   cargo clean
   cargo build --workspace
   cargo test --workspace
   cd ..
   ```

2. Test cast_cli functionality:
   ```bash
   cd cast_workspace/cli
   cargo build --release
   ./target/release/cast --help
   ./target/release/cast project --help
   # Test actual commands if possible
   cd ../..
   ```

3. Run cast ci on workspace:
   ```bash
   # Using old cast_cli temporarily
   cd cast_cli
   cargo build --release
   cd ..
   ./cast_cli/target/release/cast ci --project cast_workspace/core
   ./cast_cli/target/release/cast ci --project cast_workspace/cli
   ```

4. Test workflow tests:
   ```bash
   cd monorepo/workflow_tests
   cargo test
   cd ../..
   ```

5. Create test PR to verify workflows:
   - Push changes to feature branch
   - Create PR
   - Verify cast-ci.yml workflow succeeds
   - Verify all jobs pass
   - Note: Do not merge yet

6. Manual verification checklist:
   - [ ] Workspace builds successfully
   - [ ] All workspace tests pass
   - [ ] cast_cli binary works
   - [ ] Workflow tests pass
   - [ ] GitHub workflows pass in PR
   - [ ] Documentation is accurate
   - [ ] No broken links in docs

**Validation**:
- All builds succeed
- All tests pass
- Workflows pass in test PR
- CLI commands work correctly

**Rollback**:
At this point, rollback requires reverting multiple commits:
```bash
# If test PR shows problems:
# 1. Do not merge the PR
# 2. Checkout main/master branch
# 3. Optionally create new branch for fixes
# 4. Fix issues and restart from failed phase

# If already on main and need to rollback:
git log --oneline  # Find commit before Phase 2
git revert <commit-sha>..HEAD  # Revert all workspace commits
# Or
git reset --hard <commit-before-phase-2>
git push --force  # Only if not shared with others
```

---

## Phase 8: Cleanup

**Objective**: Remove old project directories and complete the migration.

**Prerequisites**:
- Phase 7 complete
- All tests passing
- Workflows verified in PR
- PR merged to main
- **CRITICAL**: Ensure backup branch exists

**Steps**:

1. **SAFETY**: Verify backup exists:
   ```bash
   git branch -a | grep backup/pre-workspace-restructure
   # Should show the backup branch
   ```

2. **SAFETY**: Tag current state before deletion:
   ```bash
   git tag -a workspace-migration-complete -m "State before removing old directories"
   git push origin workspace-migration-complete
   ```

3. Remove old cast directory:
   ```bash
   # Verify workspace/core has all content
   if ! diff -r cast cast_workspace/core --exclude=.git --exclude=target; then
     echo "ERROR: Directories differ, aborting deletion"
     exit 1
   fi
   
   # Remove
   git rm -r cast
   git commit -m "Remove old cast directory (migrated to cast_workspace/core)"
   ```

4. Remove old cast_cli directory:
   ```bash
   # Verify workspace/cli has all content
   if ! diff -r cast_cli cast_workspace/cli --exclude=.git --exclude=target; then
     echo "ERROR: Directories differ, aborting deletion"
     exit 1
   fi
   
   # Remove
   git rm -r cast_cli
   git commit -m "Remove old cast_cli directory (migrated to cast_workspace/cli)"
   ```

5. Remove old cast_vscode directory:
   ```bash
   # Verify workspace/vscode_ext has all content
   if ! diff -r cast_vscode cast_workspace/vscode_ext --exclude=.git --exclude=target --exclude=node_modules; then
     echo "ERROR: Directories differ, aborting deletion"
     exit 1
   fi
   
   # Remove
   git rm -r cast_vscode
   git commit -m "Remove old cast_vscode directory (migrated to cast_workspace/vscode_ext)"
   ```

6. Update cast_workspace/ISSUES.md:
   - Mark all Phase 1-8 TODOs as complete
   - Remove the completed TODO items
   - Add a "Migration completed" note with date

7. Final commit:
   ```bash
   git add cast_workspace/ISSUES.md
   git commit -m "Phase 8 complete: Workspace restructuring migration finished"
   git push
   ```

8. Verify everything works:
   ```bash
   cd cast_workspace
   cargo build --workspace
   cargo test --workspace
   cd cli
   cargo build --release
   ./target/release/cast --help
   ```

**Validation**:
- Old directories removed
- Workspace is the only cast implementation
- All builds and tests pass
- Git history preserved (use `git log --follow`)
- Backup branch and tag exist

**Rollback**:
```bash
# EMERGENCY ROLLBACK - Only if critical issues found

# ⚠️  WARNING: Force push operations are EXTREMELY DANGEROUS
# They will OVERWRITE any commits made by other team members
# ALWAYS coordinate with your team before using --force
# Consider creating a new branch for rollback instead

# Option 1: Restore from backup branch (SAFEST - no force push)
git checkout -b rollback/restore-old-cast backup/pre-workspace-restructure
git push origin rollback/restore-old-cast
# Then create PR to merge this rollback branch

# Option 2: Force reset from backup (DANGEROUS - requires coordination)
# ONLY use if you have confirmed no one else has pushed commits
git checkout main
git reset --hard backup/pre-workspace-restructure
# Verify this is what you want before pushing
git log --oneline -10
git push --force origin main  # ⚠️  DANGEROUS - COORDINATE WITH TEAM FIRST

# Option 3: Restore from tag (DANGEROUS - requires coordination)
git checkout main
git reset --hard workspace-migration-complete
git push --force origin main  # ⚠️  DANGEROUS - COORDINATE WITH TEAM FIRST

# Option 4: Manually restore directories (SAFEST - no force push)
git checkout workspace-migration-complete^  # Go back one commit
git checkout HEAD -- cast cast_cli cast_vscode
git commit -m "Rollback: Restore old cast directories"
git push origin main  # Safe push, no force needed
```

---

## Post-Migration Tasks

After all phases complete:

1. **Update copilot-instructions**:
   - Document workspace structure pattern
   - Note that cast is now a workspace
   - Add guidance for workspace development

2. **Announce to team**:
   - Document the change in team channels
   - Update any external documentation
   - Update developer setup guides

3. **Monitor for issues**:
   - Watch CI/CD for any problems
   - Monitor for any broken external references
   - Be ready to provide support

4. **Archive migration documents**:
   - Move WORKSPACE_RESTRUCTURING_DEPENDENCIES.md to docs/archive/
   - Move this migration plan to docs/archive/
   - Keep link in ISSUES.md for reference

---

## Emergency Contacts and Resources

- **Backup Branch**: `backup/pre-workspace-restructure`
- **Migration Tag**: `workspace-migration-complete`
- **Related Issues**: cast/ISSUES.md "Cast Workspace Restructuring (Epic)"

## Lessons Learned (To be filled in during migration)

Document any issues encountered and solutions:

- Issue: 
  - Solution: 

- Issue:
  - Solution:

---

## Sign-off Checklist

Before declaring migration complete:

- [ ] All 8 phases completed successfully
- [ ] Backup branch exists and verified
- [ ] Migration complete tag created
- [ ] All tests passing in workspace
- [ ] All GitHub workflows passing
- [ ] Documentation updated
- [ ] Team notified
- [ ] Old directories removed
- [ ] copilot-instructions updated
- [ ] No rollback needed for 2 weeks
- [ ] Migration documents archived

**Migration Completed By**: _________________  
**Date**: _________________  
**Final Commit SHA**: _________________  
