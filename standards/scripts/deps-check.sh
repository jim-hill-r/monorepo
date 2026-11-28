#!/usr/bin/env bash
# SEC-003 and SEC-004: dependency audits (standards/docs/security.md).
#
#   deps-check.sh --files    lockfile directories among the changed files
#                            read from stdin (one path per line)
#   deps-check.sh --all      every tracked Cargo.lock and package-lock.json
#
# SEC-003 runs cargo-deny (advisories and sources; licences later) where a Cargo.lock
# or deny.toml changed, with standards/deny.toml or the project's own
# deny.toml. SEC-004 runs `npm audit --omit=dev --audit-level=high` where a
# package-lock.json changed. A change to anything else costs nothing.
#
# Both are warnings until phase 4 of the threat check plan: findings print
# as warning[SEC-003] / warning[SEC-004] and the script always exits 0.
# cargo-deny comes from PATH, or else `mise exec cargo-deny@<version>`; when
# neither works it says so and skips SEC-003.
set -uo pipefail

# Pinned version. Bump it here only.
CARGO_DENY_VERSION=0.20.2

mode=${1:?usage: deps-check.sh --files|--all}
root=$(git rev-parse --show-toplevel)
cd "$root"

say() { printf 'deps-check: %s\n' "$*" >&2; }

case "$mode" in
  --all)
    locks=$(git ls-files -- ':(glob)**/Cargo.lock' ':(glob)**/package-lock.json')
    ;;
  --files)
    locks=""
    while IFS= read -r f; do
      [ -n "$f" ] || continue
      case "$(basename "$f")" in
        Cargo.lock | package-lock.json) [ -f "$f" ] && locks+="$f"$'\n' ;;
        deny.toml)
          d=$(dirname "$f")
          if [ "$f" = standards/deny.toml ]; then
            locks+=$(git ls-files -- ':(glob)**/Cargo.lock')$'\n'
          elif [ -f "$d/Cargo.lock" ]; then
            locks+="$d/Cargo.lock"$'\n'
          fi
          ;;
      esac
    done
    ;;
  *)
    say "unknown mode '$mode' (want --files or --all)"
    exit 2
    ;;
esac

locks=$(printf '%s' "$locks" | sed '/^$/d' | sort -u)
if [ -z "$locks" ]; then
  exit 0
fi

deny_cmd=()
if grep -q 'Cargo.lock$' <<<"$locks"; then
  # Run it, not `command -v`: a mise shim with no version set can't run.
  if cargo-deny --version >/dev/null 2>&1; then
    deny_cmd=(cargo-deny)
  elif command -v mise >/dev/null 2>&1 \
    && mise exec "cargo-deny@$CARGO_DENY_VERSION" -- cargo-deny --version >/dev/null 2>&1; then
    deny_cmd=(mise exec "cargo-deny@$CARGO_DENY_VERSION" -- cargo-deny)
  else
    say "cargo-deny is not installed and mise couldn't download it, so SEC-003"
    say "was not checked here. Install it with 'cargo install cargo-deny --locked'."
  fi
fi

warnings=0
while IFS= read -r lock; do
  d=$(dirname "$lock")
  case "$(basename "$lock")" in
    Cargo.lock)
      [ ${#deny_cmd[@]} -gt 0 ] || continue
      config=$root/standards/deny.toml
      [ -f "$d/deny.toml" ] && config=$root/$d/deny.toml
      say "$d: cargo deny (SEC-003)"
      if ! (cd "$d" && "${deny_cmd[@]}" --log-level error --config "$config" check \
        --hide-inclusion-graph advisories sources 2>&1); then
        echo "warning[SEC-003] $d: cargo-deny found problems (above). Update the crate, or file a fix issue (the project's docs/issues/), or add a dated ignore with a reason to $(basename "$config")."
        warnings=$((warnings + 1))
      fi
      ;;
    package-lock.json)
      if ! command -v npm >/dev/null 2>&1; then
        say "npm is not installed, so SEC-004 was not checked for $d"
        continue
      fi
      say "$d: npm audit (SEC-004)"
      if ! (cd "$d" && npm audit --omit=dev --audit-level=high 2>&1); then
        echo "warning[SEC-004] $d: npm audit found a high or critical advisory (above). Run 'npm audit fix' or file a fix issue (the project's docs/issues/)."
        warnings=$((warnings + 1))
      fi
      ;;
  esac
done <<<"$locks"

say "$warnings warning(s); SEC-003 and SEC-004 are warnings only for now"
exit 0
