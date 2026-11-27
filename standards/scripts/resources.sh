#!/usr/bin/env bash
# Keep only the live resources we need, and retire renamed or removed ones on
# purpose. Rules and the file format: standards/docs/resources.md.
#
#   resources.sh check
#       Static checks on the repo, no credentials. Run by `mise run //:audit`.
#         RES-001  A resource claimed on the base but not on the head (a
#                  wrangler.toml `name` changed, a binding or project was
#                  removed, a game left the release list) has a [[retired]]
#                  or [[keep]] entry in standards/resources.toml. Runs only
#                  with a base: RESOURCES_BASE, else AFFECTED_BASE (PR CI's
#                  audit sets it; the pre-push hook sets RESOURCES_BASE).
#         RES-002  No tracked file outside standards/resources.toml and dated
#                  history (plans, decisions, reviews, changelogs, issue
#                  files) mentions a retired `url`, or a retired `name` unless
#                  the entry sets `match_name = false`.
#         RES-003  Every actions/upload-artifact step sets `retention-days`
#                  of 7 or less. A longer one needs a `# RES-003:` comment in
#                  the step saying why (review decks keep 90 days).
#         RES-004  Entries in standards/resources.toml are well formed: known
#                  kind, the required fields, dates as YYYY-MM-DD, and
#                  delete_after at least 7 days after retired.
#
#   resources.sh claims [REF]
#       Every claimed resource, one `kind<TAB>name` per line, from the
#       working tree or from the git REF.
#
#   resources.sh new-entries BASE
#       The [[keep]] and [[retired]] entries the working tree adds compared
#       with BASE, one `section<TAB>kind<TAB>name` per line. PR CI labels a
#       PR `needs-human` when there are any.
#
# RESOURCES_ROOT runs the checks in another repo (the tests use fixtures).
set -euo pipefail

die() { echo "resources: $*" >&2; exit 1; }

root=${RESOURCES_ROOT:-$(git rev-parse --show-toplevel)}
cd "$root"

# The Python half: tomllib reads the TOML files, so nothing is parsed with
# regular expressions except workflow YAML (RES-003), which needs only lines.
py() {
  RES_CMD="$1" RES_REF="${2:-}" python3 -I - <<'EOF'
import datetime, os, re, subprocess, sys, tomllib

cmd = os.environ["RES_CMD"]
ref = os.environ.get("RES_REF", "")
RESOURCES = "standards/resources.toml"
KINDS = {"pages", "worker", "d1", "kv", "queue", "r2", "cron", "dns",
         "auth0-url", "branch", "release"}
errors = 0


def err(code, msg):
    global errors
    print(f"error[{code}] {msg}")
    errors += 1


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True, check=True).stdout


def files(at):
    """Tracked files at REF (or in the working tree, with ''), by path."""
    if at:
        return [p for p in git("ls-tree", "-r", "--name-only", at).splitlines()]
    return [p for p in git("ls-files", "--cached", "--others", "--exclude-standard").splitlines()
            if os.path.isfile(p)]


def read(path, at):
    if at:
        return git("show", f"{at}:{path}")
    with open(path, encoding="utf-8") as f:
        return f.read()


def load(path, at):
    try:
        return tomllib.loads(read(path, at))
    except (subprocess.CalledProcessError, FileNotFoundError):
        return {}
    except (tomllib.TOMLDecodeError, UnicodeDecodeError) as e:
        print(f"warning[RES-001] {path}{' at ' + at if at else ''} is not valid TOML: {e}")
        return {}


