#!/usr/bin/env bash
# Tests standards/scripts/deps-check.sh (SEC-003, SEC-004) in a throwaway
# repo with stand-in cargo-deny and npm that log where they ran: which
# lockfiles a change audits, and that findings warn without failing.
#   bash standards/tests/deps-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
repo=$work/repo
log=$work/ran.log
failures=0

mkdir -p "$repo/standards/scripts" "$repo/rs/sub" "$repo/web" "$repo/other" "$work/bin"
cp "$root/standards/scripts/deps-check.sh" "$repo/standards/scripts/"
echo '[advisories]' >"$repo/standards/deny.toml"
echo '# lock' >"$repo/rs/Cargo.lock"
echo '{}' >"$repo/web/package-lock.json"
echo 'x' >"$repo/other/file"
git -C "$repo" init -q
git -C "$repo" add -A

# Stand-ins: log "<tool> <dir>", and fail where a FAIL file exists.
cat >"$work/bin/cargo-deny" <<SH
#!/usr/bin/env bash
[ "\${1:-}" = --version ] && { echo 'cargo-deny 0.0.0'; exit 0; }
echo "deny \${PWD#$repo/}" >>"$log"
[ ! -f FAIL ]
SH
cat >"$work/bin/npm" <<SH
#!/usr/bin/env bash
echo "npm \${PWD#$repo/}" >>"$log"
[ ! -f FAIL ]
SH
chmod +x "$work/bin/cargo-deny" "$work/bin/npm"
export PATH="$work/bin:$PATH"

# check NAME WANT_RAN FILES...: run --files with FILES and compare what ran.
check() {
  local name=$1 want=$2
  shift 2
  : >"$log"
  out=$(cd "$repo" && printf '%s\n' "$@" | bash standards/scripts/deps-check.sh --files 2>&1)
  local got=$? ran
  ran=$(sort "$log" | tr '\n' ',' | sed 's/,$//')
  if [ "$got" -ne 0 ] || [ "$ran" != "$want" ]; then
    echo "FAIL $name: exit $got (want 0), ran '$ran' (want '$want')"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
  fi
}

check "no lockfile changed" "" other/file rs/src/main.rs
check "Cargo.lock" "deny rs" rs/Cargo.lock other/file
check "package-lock.json" "npm web" web/package-lock.json
check "project deny.toml" "deny rs" rs/deny.toml
check "shared deny.toml" "deny rs" standards/deny.toml
check "deleted lockfile" "" gone/Cargo.lock

# A finding is a warning: exit 0, with the SEC code in the output.
touch "$repo/rs/FAIL"
check "finding warns" "deny rs" rs/Cargo.lock
case "$out" in
  *"warning[SEC-003] rs:"*) ;;
  *) echo "FAIL finding warns: no warning[SEC-003] in:"; echo "$out" | sed 's/^/    /'
     failures=$((failures + 1)) ;;
esac
rm "$repo/rs/FAIL"

# --all audits every tracked lockfile.
: >"$log"
(cd "$repo" && bash standards/scripts/deps-check.sh --all >/dev/null 2>&1)
ran=$(sort "$log" | tr '\n' ',' | sed 's/,$//')
[ "$ran" = "deny rs,npm web" ] || { echo "FAIL --all: ran '$ran'"; failures=$((failures + 1)); }

if [ "$failures" -gt 0 ]; then
  echo "deps-check.test.sh: $failures failure(s)"
  exit 1
fi
echo "deps-check.test.sh: all cases passed"
