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
#       Needs gh, GH_TOKEN (contents: write), GITHUB_REPOSITORY, GITHUB_SHA
#       and git history (the tag skips commits that change workflows, which
#       the workflow token may not tag).
#
#   downloads.sh tag-target SHA
#       The commit the tag goes on for SHA (see tag_target).
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

# The newest commit at or before $1 (within 200) that doesn't change
# .github/workflows; $1 itself when none is found. Needs the history fetched.
tag_target() {
  local c
  for c in $(git rev-list --max-count=200 "$1" 2>/dev/null); do
    if ! git diff-tree --no-commit-id --name-only -r --root "$c" | grep -q '^\.github/workflows/'; then
      echo "$c"
      return
    fi
  done
  echo "$1"
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

  # GitHub refuses (403) to let the workflow token create or move a tag onto
  # a commit that changes .github/workflows, so tag the newest commit at or
  # before GITHUB_SHA that doesn't. The notes name the commit actually built.
  local target
  target=$(tag_target "$GITHUB_SHA")
  [ "$target" = "$GITHUB_SHA" ] ||
    echo "downloads: $GITHUB_SHA changes workflows, so the tag goes on $target"

  if gh release view "$tag" --repo "$GITHUB_REPOSITORY" >/dev/null 2>&1; then
    # Move the tag (best effort), then replace the assets.
    gh api -X PATCH "repos/$GITHUB_REPOSITORY/git/refs/tags/$tag" \
      -f sha="$target" -F force=true >/dev/null ||
      echo "::warning::could not move tag $tag to $target; uploading the new builds anyway"
    gh release edit "$tag" --repo "$GITHUB_REPOSITORY" \
      --title "$project (nightly)" --notes "$notes" --prerelease --latest=false
    gh release upload "$tag" --repo "$GITHUB_REPOSITORY" --clobber "${files[@]}"
  else
    gh release create "$tag" --repo "$GITHUB_REPOSITORY" --target "$target" \
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
  tag-target) tag_target "${2:?usage: downloads.sh tag-target SHA}" ;;
  publish) shift; publish "$@" ;;
  *) die "usage: downloads.sh list | publish TAG DIR [PROJECT]" ;;
esac