def claims(at):
    """Set of (kind, name) claimed by files (standards/docs/resources.md)."""
    out = set()
    paths = files(at)
    for p in paths:
        base = os.path.basename(p)
        if base == "wrangler.toml":
            w = load(p, at)
            name = w.get("name")
            if not name:
                continue
            out.add(("pages" if "pages_build_output_dir" in w else "worker", name))
            for d in w.get("d1_databases", []):
                if d.get("database_name"):
                    out.add(("d1", d["database_name"]))
            for k in w.get("kv_namespaces", []):
                if k.get("binding"):
                    out.add(("kv", f"{name}/{k['binding']}"))
            for r in w.get("r2_buckets", []):
                if r.get("bucket_name"):
                    out.add(("r2", r["bucket_name"]))
            q = w.get("queues", {})
            for side in ("producers", "consumers"):
                for e in q.get(side, []):
                    if e.get("queue"):
                        out.add(("queue", e["queue"]))
            for c in w.get("triggers", {}).get("crons", []):
                out.add(("cron", f"{name}/{c}"))
        elif base == "mise.toml":
            tag = load(p, at).get("env", {}).get("RELEASE_TAG")
            if isinstance(tag, str) and tag:
                out.add(("release", tag))
    return out


def entries(at):
    data = load(RESOURCES, at)
    return {s: [e for e in data.get(s, []) if isinstance(e, dict)] for s in ("keep", "retired")}


if cmd == "claims":
    for kind, name in sorted(claims(ref)):
        print(f"{kind}\t{name}")
    sys.exit(0)

if cmd == "new-entries":
    old = {(s, e.get("kind"), e.get("name")) for s, es in entries(ref).items() for e in es}
    for s, es in entries("").items():
        for e in es:
            key = (s, e.get("kind"), e.get("name"))
            if key not in old:
                print("\t".join(str(x) for x in key))
    sys.exit(0)

# ---- check ----
ents = entries("")

# RES-004: well-formed entries.
DATE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


def date(e, field, where, required=True):
    v = e.get(field, "")
    if v == "" and not required:
        return None
    if not isinstance(v, str) or not DATE.match(v):
        err("RES-004", f"{where}: `{field}` must be a YYYY-MM-DD date, not {v!r}")
        return None
    try:
        return datetime.date.fromisoformat(v)
    except ValueError:
        err("RES-004", f"{where}: `{field}` {v!r} is not a real date")
        return None


for s, es in ents.items():
    for i, e in enumerate(es):
        where = f"{RESOURCES} [[{s}]] #{i + 1} ({e.get('name', '?')})"
        if e.get("kind") not in KINDS:
            err("RES-004", f"{where}: `kind` must be one of {', '.join(sorted(KINDS))}")
        needed = ["kind", "name", "why"] + (["review"] if s == "keep" else ["retired", "delete_after"])
        for f in needed:
            if not e.get(f):
                err("RES-004", f"{where}: missing `{f}`")
        if s == "keep":
            date(e, "review", where)
        else:
            r = date(e, "retired", where)
            d = date(e, "delete_after", where)
            date(e, "deleted", where, required=False)
            if r and d and (d - r).days < 7:
                err("RES-004", f"{where}: `delete_after` must be at least 7 days after `retired` (the grace period to undo)")
            urls = e.get("url", [])
            if not isinstance(urls, (str, list)):
                err("RES-004", f"{where}: `url` must be a string or a list of strings")

# RES-001: a dropped claim needs an entry.
if ref:
    try:
        git("rev-parse", "--verify", "--quiet", f"{ref}^{{commit}}")
    except subprocess.CalledProcessError:
        print(f"error[RES-001] base {ref} is not a commit here; fetch it (git fetch origin main) and run again")
        sys.exit(1)
    covered = {(e.get("kind"), e.get("name")) for es in ents.values() for e in es}
    for kind, name in sorted(claims(ref) - claims("")):
        if (kind, name) not in covered:
            err("RES-001", f"{kind} `{name}` is claimed on {ref} but not here; add a [[retired]] entry "
                f"(or [[keep]] if it stays) to {RESOURCES} (standards/docs/resources.md, \"Renaming a resource\")")

