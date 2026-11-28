# 0010. Any team's guide, picked on the same site

- Status: Accepted
- Date: 2026-11-28
- Builds on: [0002](0002-optimize-super-bowl-odds.md),
  [0007](0007-nfc-north-divisional-mode.md) and
  [0009](0009-cloudflare-pages-nightly-cd.md)

## Context

Jim, 2026-11-28: "We should also make it so that you can pick any team in
the bear_down app and get recommendations." The simulator already measures
every swing on one "focus" team (`SimConfig::focus`), so the work is running
it once per team, writing the explanations for any team, and letting a fan
pick.

## Decision

- **Same site, Bears first.** The site keeps its name, address and Chicago
  Bears theme. The Bears stay the default team.
- **One file per team.** The nightly build writes the Bears' guide to
  `rooting.json`, as before, and every other team's to
  `teams/<team>.json` in the same format, with a new `team` field
  (`docs/rooting-json.md`). The odds object is now `focus` (it was
  `bears`); the page still reads older files.
- **A team picker on the page.** "Your team" in the header lists all 32
  teams. The choice is kept in the address (`?team=DET`, nothing for the
  Bears) so a link opens on that team, and in `localStorage` so a fan's
  next visit opens on it too.
- **Every text names the team.** The explanations, the banner and the page
  say "the Chiefs" and "AFC West title" for a Chiefs fan, the same way they
  say "the Bears" and "NFC North title" for a Bears fan.
- **NFC North mode stays NFC North.** The toggle and its stat show only for
  the four NFC North teams. The goal is the same for all four.
- **The Bears only, for now:** votes (BEAR-010) and the archive of past
  weeks. Votes on other teams' verdicts would mix into the A/B test, whose
  arms were set for the Bears' guide, and the archive would cost 32 times
  the build minutes. Both are backlog issues.

The other options Jim was offered were per-team colours (32 themes through
the design system) and a team-neutral site name and address. Either can
follow without redoing this.

## Cost

$0. The 31 extra guides add about four minutes to the nightly deploy on a
four-core runner (measured: 3 min 43 s for all 32, against about 30 s for
the Bears alone), on free GitHub Actions minutes for a public repo. Pages
gains 31 JSON files of about 50 KB each, far inside the free plan's 20,000
files. No new service.

## Consequences

- A guide for a team whose most recent game was a loss skips the second
  simulation the banner would need, so the nightly build only pays for it
  after a win.
- The Bears-specific requirements (BEAR-001 to BEAR-009) hold for every
  team's guide; BEAR-011 covers the picker.
