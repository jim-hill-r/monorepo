#!/usr/bin/env bash
# THR-*: check every project's docs/threat.toml against the security rules
# with `threat check` (the threat project; the format and rules are in
# standards/docs/security.md). Warnings only until phase 4 of the threat
# check plan, so this always exits 0.
#
#   threat-check.sh [--projects a,b]
#       Check the listed projects, or every one. Called by `mise run
#       //:audit`. Without --projects, THREAT_PROJECTS, or else
#       COST_PROJECTS (both comma separated; hooks.sh and PR CI set
#       COST_PROJECTS to the affected projects), picks them when set, even
#       empty.
#
# The binary is built once into threat/target, so later runs take well
# under a second. Without cargo it prints a warning and exits 0; PR CI
# checks it. Run from anywhere in the repo.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

if ! command -v cargo >/dev/null 2>&1; then
  echo "warning: threat check skipped: cargo is not installed"
  exit 0
fi

if [ $# -eq 0 ]; then
  if [ -n "${THREAT_PROJECTS+set}" ]; then
    set -- --projects "$THREAT_PROJECTS"
  elif [ -n "${COST_PROJECTS+set}" ]; then
    set -- --projects "$COST_PROJECTS"
  fi
fi

if ! cargo run --quiet --manifest-path threat/Cargo.toml -p threat_cli -- check --root . "$@"; then
  echo "warning: threat check could not run; see the output above"
fi
exit 0