# RES-002: no references to retired names or addresses.
HISTORY = re.compile(
    r"^(standards/resources\.toml"
    r"|standards/docs/plans/.*"
    r"|standards/docs/decisions/.*"
    r"|(.*/)?docs/reviews/.*"
    r"|(.*/)?docs/(retros|plans)/.*"
    r"|(.*/)?issues/.*"
    r"|(.*/)?CHANGELOG[^/]*)$")
needles = []
for e in ents["retired"]:
    urls = e.get("url", [])
    urls = [urls] if isinstance(urls, str) else [u for u in urls if isinstance(u, str)]
    for u in urls:
        u = re.sub(r"^[a-z]+://", "", u).rstrip("/")
        if u:
            needles.append((u, re.compile(re.escape(u), re.I)))
    n = e.get("name")
    if isinstance(n, str) and n and e.get("match_name", True):
        needles.append((n, re.compile(r"(?<![A-Za-z0-9_.-])" + re.escape(n) + r"(?![A-Za-z0-9_-])")))
if needles:
    for p in files(""):
        if HISTORY.match(p):
            continue
        try:
            text = read(p, "")
        except (UnicodeDecodeError, OSError):
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for shown, rx in needles:
                if rx.search(line):
                    err("RES-002", f"{p}:{lineno} mentions retired `{shown}`; point it at the replacement "
                        f"or remove it (standards/docs/resources.md)")

if errors:
    print(f"resources: {errors} error(s). See standards/docs/resources.md")
    sys.exit(1)
print(f"resources: ok ({len(ents['keep'])} kept, {len(ents['retired'])} retired"
      f"{', claims compared with ' + ref if ref else ''})")
EOF
}

# RES-003: artifact retention. Lines only: a step runs from its `- ` line
# to the next line at the same or a lower indent that isn't blank or a
# comment.
retention() {
  local errors=0 f
  for f in $(git ls-files -- '.github/*.yml' '.github/*.yaml'); do
    [ -f "$f" ] || continue
    awk -v file="$f" '
      function finish() {
        if (in_step && upload) {
          if (days == "") { printf "error[RES-003] %s:%d upload-artifact sets no retention-days; set 7 or less (standards/docs/resources.md)\n", file, start; bad++ }
          else if (days !~ /^[0-9]+$/) { printf "error[RES-003] %s:%d retention-days %s is not a number\n", file, start, days; bad++ }
          else if (days + 0 > 7 && !reason) { printf "error[RES-003] %s:%d keeps artifacts %s days; use 7 or less, or add a \"# RES-003:\" comment in the step saying why\n", file, start, days; bad++ }
        }
        in_step = 0; upload = 0
      }
      {
        line = $0
        match(line, /^ */); indent = RLENGTH
        blank = (line ~ /^ *$/ || line ~ /^ *#/)
        if (in_step && !blank && indent <= step_indent) finish()
        if (line ~ /^ *- /) { finish(); in_step = 1; step_indent = indent; start = NR; days = ""; reason = 0 }
        if (!in_step) next
        if (line ~ /uses:[ ]*["\047]?actions\/upload-artifact@/) upload = 1
        if (line ~ /# *RES-003:/) reason = 1
        if (line ~ /^[ -]*retention-days:/) { d = line; sub(/^[ -]*retention-days: */, "", d); sub(/ *#.*$/, "", d); gsub(/["\047]/, "", d); days = d }
      }
      END { finish(); exit bad > 0 }
    ' "$f" || errors=$((errors + 1))
  done
  return "$errors"
}

case "${1:-}" in
  check)
    status=0
    py check "${RESOURCES_BASE:-${AFFECTED_BASE:-}}" || status=1
    retention || status=1
    exit "$status"
    ;;
  claims) py claims "${2:-}" ;;
  new-entries) py new-entries "${2:?usage: resources.sh new-entries BASE}" ;;
  *) die "usage: resources.sh check | claims [REF] | new-entries BASE" ;;
esac
