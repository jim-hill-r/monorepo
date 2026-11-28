# Sparkhill requirements

What sparkhill promises its users. The format and the rules are in
[standards/docs/evals.md](../../standards/docs/evals.md); the evals that
check each requirement are in [evals.toml](../evals.toml).

So far these cover **Blue Eel** (`webapp`, deployed by `webapp_cloudflare`
to <https://jimhillr-sparkhill-webapp.pages.dev>), the letter and word
tracing app that helps learners, including children with dyslexia, learn
to write. Jim made it top priority on 2026-11-27, with the goal of parity
with the live app at <https://blue.eel.education>. The Perseus `website`
has no requirements yet.

Drafted by an agent on 2026-11-27 from the web app's README, the "Blue Eel
parity" epic in [standards/docs/issues/README.md](../../standards/docs/issues/README.md), the parity PRs
(#709, #762, #767) and the threat model. When these were drafted
cloud sandboxes couldn't load blue.eel.education, so "like the live app"
meant what its page head and public pattern data showed. Sandboxes can
load it since 2026-11-27; the gaps that turned up are in
[standards/docs/issues/README.md](../../standards/docs/issues/README.md).

## SPRK-001: A learner can start practising straight away

- Priority: must
- Why: The learner is often a child; the first screen must get them
  tracing with no account, no setup and nothing to read.
- Accept: the home page shows "Blue Eel", "Writing made simple!" and a
  Begin button, as on the live app; Begin opens the letter level, whose
  first letter is b; no sign-in or download is needed. The learner's name
  is optional, in the settings panel (SPRK-010).

## SPRK-002: Every letter is shown being written before the learner tries it

- Priority: must
- Why: Learners copy the stroke order and direction they are shown, so
  the demonstration must draw each letter the way Blue Eel teaches it.
- Accept: every lowercase letter, capital and practice word has a
  demonstration which, with the Tracing technique (the default, see
  SPRK-009), is drawn stroke by stroke from the live app's own recorded
  pattern on the four writing guide lines (cap, mean, base and beard), in
  Blue Eel's colours; the patterns ship with the app, so a demonstration
  never waits on another site.

## SPRK-003: A tracing is checked fairly

- Priority: must
- Why: A learner who traces the letter right must be told so even with a
  wobbly finger, and one who draws something else must not be, or the
  practice teaches nothing.
- Accept: a tracing that follows the letter's strokes on the right guide
  lines passes ("Great job!") wherever it sits across the canvas, at any
  canvas size and with a shaky hand; another letter, a stroke drawn
  backwards, a missing or extra stroke, or a letter too small or on the
  wrong lines fails ("Try again!"); no letter passes as another; after 3
  misses the next letter comes up and the missed one returns later.

## SPRK-004: Learners move from letters to words to capitals

- Priority: must
- Why: The course builds up from a few easy letters, and a learner should
  only move on once they have them.
- Accept: letters come in Blue Eel's groups, starting b, c, d, f; a new
  group opens only once the open letters are traced right first time 3
  times each; misses alone never end a level; when a level ends with
  Tracing, "Go" starts it again with Pattern, and after Pattern with
  Freeform (SPRK-009); when it ends with Freeform, "Go" opens the next
  one, back with Tracing: letters, then words, then capitals, then the
  congratulations page, whose "Start Over" goes home.

## SPRK-005: Each learner's progress is kept on their device

- Priority: must
- Why: Practice happens a few minutes at a time, often with several
  children sharing one tablet, so each must carry on where they left off.
- Accept: progress is saved after every tracing under the name typed as
  Student in the settings panel ("Standard" when blank); a reload or a
  later visit carries on where that learner was; another name starts
  afresh; the settings panel remembers the last name; practising a
  sequence (SPRK-008) keeps its own progress, apart from the whole
  level's.

## SPRK-006: Nothing a learner types or draws leaves their device

- Priority: must
- Why: The learners are children. Blue Eel needs no account and sends
  nothing anywhere, so it has no personal data to protect or lose.
- Accept: using the app makes no request to any other site, and none to
  its own site carrying a name or a tracing; the site's security headers
  let the page connect only to its own origin; there is no sign-in,
  cookie or third-party script.

## SPRK-007: Blue Eel works and installs like an app on a phone or tablet

- Priority: should
- Why: Learners trace with a finger on a tablet or phone, as on the live
  app, and come back to it from the home screen.
- Accept: the page can't be pinch-zoomed, so tracing draws instead of
  zooming; it has the live app's title, description and theme colour; it
  links a web app manifest named "Blue Eel" that opens full screen, with
  icons that load; it can be added to an iPhone's or Android phone's home
  screen.

## SPRK-008: A few letters or words can be practised on their own

- Priority: could
- Why: A teacher or parent can focus a session on the letters a learner
  finds hard.
- Accept: a link such as `/letter?practice=b,d`, `/word?practice=hi` or
  `/capital?practice=T` practises only those, in that order, as one group,
  ignoring anything that has no pattern; such a session never changes the
  learner's saved progress. The settings panel's Sequence, such as
  `d, f, fun`, does the same and is kept on the device: its single
  lowercase letters on the letter level, its capitals on the capital
  level and its longer entries on the word level; a level it leaves out
  has its own groups.

## SPRK-009: A learner can practise with less help

- Priority: should
- Why: As on the live app, a learner who can trace a letter moves on to
  writing it from its start and end dots, then from memory, which is how
  they learn to write it alone.
- Accept: a Technique picker in the settings panel, and a link with
  `?tech=Tracing`, `?tech=Pattern` or `?tech=Freeform`, chooses how
  letters are shown: Tracing animates every stroke; Pattern shows only
  each stroke's start and end dots, which stay while the learner draws;
  Freeform shows nothing. Tracing is the default, the choice is kept on
  the device, and a letter written right passes with any technique.

## SPRK-010: The practice menu and settings panel work like the live app

- Priority: should
- Why: Learners and teachers who know the live app find each level in its
  left menu and set the student, what to practise and how in its settings
  panel, so Blue Eel keeps them where they are used to.
- Accept: a menu button at the top left opens Letter Practice and Word
  Practice (and this app's Capital Practice), each with a caption; a
  button at the top right opens a settings panel with Student, Sequence
  and Technique, which Save keeps on the device and which starts the open
  level again with them; Escape or a tap outside closes either. It has no
  Upload Practice switch, since nothing leaves the device (SPRK-006).
