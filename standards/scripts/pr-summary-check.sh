#!/usr/bin/env bash
# PR-001: a pull request description opens with a short executive summary
# (Jim, 2026-11-27): what the PR does, why, and any engineering tradeoffs,
# in at most 75 words, linking to the committed docs, plans or issues that
# hold the details. See "Pull request descriptions" in
# standards/docs/pull-requests.md.
#
#   PR_BODY="$(cat body.md)" bash standards/scripts/pr-summary-check.sh
#   bash standards/scripts/pr-summary-check.sh <body.md
#
# The summary is the text after the attribution lines (HTML comments and the
# "_Requested by" line) and the attention badges ("> **Attention:**") up to the first heading or rule. A leading
# "## Summary" heading is allowed and skipped. Link targets don't count as
# words. Warnings only: it always exits 0, and in GitHub Actions it prints
# ::warning annotations and writes the job summary.
set -uo pipefail

max_words=75
body=${PR_BODY-$(cat)}
body=${body//$'\r'/}
# Drop HTML comments, including multi-line ones like the template's guidance.
body=$(perl -0pe 's/<!--.*?-->//gs' <<<"$body")

summary=$(awk '
  !started && /^_Requested by / { next }                  # attribution line
  !started && /^> \*\*Attention:\*\*/ { next }           # attention badges
  !started && /^[[:space:]]*$/ { next }                   # leading blanks
  !started && /^#+[[:space:]]+[Ss]ummary[[:space:]]*$/ { started = 1; next }
  /^#/ || /^(---|\*\*\*|___)[[:space:]]*$/ { exit }
  { started = 1; print }
' <<<"$body")

# Markdown links count as their text; bare URLs and link targets don't count.
words=$(sed -E 's/\]\([^)]*\)/]/g; s#https?://[^[:space:]]+##g' <<<"$summary" | wc -w)
words=${words// /}

warnings=()
if [ "$words" -eq 0 ]; then
  warnings+=("The description has no executive summary: open it, after the attribution lines, with at most $max_words words on what, why and tradeoffs.")
elif [ "$words" -gt "$max_words" ]; then
  warnings+=("The executive summary is $words words; keep it to $max_words and link to committed docs, plans or issues for the details.")
fi
if grep -qF '<Summary:' <<<"$summary"; then
  warnings+=("The executive summary is still the template's placeholder; replace it with what, why and tradeoffs.")
fi
if [ "$words" -gt 0 ] && ! grep -qE '\]\(|https?://' <<<"$summary"; then
  warnings+=("The executive summary has no link; point it at the committed docs, plan or issue file that holds the details.")
fi

if [ "${#warnings[@]}" -eq 0 ]; then
  echo "PR-001 ok: executive summary is $words words."
else
  for w in "${warnings[@]}"; do
    if [ -n "${GITHUB_ACTIONS-}" ]; then
      echo "::warning title=PR-001 PR summary::$w"
    else
      echo "warning[PR-001] $w"
    fi
  done
fi

if [ -n "${GITHUB_STEP_SUMMARY-}" ]; then
  {
    echo "### PR summary (PR-001)"
    echo
    echo "Executive summary: $words words (limit $max_words)."
    for w in "${warnings[@]}"; do echo "- $w"; done
  } >>"$GITHUB_STEP_SUMMARY"
fi
exit 0
