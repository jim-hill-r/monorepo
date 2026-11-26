#!/usr/bin/env bash
# Downloadable builds of the games, published to GitHub Releases each night
# by .github/workflows/nightly-downloads.yml. A project opts in by setting
# RELEASE_TAG in its mise.toml [env] (see "Game builds" in
# standards/docs/mise.md).
#
#   downloads.sh list
#       JSON array of the projects that publish downloads, each
#       {"project", "slug", "tag", "deploy_url"}. Also `mise run //:downloads`.
#
#   downloads.sh publish TAG DIR [PROJECT]
#       Create or update the prerelease TAG so it points at GITHUB_SHA and
#       holds every file in DIR, replacing assets of the same name. Asset
#       names stay the same every night, so this link always serves the
#       newest build:
#         https://github.com/<repo>/releases/download/TAG/<asset>
#       Needs gh, GH_TOKEN (contents: write), GITHUB_REPOSITORY, GITHUB_SHA.
set -euo pipefail

die() { echo "downloads: $*" >&2; exit 1; }

root=$(git rev-parse --show-toplevel)

list() {
  cd "$root"
  PROJECTS=$(bash standards/scripts/affected.sh --all) python3 - <<'EOF'
import json, os, tomllib
out = []
for p in json.loads(os.environ["PROJECTS"]):
    with open(os.path.join(p, "mise.toml"), "rb") as f:
        env = tomllib.load(f).get("env", {})
    tag = env.get("RELEASE_TAG")
    if tag:
        out.append({
            "project": p,
            "slug": p.replace("/", "-"),
            "tag": tag,
            "deploy_url": env.get("DEPLOY_URL", ""),
        })
print(json.dumps(out))
EOF
}

publish() {
  local tag=${1:?usage: downloads.sh publish TAG DIR [PROJECT]}
  local dir=${2:?usage: downloads.sh publish TAG DIR [PROJECT]}
  local project=${3:-$tag}
  : "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}"
  : "${GITHUB_SHA:?GITHUB_SHA is not set}"
  local files=()
  local f
  for f in "$dir"/*; do
    [ -f "$f" ] && files+=("$f")
  done
  [ "${#files[@]}" -gt 0 ] || die "no files in $dir to publish"

  local url="https://github.com/$GITHUB_REPOSITORY/releases/download/$tag"
  local notes="Nightly playtest builds of \`$project\` from commit $GITHUB_SHA ($(date -u +%Y-%m-%d)). Not signed for public release; see $project/README.md, \"Playtesting\"."
  notes+=$'\n\n'
  for f in "${files[@]}"; do
    notes+="- [$(basename "$f")]($url/$(basename "$f"))"$'\n'
  done
  if [ -n "${DEPLOY_URL:-}" ]; then
    notes+="- Play in the browser: $DEPLOY_URL"$'\n'
  fi

  if gh release view "$tag" --repo "$GITHUB_REPOSITORY" >/dev/null 2>&1; then
    # Move the tag to this commit, then replace the assets.
    gh api -X PATCH "repos/$GITHUB_REPOSITORY/git/refs/tags/$tag" \
      -f sha="$GITHUB_SHA" -F force=true >/dev/null
    gh release edit "$tag" --repo "$GITHUB_REPOSITORY" \
      --title "$project (nightly)" --notes "$notes" --prerelease --latest=false
    gh release upload "$tag" --repo "$GITHUB_REPOSITORY" --clobber "${files[@]}"
  else
    gh release create "$tag" --repo "$GITHUB_REPOSITORY" --target "$GITHUB_SHA" \
      --title "$project (nightly)" --notes "$notes" --prerelease --latest=false \
      "${files[@]}"
  fi

  {
    echo "### Published $project downloads"
    echo
    for f in "${files[@]}"; do
      echo "- $url/$(basename "$f")"
    done
  } >>"${GITHUB_STEP_SUMMARY:-/dev/null}"
  echo "downloads: published ${#files[@]} file(s) to $tag"
}

case "${1:-}" in
  list) list ;;
  publish) shift; publish "$@" ;;
  *) die "usage: downloads.sh list | publish TAG DIR [PROJECT]" ;;
esac
