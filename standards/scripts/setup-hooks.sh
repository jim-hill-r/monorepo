#!/usr/bin/env bash
# Install lefthook and the git hooks in lefthook.yml. Run by `mise run //:hooks`;
# plain `bash standards/scripts/setup-hooks.sh` works too. Safe to re-run.
#
# lefthook comes from mise (`mise exec lefthook@<version>`), at the version in
# lefthook.yml's min_version. Cloud sandboxes often can't download it through
# mise (GitHub release downloads are blocked), so this falls back to the same
# version from npm, then from PyPI. `lefthook install` writes the absolute path
# of the binary it ran from into the hooks, so they find it later without mise
# on PATH.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

say() { printf 'hooks: %s\n' "$*" >&2; }

version=$(sed -nE 's/^min_version: *"?([0-9.]+)"?.*/\1/p' lefthook.yml)
: "${version:?lefthook.yml has no min_version}"

if command -v mise >/dev/null 2>&1 \
  && MISE_YES=1 mise exec "lefthook@$version" -- lefthook install 2>/dev/null; then
  say "git hooks installed with lefthook $version from mise"
  exit 0
fi

have() { lefthook version 2>/dev/null | grep -qxF "$version"; }

if ! have && command -v npm >/dev/null 2>&1; then
  say "mise couldn't install lefthook $version; trying npm"
  npm install --global --silent "lefthook@$version" || true
  hash -r
fi
if ! have && command -v pip >/dev/null 2>&1; then
  say "trying pip"
  pip install --quiet "lefthook==$version" || true
  hash -r
fi
if ! have; then
  say "could not install lefthook $version through mise, npm or pip"
  exit 1
fi

lefthook install
say "git hooks installed with lefthook $version"
