#!/usr/bin/env bash
# SEC-001: scan for committed secrets with gitleaks (standards/docs/security.md).
#
#   secrets-check.sh staged         the staged diff (pre-commit hook)
#   secrets-check.sh range <base>   the commits in <base>..HEAD (pre-push hook, PR CI)
#   secrets-check.sh history        every commit (weekly full-history scan)
#
# Rules are the gitleaks defaults plus ours in .gitleaks.toml; known false
# positives are in .gitleaksignore. Findings are printed redacted, since CI
# logs are public.
#
# gitleaks comes from PATH, or else from `mise exec gitleaks@<version>`.
# When neither works (cloud sandboxes can't download GitHub releases), it
# warns and exits 0 outside CI, so PR CI runs the check instead; in CI
# (`CI` set) a missing gitleaks is an error.
set -uo pipefail

# Pinned version. Bump it here only.
GITLEAKS_VERSION=8.30.1

mode=${1:?usage: secrets-check.sh staged|range <base>|history}
root=$(git rev-parse --show-toplevel)
cd "$root"

say() { printf 'secrets-check: %s\n' "$*" >&2; }

gitleaks_cmd=()
# `gitleaks version`, not `command -v`: a mise shim on PATH with no version
# set for this directory is found but can't run.
if gitleaks version >/dev/null 2>&1; then
  gitleaks_cmd=(gitleaks)
elif command -v mise >/dev/null 2>&1 \
  && mise exec "gitleaks@$GITLEAKS_VERSION" -- gitleaks version >/dev/null 2>&1; then
  gitleaks_cmd=(mise exec "gitleaks@$GITLEAKS_VERSION" -- gitleaks)
fi

if [ ${#gitleaks_cmd[@]} -eq 0 ]; then
  if [ -n "${CI:-}" ]; then
    say "error[SEC-001] gitleaks $GITLEAKS_VERSION could not be installed"
    exit 1
  fi
  say "warning[SEC-001] gitleaks is not installed and mise couldn't download it,"
  say "so secrets were not scanned here. PR CI scans them. Say so in the PR."
  exit 0
fi

opts=(git --no-banner --verbose --redact --config .gitleaks.toml --gitleaks-ignore-path .gitleaksignore)
case "$mode" in
  staged) opts+=(--staged) ;;
  range)
    base=${2:?usage: secrets-check.sh range <base>}
    opts+=(--log-opts="$base..HEAD")
    ;;
  history) ;;
  *)
    say "unknown mode '$mode' (want staged, range <base> or history)"
    exit 2
    ;;
esac

if ! "${gitleaks_cmd[@]}" "${opts[@]}" .; then
  say "error[SEC-001] gitleaks found a secret, or failed (output above)."
  say "A real secret: remove it from the change, then rotate it if it was ever"
  say "pushed. Not a secret: add its fingerprint to .gitleaksignore with a"
  say "comment saying why. See \"Secrets\" in standards/docs/security.md."
  exit 1
fi
