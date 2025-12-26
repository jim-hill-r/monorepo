# Cast Workspace Restructuring - Current Dependencies and References

This document catalogs all current dependencies and references to the cast, cast_cli, and cast_vscode projects to aid in the workspace restructuring effort defined in ISSUES.md.

**Last Updated**: 2025-12-26

## Project Overview

### cast (Library)
- **Location**: `/cast/`
- **Type**: Rust library crate
- **Purpose**: Core Cast library providing monorepo tooling functionality
- **Package Name**: `cast` (in Cargo.toml)

### cast_cli (Binary)
- **Location**: `/cast_cli/`
- **Type**: Rust binary crate
- **Purpose**: Command-line interface for Cast commands
- **Package Name**: `cast_cli` (in Cargo.toml)
- **Dependencies**: Depends on the `cast` library

### cast_vscode (VSCode Extension)
- **Location**: `/cast_vscode/`
- **Type**: TypeScript/Node.js VSCode extension
- **Purpose**: VSCode extension for Cast monorepo tooling
- **Package Name**: `cast` (in package.json, displayName: "Cast")

## Dependency References

### cast_cli → cast Dependency

**File**: `/cast_cli/Cargo.toml`
```toml
[dependencies]
cast = { path = "../cast" }
```
- The cast_cli binary depends on the cast library using a relative path
- **Action Needed**: Update this path reference when moving to workspace structure

## GitHub Actions Workflows

### cast-ci.yml
**File**: `/.github/workflows/cast-ci.yml`

References to cast_cli:
- **Line 38-41**: Build cast CLI
  ```yaml
  - name: Build cast CLI
    run: |
      cd cast_cli
      cargo build --release
  ```
- **Line 75**: Set CAST_BIN path
  ```yaml
  CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"
  ```
- **Line 106**: Set CAST_BIN path (again in different step)
  ```yaml
  CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"
  ```

**Action Needed**: Update all paths from `cast_cli/` to new workspace structure

### cast-cd.yml
**File**: `/.github/workflows/cast-cd.yml`

References to cast_cli:
- **Line 31-33**: Build cast CLI
  ```yaml
  - name: Build cast CLI
    run: |
      cd cast_cli
      cargo build --release
  ```
- **Line 67**: Set CAST_BIN path
  ```yaml
  CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"
  ```
- **Line 98**: Set CAST_BIN path (again in different step)
  ```yaml
  CAST_BIN="$GITHUB_WORKSPACE/cast_cli/target/release/cast"
  ```

**Action Needed**: Update all paths from `cast_cli/` to new workspace structure

## Dependabot Configuration

**File**: `/.github/dependabot.yml`

References:
- **Lines 6-20**: cast_cli Cargo dependencies
  ```yaml
  - package-ecosystem: "cargo"
    directory: "/projects/cast_cli"
    schedule:
      interval: "weekly"
    labels:
      - "dependencies"
      - "rust"
      - "cast_cli"
  ```
  Note: Currently uses incorrect path `/projects/cast_cli` (should be `/cast_cli`)

- **Lines 22-36**: cast Cargo dependencies
  ```yaml
  - package-ecosystem: "cargo"
    directory: "/projects/cast"
    schedule:
      interval: "weekly"
    labels:
      - "dependencies"
      - "rust"
      - "cast"
  ```
  Note: Currently uses incorrect path `/projects/cast` (should be `/cast`)

**Action Needed**: Update directories to reflect actual current paths first, then update to new workspace structure

## Documentation References

### README.md (Root)
**File**: `/README.md`

- **Line 12**: Installation instructions
  ```markdown
  - Cast (cli for this monorepo): Run `cargo install --path ./cast_cli` and `code --install-extension ./cast_vscode/cast.vsix`
  ```

**Action Needed**: Update paths in installation instructions

### REPOSITORY_STRUCTURE.md
**File**: `/REPOSITORY_STRUCTURE.md`

- **Line 21**: Structure diagram
  ```
  ├── cast/                # Core Cast library for monorepo tooling
  ```
- **Line 22**: Structure diagram
  ```
  ├── cast_cli/            # Cast command-line interface
  ```
- **Lines 71-80**: Cast CLI documentation
  ```markdown
  ## Cast CLI
  
  The `cast` CLI is the primary tool for managing this monorepo:
  - Located at `cast_cli/`
  - Build with: `cargo build --release`
  - Commands:
    - `cast project new <name>` - Create new project from exemplars
    - `cast project with-changes --base <ref> --head <ref>` - Find projects with changes
    - `cast ci` - Run CI checks (lint, build, test)
    - `cast session start` - Start a work session
  ```

**Action Needed**: Update all path references and structure documentation

### .github/workflows/README.md
**File**: `/.github/workflows/README.md`

- **Line 18**: Description of cast-ci.yml workflow
  ```markdown
  - Builds the `cast` CLI from `cast_cli`
  ```
- **Line 30**: Setup requirements for cast-ci.yml
  ```markdown
  3. The `cast_cli` project must be buildable
  ```
