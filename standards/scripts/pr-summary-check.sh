#!/usr/bin/env bash
# PR-001: a pull request description has the sections Jim asked for
# (2026-11-27), after the attribution lines:
#
#   ## Attention      one bullet per tag: exactly one requirements tag, one
#                     cost tag, one threat/risk tag and one docs tag, from
#                     the closed set below (`chores badges` writes it)
#   ## What changed   at most 50 words
#   ## Why            at most 50 words
#   ## Cost changes   under 25 words ("None." when nothing changes)
#   ## Risk changes   under 25 words ("None." when nothing changes)
#   ## Details        anything else: testing, review notes, links
#
# A planning PR, one that adds or edits a plan or decision record (Markdown
# in a plans/ or decisions/ folder, not a README or a test fixture), can't
# say "None." for cost (Jim, 2026-11-28): its Cost changes section names
# the plan's cost Now (all current projects), at 5x and at 100x as many
# ("Cost at scale" in standards/docs/cost.md). PR_FILES lists the PR's
# added and edited files, one per line; without it there's no plan check.
#
# See "Pull request descriptions" in standards/docs/pull-requests.md.
# `chores pr-body --check` applies the same rules; keep the two in step.
#
#   PR_BODY="$(cat body.md)" bash standards/scripts/pr-summary-check.sh
#   bash standards/scripts/pr-summary-check.sh <body.md
#   PR_FILES="$(git diff --name-only --diff-filter=AMR main)" PR_BODY=... bash ...
#
# A section runs from its heading (any level, any case) to the next heading
# or rule. HTML comments are ignored, Markdown links count as their text,
# and link targets and bare URLs don't count. Warnings only: it always exits
# 0, and in GitHub Actions it prints ::warning annotations and writes the
# job summary.
set -uo pipefail

# The attention tags, by group. Keep in step with `Tag` in chores/src/badges.rs.
requirement_tags="must-requirement-changed requirement-changed requirements-unchanged"
cost_tags="cost-changed cost-unchanged"
risk_tags="threat-risk-changed threat-risk-unchanged"
docs_tags="docs-only docs-changed docs-unchanged"

body=${PR_BODY-$(cat)}

