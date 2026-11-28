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
#   resources.sh sweep
#       The weekly read-only sweep (the `resource-sweep` job of All Free
#       Tier Usage). Lists what is live into $RESOURCES_OUT/live.json:
#       Pages projects (with their deployment counts), Workers and their
#       Cron Triggers, D1, KV, Queues and DNS records with the read-only
#       Cloudflare token (CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID), and
#       branches, releases and Actions caches with GH_TOKEN (GITHUB_REPOSITORY
#       names the repo). A kind a token can't read is listed as unlisted, a
#       warning, not a failure. Then `chores sweep` sorts each resource into
#       claimed, kept, retired, temporary or orphan and writes report.md
#       (also to the run summary) and report.json. Exit 1 on an orphan, a
#       keep past its review date or a retired resource still live after
#       delete_after. It never changes anything.
#
# RESOURCES_ROOT runs the checks in another repo (the tests use fixtures).
set -euo pipefail

die() { echo "resources: $*" >&2; exit 1; }

root=${RESOURCES_ROOT:-$(git rev-parse --show-toplevel)}
# The sweep's classifier, next to this script, so fixture repos use it too.
chores="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/chores/Cargo.toml"
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

# ---- sweep ----

sweep() {
  local out=${RESOURCES_OUT:-$(mktemp -d)}
  local summary=${GITHUB_STEP_SUMMARY:-/dev/null}
  local cf_api=${CF_API:-https://api.cloudflare.com/client/v4}
  local repo=${GITHUB_REPOSITORY:-jimhillr/monorepo}
  mkdir -p "$out"
  local res="$out/resources.jsonl" unl="$out/unlisted.jsonl"
  : >"$res"
  : >"$unl"

  # add KIND NAME [DETAIL [PR [UPDATED]]]: one live resource.
  add() {
    jq -nc --arg k "$1" --arg n "$2" --arg d "${3:-}" --arg p "${4:-}" --arg u "${5:-}" \
      '{kind: $k, name: $n} + (if $d == "" then {} else {detail: $d} end)
       + (if $p == "" then {} else {pr: $p} end) + (if $u == "" then {} else {updated: $u} end)' >>"$res"
  }
  unlisted() { jq -nc --arg k "$1" --arg w "$2" '{kind: $k, why: $w}' >>"$unl"; echo "::warning::resource sweep: $1 not listed: $2"; }

  # cf PATH: a successful Cloudflare API response body, or status 1. The
  # token goes through stdin so it is never on a command line.
  cf() {
    local body
    body=$(printf 'header = "Authorization: Bearer %s"\n' "$CLOUDFLARE_API_TOKEN" |
      curl -fsS -K - -A jimhillr-resource-sweep "$cf_api/$1" 2>/dev/null) || return 1
    jq -e '.success == true' >/dev/null <<<"$body" || return 1
    printf '%s\n' "$body"
  }
  local acct="accounts/${CLOUDFLARE_ACCOUNT_ID:-}"

  if [ -z "${CLOUDFLARE_API_TOKEN:-}" ] || [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
    local k
    for k in pages worker cron d1 kv queue dns; do unlisted "$k" "CLOUDFLARE_API_TOKEN or CLOUDFLARE_ACCOUNT_ID isn't set"; done
  else
    local body page name count id ok
    # Pages projects, 10 a page (the API's most).
    page=1 ok=1
    while :; do
      body=$(cf "$acct/pages/projects?per_page=10&page=$page") || { ok=0; break; }
      while IFS= read -r name; do
        [ -n "$name" ] || continue
        count=$(cf "$acct/pages/projects/$name/deployments?per_page=1" | jq -r '.result_info.total_count // empty' 2>/dev/null || true)
        add pages "$name" "${count:+$count deployments}"
      done < <(jq -r '.result[].name' <<<"$body")
      [ "$(jq '.result | length' <<<"$body")" -ge 10 ] || break
      page=$((page + 1))
      [ "$page" -le 20 ] || break
    done
    [ "$ok" = 1 ] || unlisted pages "the token can't list Pages projects"
    # Workers and their Cron Triggers.
    if body=$(cf "$acct/workers/scripts"); then
      while IFS= read -r id; do
        [ -n "$id" ] || continue
        add worker "$id"
        if sched=$(cf "$acct/workers/scripts/$id/schedules"); then
          while IFS= read -r c; do [ -n "$c" ] && add cron "$id/$c"; done < <(jq -r '.result.schedules[]?.cron' <<<"$sched")
        fi
      done < <(jq -r '.result[].id' <<<"$body")
    else
      unlisted worker "the token can't list Workers (needs Workers Scripts: Read)"
      unlisted cron "the token can't list Workers (needs Workers Scripts: Read)"
    fi
    if body=$(cf "$acct/d1/database?per_page=100"); then
      while IFS= read -r name; do [ -n "$name" ] && add d1 "$name"; done < <(jq -r '.result[].name' <<<"$body")
    else
      unlisted d1 "the token can't list D1 databases (needs D1: Read)"
    fi
    if body=$(cf "$acct/storage/kv/namespaces?per_page=100"); then
      while IFS= read -r name; do [ -n "$name" ] && add kv "$name"; done < <(jq -r '.result[].title' <<<"$body")
    else
      unlisted kv "the token can't list KV namespaces (needs Workers KV Storage: Read)"
    fi
    if body=$(cf "$acct/queues"); then
      while IFS= read -r name; do [ -n "$name" ] && add queue "$name"; done < <(jq -r '.result[]?.queue_name' <<<"$body")
    else
      unlisted queue "the token can't list Queues (needs Queues: Read)"
    fi
    # DNS records, one row per name, once a zone exists.
    if body=$(cf "zones?account.id=$CLOUDFLARE_ACCOUNT_ID&per_page=50"); then
      while IFS=$'\t' read -r id zone; do
        [ -n "$id" ] || continue
        if recs=$(cf "zones/$id/dns_records?per_page=500"); then
          while IFS=$'\t' read -r name types; do
            [ -n "$name" ] && add dns "$name" "$types in $zone"
          done < <(jq -r '.result | group_by(.name)[] | [.[0].name, ([.[].type] | unique | join(", "))] | @tsv' <<<"$recs")
        else
          unlisted dns "the token can't read DNS records of $zone (needs DNS: Read)"
        fi
      done < <(jq -r '.result[] | [.id, .name] | @tsv' <<<"$body")
    else
      unlisted dns "the token can't list zones (needs Zone: Read)"
    fi
  fi

  if [ -z "${GH_TOKEN:-}" ] || ! command -v gh >/dev/null; then
    local k
    for k in branch release cache; do unlisted "$k" "GH_TOKEN isn't set"; done
  else
    local open_prs open_heads closed_heads branches b pr updated
    if open_prs=$(gh api --paginate "repos/$repo/pulls?state=open&per_page=100" \
          --jq ".[] | [.number, (if .head.repo.full_name == \"$repo\" then .head.ref else \"\" end)] | @tsv") &&
       closed_heads=$(gh api --paginate "repos/$repo/pulls?state=closed&per_page=100" \
          --jq ".[] | select(.head.repo.full_name == \"$repo\") | .head.ref") &&
       branches=$(gh api --paginate "repos/$repo/branches?per_page=100" --jq '.[].name'); then
      open_heads=$(cut -f2 <<<"$open_prs" | sed '/^$/d')
      while IFS= read -r b; do
        [ -n "$b" ] || continue
        updated=""
        if grep -qxF -- "$b" <<<"$open_heads"; then pr=open
        elif grep -qxF -- "$b" <<<"$closed_heads"; then pr=closed
        else
          pr=none
          [ "$b" = main ] || updated=$(gh api "repos/$repo/branches/$(jq -rn --arg b "$b" '$b | @uri')" \
            --jq '.commit.commit.committer.date' 2>/dev/null | cut -c1-10 || true)
        fi
        add branch "$b" "" "$pr" "$updated"
      done <<<"$branches"
      # Actions caches, one row per ref: claimed while its branch or PR is.
      if caches=$(gh api --paginate "repos/$repo/actions/caches?per_page=100" \
            --jq '.actions_caches[] | [.ref, .size_in_bytes] | @tsv'); then
        while IFS=$'\t' read -r ref n bytes; do
          [ -n "$ref" ] || continue
          case "$ref" in
            refs/heads/main) pr=main ;;
            refs/heads/*) grep -qxF -- "${ref#refs/heads/}" <<<"$branches" && pr=open || pr=closed ;;
            refs/pull/*) num=${ref#refs/pull/}; num=${num%%/*}
              cut -f1 <<<"$open_prs" | grep -qxF -- "$num" && pr=open || pr=closed ;;
            *) pr=closed ;;
          esac
          add cache "$ref" "$n caches, $((bytes / 1048576)) MB" "$pr"
        done < <(awk -F'\t' 'NF { n[$1]++; s[$1] += $2 } END { for (r in n) printf "%s\t%d\t%d\n", r, n[r], s[r] }' <<<"$caches" | sort)
      else
        unlisted cache "GH_TOKEN can't list Actions caches (needs actions: read)"
      fi
    else
      unlisted branch "GH_TOKEN can't list branches or pull requests"
    fi
    if rel=$(gh api --paginate "repos/$repo/releases?per_page=100" --jq '.[].tag_name'); then
      while IFS= read -r name; do [ -n "$name" ] && add release "$name"; done <<<"$rel"
    else
      unlisted release "GH_TOKEN can't list releases"
    fi
  fi

  jq -s --slurpfile u "$unl" '{unlisted: $u, resources: .}' "$res" >"$out/live.json"
  py claims "" >"$out/claims.tsv"
  local status=0
  cargo run --quiet --manifest-path "$chores" -- --root "$root" sweep \
    --live "$out/live.json" --claims "$out/claims.tsv" --json "$out/report.json" >"$out/report.md" || status=$?
  cat "$out/report.md"
  cat "$out/report.md" >>"$summary"
  case $status in
    0) ;;
    1) echo "::error::resource sweep: orphans, overdue keeps or failed deletions; see the run summary (standards/docs/resources.md)" ;;
    *) echo "::error::resource sweep failed (exit $status)" ;;
  esac
  return "$status"
}

case "${1:-}" in
  sweep) sweep ;;
  check)
    status=0
    py check "${RESOURCES_BASE:-${AFFECTED_BASE:-}}" || status=1
    retention || status=1
    exit "$status"
    ;;
  claims) py claims "${2:-}" ;;
  new-entries) py new-entries "${2:?usage: resources.sh new-entries BASE}" ;;
  *) die "usage: resources.sh check | claims [REF] | new-entries BASE | sweep" ;;
esac