- **Line 83**: Description of cast-cd.yml workflow
  ```markdown
  - Builds the `cast` CLI from `cast_cli`
  ```
- **Line 95**: Setup requirements for cast-cd.yml
  ```markdown
  3. The `cast_cli` project must be buildable
  ```

**Action Needed**: Update documentation to reflect new workspace structure

### .github/workflows/TROUBLESHOOTING.md
**File**: `/.github/workflows/TROUBLESHOOTING.md`

- **Line 34**: Example troubleshooting issue
  ```markdown
  The issue was found in `cast_cli/src/bin/cast.rs` where imports were not alphabetically sorted:
  ```

**Action Needed**: Update path reference in troubleshooting examples

## Test References

### workflow_tests
**Files**: 
- `/workflow_tests/tests/cast_ci_workflow_tests.rs`
- `/workflow_tests/tests/cast_cd_workflow_tests.rs`
- `/workflow_tests/src/lib.rs`

References in cast_ci_workflow_tests.rs:
- **Lines 36-46**: Test checks for cast_cli usage in workflow
- **Lines 60-68**: Test checks for cast_cli build step
- **Lines 71-79**: Test checks for "cast ci" command

References in cast_cd_workflow_tests.rs:
- **Lines 47-57**: Test checks for cast_cli usage in workflow
- **Lines 71-79**: Test checks for cast_cli build step
- **Lines 82-90**: Test checks for "cast cd" command

References in lib.rs:
- **Lines 43-46**: Helper function for cast_cli path
  ```rust
  pub fn get_cast_cli_cargo_path() -> PathBuf {
      get_repo_root().join("cast_cli/Cargo.toml")
  }
  ```

**Action Needed**: Update test assertions and helper functions to work with new workspace structure

## VSCode Extension References

### cast_vscode Files
- Has its own Cast.toml, ISSUES.md, README.md
- package.json defines the extension metadata
- Built artifacts: cast.vsix

**Action Needed**: Move entire directory to workspace, update references in documentation

## Summary of Files Requiring Updates

### Configuration Files
1. `/.github/workflows/cast-ci.yml` - Multiple path references
2. `/.github/workflows/cast-cd.yml` - Multiple path references
3. `/.github/dependabot.yml` - Directory paths (currently incorrect)
4. `/cast_cli/Cargo.toml` - Dependency path to cast library

### Documentation Files
1. `/README.md` - Installation instructions
2. `/REPOSITORY_STRUCTURE.md` - Structure diagrams and descriptions
3. `/.github/workflows/README.md` - Workflow documentation
4. `/.github/workflows/TROUBLESHOOTING.md` - Troubleshooting examples
5. `/cast/ISSUES.md` - Workspace restructuring epic (this work)

### Test Files
1. `/workflow_tests/src/lib.rs` - Path helper function
2. `/workflow_tests/tests/cast_ci_workflow_tests.rs` - Test assertions
3. `/workflow_tests/tests/cast_cd_workflow_tests.rs` - Test assertions

## Proposed Workspace Structure

After restructuring, the layout will be:
```
/cast/                          # Workspace root
├── Cargo.toml                  # [workspace] configuration
├── Cast.toml                   # Workspace Cast config
├── README.md                   # Workspace documentation
├── ISSUES.md                   # Workspace issues (moved from current)
├── core/                       # cast library (renamed from cast)
│   ├── Cargo.toml             # package name: cast_core
│   └── src/
├── cli/                        # cast_cli (moved)
│   ├── Cargo.toml             # package name: cast_cli
│   └── src/
└── vscode_ext/                 # cast_vscode (moved and renamed)
    ├── package.json
    └── src/
```

## Migration Path Considerations

### Phase-by-Phase Updates Required:

**Phase 1**: Document current state (✓ This document)

**Phase 2**: Create workspace structure
- No external references need updating yet

**Phase 3**: Rename and move cast → cast_core
- Update: cast_cli/Cargo.toml dependency path
- Update: Any other internal references

**Phase 4**: Move cast_cli → cast/cli
- Update: GitHub workflows build paths
- Update: GitHub workflows CAST_BIN paths
- Update: workflow_tests helper functions
- Update: workflow_tests test assertions

**Phase 5**: Move cast_vscode → cast/vscode_ext
- Update: README.md installation instructions

**Phase 6**: Update configuration files
- Update: dependabot.yml directories (fix current issues + new structure)
- Update: REPOSITORY_STRUCTURE.md
- Update: Any other documentation

**Phase 7**: Testing
- Verify all workflows still work
- Verify all tests pass
- Verify cast ci runs successfully

**Phase 8**: Cleanup
- Remove old directories
- Remove completed TODOs from ISSUES.md

## Notes for Implementation

1. Each phase should be completed and tested before moving to the next
2. The cast_cli binary is critical to CI/CD - extra care needed
3. Workflow tests validate workflow correctness - keep them updated
4. The dependabot.yml currently has incorrect paths that should be fixed
5. Consider using Git's `git mv` command to preserve history when moving files
6. The workspace Cargo.toml will need proper member declarations
7. The cast library will need renaming to cast_core to avoid conflicts with workspace name
