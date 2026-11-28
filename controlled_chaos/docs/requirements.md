# Controlled Chaos requirements

What Controlled Chaos promises the people who play it. The format and the
rules are in [standards/docs/evals.md](../../standards/docs/evals.md); the
evals that check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the [README](../README.md), the
[official rules](official_rules.md), the controlled_chaos issues in the
`standards/docs/issues/` (scripted runs, Bevy UAT, in-app feedback) and the existing
tests. Controlled Chaos has no deep review yet, so these are a first guess
at Jim's intent.

Only CHAOS-001 is a `must` for now: it is the only promise a production
eval can check today (the live web build answers with the game). The rules
(CHAOS-002) become a candidate `must` once the eval engine's `script` kind
plays the smoke script against the released build (eval engine phase 2).

## CHAOS-001: Anyone can play a game in a browser with nothing to install

- Priority: must
- Why: The browser build is how Jim and playtesters try the game; a link
  must be enough.
- Accept: <https://jimhillr-controlled-chaos.pages.dev> loads the game's
  page, start-up script and WebAssembly; from the start screen, Start Game
  begins a game against the CPU, and a player can draw, attack and defend
  through full rounds; if the game fails to start, the page says so instead
  of staying blank; no game in the web build panics.

## CHAOS-002: Every game follows the official rules

- Priority: should
- Why: A card game is only fair, and only worth testing, if the software
  plays by the rules that are written down.
- Accept: each turn runs draw, play, battle and end in order; in a battle
  the higher card wins, the loser loses 1 life and the winner gains 5
  happiness, kept between 0 and 100; a player with no life or no civilians
  loses at once; when the deck runs out the happier player wins, and equal
  happiness is a draw; the same seed deals the same game every time. The
  rules are those in [official_rules.md](official_rules.md).

## CHAOS-003: The game can be played by touch, mouse or keyboard

- Priority: should
- Why: Playtesters use phones, laptops and desktops, and shouldn't need
  instructions to start.
- Accept: every card in the hand and the Draw card / End turn button can be
  tapped or clicked, with touch targets at least 48 px; Space draws and
  ends the turn and 1 to 8 pick a card; the screen always says what to do
  next, whose turn it is and why the game ended.

## CHAOS-004: Everything on screen is easy to read, in any colour setting

- Priority: should
- Why: The game uses the design system's gamey theme and must be readable
  by everyone, including players who need more contrast.
- Accept: every text and button colour meets WCAG AA contrast against its
  background in dark and light; accessible mode meets AAA and makes text
  one step larger, and starts on in a browser that asks for more contrast;
  the eight card categories each have their own colour, distinct from the
  card body.

## CHAOS-005: The game remembers display settings between launches

- Priority: should
- Why: A player who picks light colours or accessible mode shouldn't have
  to pick it again each time.
- Accept: the colour scheme and accessible mode chosen on the start screen
  are the same at the next launch, on web, desktop and Android; the
  settings stay on the player's device; a missing, damaged or unwritable
  settings file never stops the game from starting.

## CHAOS-006: We can see whether people play, without identifying anyone

- Priority: should
- Why: Jim needs to know whether playtesters play and where the game
  breaks, without tracking them.
- Accept: the web build sends session start, game start, every fifth
  round, game over and panic events to the telemetry Worker, tagged with
  the build's commit; the events carry no ids, cookies or storage, and
  browsers that send Do Not Track or Global Privacy Control are not
  counted; desktop and Android builds send nothing.

## CHAOS-007: The CPU opponent plays sensibly

- Priority: could
- Why: A solo player needs an opponent that makes reasonable choices, or
  the game teaches nothing.
- Accept: the CPU attacks with its highest card and defends with the
  lowest card that wins, or its lowest card when none wins; its hand is
  never shown to the player.

## CHAOS-008: Desktop and Android builds can be downloaded

- Priority: should
- Why: Some playtesting needs a real app on a Mac or a phone, not a
  browser tab.
- Accept: the `controlled_chaos-nightly` release always has a macOS zip
  and an Android APK, rebuilt each night from the latest `main` that passed
  CI, linked from the README.

## CHAOS-009: The web build is safe to open

- Priority: should
- Why: The game is a public link; it must not be framed by another site or
  load code from anywhere but its own files.
- Accept: every page of the web build sends a Content-Security-Policy that
  allows only its own scripts and forbids framing, plus
  `X-Content-Type-Options: nosniff` and Strict-Transport-Security.