# Plans and decision records in PR_FILES. Keep in step with `is_plan_file`
# in chores/src/badges.rs.
plans=$(printf '%s\n' "${PR_FILES-}" | perl -ne '
  chomp; next unless m{/};
  my @d = split m{/}; my $name = lc pop @d;
  next unless $name =~ /\.md$/ && $name ne "readme.md";
  next unless grep { $_ eq "plans" || $_ eq "decisions" } @d;
  next if grep { $_ eq "fixtures" } @d;
  print "$_\n";')

# One line per section: name, max words, found (0/1), words, placeholder (0/1).
report=$(PR_BODY=$body perl -e '
  my $b = $ENV{PR_BODY};
  $b =~ s/\r//g;
  $b =~ s/<!--.*?-->//gs;
  my @sections = (["Attention", 0], ["What changed", 50], ["Why", 50], ["Cost changes", 24],
                  ["Risk changes", 24], ["Details", 0]);
  my (%found, %text, $cur);
  for my $line (split /\n/, $b) {
    if ($line =~ /^#+\s+(.*?)\s*$/) {
      my $h = lc $1;
      $cur = undef;
      for my $s (@sections) {
        if ($h eq lc $s->[0]) { $cur = $s->[0]; $found{$cur} = 1; $text{$cur} //= ""; }
      }
      next;
    }
    if ($line =~ /^(---|\*\*\*|___)\s*$/) { $cur = undef; next; }
    $text{$cur} .= "$line\n" if defined $cur;
  }
  for my $s (@sections) {
    my ($name, $max) = @$s;
    my $t = $text{$name} // "";
    (my $w = $t) =~ s/\]\([^)]*\)/]/g;
    $w =~ s{https?://\S+}{}g;
    my @words = split " ", $w;
    (my $trim = $t) =~ s/^\s+|\s+$//g;
    my $placeholder = ($trim =~ /^<.*>$/s) ? 1 : 0;
    printf "%s\t%d\t%d\t%d\t%d\n", $name, $max, $found{$name} ? 1 : 0,
      scalar @words, $placeholder;
  }
  # Scale markers the Cost changes section leaves out (SCALES in chores).
  (my $c = lc($text{"Cost changes"} // "")) =~ s/\x{d7}|\xc3\x97/x/g;
  my @missing = grep { my $m = $_; $c !~ /(?<![a-z0-9])\Q$m\E(?![a-z0-9])/ } ("now", "5x", "100x");
  print "scales\t", join(", ", @missing), "\n";
  # Attention bullets: the first word after "- " with a letter in it.
  for my $line (split /\n/, $text{"Attention"} // "") {
    next unless $line =~ /^\s*- (.*)$/;
    my ($tag) = grep { /[A-Za-z0-9]/ } split " ", $1;
    $tag //= "";
    $tag =~ s/:$//; $tag =~ s/^`+|`+$//g;
    print "tag\t$tag\n";
  }
')

warnings=()
counts=()
req=0 cost=0 risk=0 docs=0
while IFS=$'\t' read -r kind tag; do
  [ "$kind" = tag ] || continue
  if [[ " $requirement_tags " == *" $tag "* ]]; then req=$((req + 1))
  elif [[ " $cost_tags " == *" $tag "* ]]; then cost=$((cost + 1))
  elif [[ " $risk_tags " == *" $tag "* ]]; then risk=$((risk + 1))
  elif [[ " $docs_tags " == *" $tag "* ]]; then docs=$((docs + 1))
  else
    known="$requirement_tags $cost_tags $risk_tags $docs_tags"
    warnings+=("\"$tag\" is not an attention tag; use one of ${known// /, }.")
  fi
done <<<"$report"
if grep -q $'^Attention\t0\t1' <<<"$report"; then
  for pair in "requirements:$req" "cost:$cost" "threat/risk:$risk" "docs:$docs"; do
    n=${pair##*:}
    [ "$n" -eq 1 ] || warnings+=("The \"## Attention\" section has $n ${pair%:*} tags; it needs exactly one.")
  done
fi
missing_scales=$(sed -n $'s/^scales\t//p' <<<"$report")
if [ -n "$plans" ] && [ -n "$missing_scales" ] && grep -q $'^Cost changes\t24\t1' <<<"$report"; then
  first=$(head -n1 <<<"$plans")
  n=$(wc -l <<<"$plans")
  more=""
  [ "$n" -gt 1 ] && more=" and $((n - 1)) more"
  warnings+=("This PR changes a plan or decision (\`$first\`$more), so \"## Cost changes\" must estimate its cost as implemented across all current projects, at 5x and at 100x as many: name Now, 5x and 100x (missing $missing_scales). See \"Cost at scale\" in standards/docs/cost.md.")
fi
report=$(grep -v $'^\(tag\|scales\)\t' <<<"$report")

while IFS=$'\t' read -r name max found words placeholder; do
  if [ "$found" -eq 0 ]; then
    if [ "$name" = Attention ]; then
      warnings+=("The description has no \"## Attention\" section; run \`chores badges\` and put its output first.")
    else
      warnings+=("The description has no \"## $name\" section.")
    fi
    continue
  fi
  [ "$max" -eq 0 ] && continue # Attention and Details: no word limit
  counts+=("$name $words")
  limit="at most $max words"
  [ "$max" -eq 24 ] && limit="under 25 words"
  if [ "$placeholder" -eq 1 ]; then
    warnings+=("The \"## $name\" section is still the template's placeholder.")
  elif [ "$words" -eq 0 ]; then
    case $name in
      Cost* | Risk*) warnings+=("The \"## $name\" section is empty; write \"None.\" when nothing changes.") ;;
      *) warnings+=("The \"## $name\" section is empty; keep it to $limit.") ;;
    esac
  elif [ "$words" -gt "$max" ]; then
    warnings+=("The \"## $name\" section is $words words; keep it to $limit and move the rest to Details or a linked doc.")
  fi
done <<<"$report"

summary_line="$(IFS=,; echo "${counts[*]}")"
summary_line=${summary_line//,/, }
if [ "${#warnings[@]}" -eq 0 ]; then
  echo "PR-001 ok: $summary_line words."
else
  for w in "${warnings[@]}"; do
    if [ -n "${GITHUB_ACTIONS-}" ]; then
      echo "::warning title=PR-001 PR description::$w"
    else
      echo "warning[PR-001] $w"
    fi
  done
fi

if [ -n "${GITHUB_STEP_SUMMARY-}" ]; then
  {
    echo "### PR description (PR-001)"
    echo
    echo "Words per section: ${summary_line:-none found}. Limits: What changed and Why at most 50, Cost changes and Risk changes under 25."
    for w in "${warnings[@]}"; do echo "- $w"; done
  } >>"$GITHUB_STEP_SUMMARY"
fi
exit 0
