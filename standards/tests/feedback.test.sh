#!/usr/bin/env bash
# Tests standards/scripts/feedback.sh against a fake Analytics Engine API
# and a throwaway git origin: the query's rows, the per-arm summary, and
# that store merges weeks without duplicates and pushes only feedback-data.
#   bash standards/tests/feedback.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
script=$root/standards/scripts/feedback.sh
work=$(mktemp -d)
server=""
trap '[ -n "$server" ] && kill "$server" 2>/dev/null; rm -rf "$work"' EXIT
failures=0

ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; failures=$((failures + 1)); }
expect() { # expect NAME HAYSTACK NEEDLE
  if [[ "$2" == *"$3"* ]]; then ok "$1"; else fail "$1: wanted '$3' in: $2"; fi
}

# A fake Analytics Engine SQL endpoint that saves the query it was sent.
cat >"$work/answer.json" <<'JSON'
{"meta":[],"rows":4,"data":[
 {"timestamp":"2026-11-27 04:00:00","kind":"exposure","arm":"a","tab":"super_bowl","game":"","verdict":"","vote":"","note":"","season":2026,"week":12},
 {"timestamp":"2026-11-27 04:01:00","kind":"feedback","arm":"a","tab":"super_bowl","game":"2026_12_PHI_DAL","verdict":"doesnt_matter","vote":"no","note":"","season":"2026","week":"12"},
 {"timestamp":"2026-11-27 04:01:05","kind":"feedback","arm":"a","tab":"super_bowl","game":"2026_12_PHI_DAL","verdict":"doesnt_matter","vote":"note","note":"Felt like it | mattered","season":2026,"week":12},
 {"timestamp":"2026-11-27 05:00:00","kind":"feedback","arm":"b","tab":"playoffs","game":"2026_12_ATL_MIN","verdict":"ATL","vote":"yes","note":"","season":2026,"week":12}
]}
JSON
cat >"$work/server.py" <<'PY'
import http.server, sys
answer, saved = sys.argv[1], sys.argv[2]
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers["Content-Length"]))
        open(saved, "wb").write(body + b"\n" + self.headers["Authorization"].encode())
        data = open(answer, "rb").read()
        self.send_response(200); self.end_headers(); self.wfile.write(data)
    def log_message(self, *a): pass
s = http.server.HTTPServer(("127.0.0.1", 0), H)
print(s.server_port, flush=True)
s.serve_forever()
PY
python3 "$work/server.py" "$work/answer.json" "$work/sent.txt" >"$work/port" &
server=$!
for _ in $(seq 50); do [ -s "$work/port" ] && break; sleep 0.1; done
port=$(cat "$work/port")

rows=$(CLOUDFLARE_API_TOKEN=tok CLOUDFLARE_ACCOUNT_ID=acct FEEDBACK_CF_API="http://127.0.0.1:$port" \
  bash "$script" query bear_down 8 2>&1)
expect "query reads every row" "$(grep -c . <<<"$rows")" "4"
expect "query keeps vote fields" "$(sed -n 2p <<<"$rows")" '"game":"2026_12_PHI_DAL","verdict":"doesnt_matter","vote":"no"}'
expect "query turns season and week into numbers" "$(sed -n 2p <<<"$rows")" '"season":2026,"week":12'
expect "an exposure has no vote fields" "$(sed -n 1p <<<"$rows")" '"tab":"super_bowl"}'
expect "a note keeps its text" "$(sed -n 3p <<<"$rows")" '"note":"Felt like it | mattered"'
sent=$(cat "$work/sent.txt" 2>/dev/null)
expect "query filters on the project index" "$sent" "index1 = 'bear_down'"
expect "query asks for the last N days" "$sent" "INTERVAL '8' DAY"
expect "query sends the token" "$sent" "Bearer tok"

out=$(CLOUDFLARE_API_TOKEN=tok CLOUDFLARE_ACCOUNT_ID=acct FEEDBACK_CF_API="http://127.0.0.1:$port" \
  bash "$script" query "x' OR 1=1" 2>&1)
expect "query refuses an odd project name" "$out" "project must be"
out=$(env -u CLOUDFLARE_API_TOKEN bash "$script" query bear_down 2>&1)
expect "query needs a token" "$out" "CLOUDFLARE_API_TOKEN is not set"

printf '%s\n' "$rows" >"$work/rows.jsonl"
summary=$(bash "$script" summarize "$work/rows.jsonl")
expect "summary names the week" "$summary" "# Fan feedback, 2026 week 12"
expect "arm A counts its visit and vote" "$summary" "| A | 1 | 1 | 0 | 1 | 0 | 0% | 1 |"
expect "arm B counts its vote" "$summary" "| B | 0 | 1 | 1 | 0 | 0 | 100% | 0 |"
expect "a no-pick verdict is grouped" "$summary" "| A | no pick | 1 | 0 | 1 | 0 |"
expect "a pick is grouped" "$summary" "| B | pick | 1 | 1 | 0 | 0 |"
expect "notes are listed" "$summary" "- A, super_bowl, 2026_12_PHI_DAL (doesnt_matter): Felt like it | mattered"

# store: twice into a throwaway origin; the second run adds one new row.
git init -q --bare "$work/origin.git"
git init -q "$work/repo"
git -C "$work/repo" remote add origin "$work/origin.git"
git -C "$work/repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m init
(cd "$work/repo" && bash "$script" store bear_down "$work/rows.jsonl" >/dev/null 2>&1)
{ cat "$work/rows.jsonl"; echo '{"ts":"2026-12-02 01:00:00","kind":"feedback","arm":"b","season":2026,"week":13,"tab":"playoffs","game":"g","verdict":"CHI","vote":"yes"}'; } >"$work/rows2.jsonl"
(cd "$work/repo" && bash "$script" store bear_down "$work/rows2.jsonl" >/dev/null 2>&1)
tree=$(git -C "$work/origin.git" ls-tree -r --name-only feedback-data 2>&1)
expect "store writes the week's events" "$tree" "bear_down/2026-week-12.jsonl"
expect "store writes the week's summary" "$tree" "bear_down/2026-week-12.md"
expect "store starts a new week" "$tree" "bear_down/2026-week-13.jsonl"
expect "store drops duplicates" "$(git -C "$work/origin.git" show feedback-data:bear_down/2026-week-12.jsonl | wc -l | tr -d ' ')" "4"
out=$(cd "$work/repo" && bash "$script" store bear_down "$work/rows.jsonl" 2>&1)
expect "store with nothing new makes no commit" "$out" "nothing new"
if git -C "$work/origin.git" rev-parse -q --verify refs/heads/main >/dev/null || git -C "$work/origin.git" rev-parse -q --verify refs/heads/master >/dev/null; then
  fail "store pushed a branch other than feedback-data"
else
  ok "only feedback-data was pushed"
fi
: >"$work/empty.jsonl"
expect "store with no events does nothing" "$(cd "$work/repo" && bash "$script" store bear_down "$work/empty.jsonl" 2>&1)" "no events"

out=$(bash "$script" nope 2>&1)
expect "unknown command prints usage" "$out" "usage: feedback.sh"

if [ "$failures" -gt 0 ]; then
  echo "feedback.test.sh: $failures failure(s)"
  exit 1
fi
echo "feedback.test.sh: all passed"
