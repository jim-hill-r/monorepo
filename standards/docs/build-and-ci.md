# Build and CI Standards

## Artifact Management

- All build artifacts MUST be placed in `artifacts/{target-architecture}/` subdirectories (e.g., `artifacts/x86_64-unknown-linux-gnu/`, `artifacts/aarch64-apple-darwin/`).
- Build artifacts with `-dirty` suffix MUST NOT be committed to the repository.
- Projects MUST include a `.gitignore` file that excludes dirty artifacts using the pattern `artifacts/**/*-dirty.zip` and `artifacts/**/*-dirty`.

## CI Requirements

- All projects MUST run `cast ci` successfully before requesting code review.
- All projects with code changes MUST have `cast ci` pass before merging.
- CI failures MUST be addressed before requesting code review.
