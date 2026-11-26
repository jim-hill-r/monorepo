# Plan: robust, explained recommendations

- Status: accepted (Jim, 2026-11-26); steps 1 to 4 done
- Requirements: [BEAR-003, BEAR-004, BEAR-005, BEAR-006](../requirements.md)

## Where we are

After PRs #629 and #638 (2026-11-26):

- Each game's swing comes from paired simulations: every season is played
  twice from the same seed, once with each result. That removed most of the
  noise, but 100,000 pairs still can't separate a 0.01-point swing from zero.
- Each pick has reasons, but they are rules of thumb written next to the
  numbers: "Falcons win raises strength of victory". The simulation never
  says whether strength of victory actually decided anything.
- A "doesn't matter" game has no explanation at all.
- Conflicts aren't shown. Saints at Bengals today: a Saints win comes out
  ahead by 0.02 playoff points and 0.01 Super Bowl points, so the
  strength-of-victory gain outweighs one more NFC team's win. The page shows
  one strength-of-victory line in the playoff view and nothing in the Super
  Bowl view, and never mentions the effect pulling the other way.

## Approach

Explain from what the simulation actually did, not from rules of thumb.
In each paired season, the only difference is the one game, so whatever
changed for the Bears between the two seasons was caused by that game. Record
what changed and why, and add it up.

### 1. Channel attribution in the simulator (core)

For each game, in each pair of seasons, classify how the Bears' outcome
differs between the two results:

| Channel | Detected when |
| --- | --- |
| Division title | the Bears win the NFC North in one season and not the other |
| Wild card | the Bears make the playoffs in one and not the other, outside the division |
| Seed | same qualification, different seed |
| Tiebreaker, by step | the Bears' seed changes while their record doesn't, and the step that separated them from the team that moved is head-to-head, division record, common games, conference record, strength of victory, strength of schedule or later |
| Playoff path | same seed, different NFC opponents (who made it, their seeds) |
| Super Bowl opponent | same NFC path, different AFC champion |

`Standings::seeds` gains an optional trace of which tiebreaker step decided
each pair of teams, so the tiebreaker row is measured, not guessed. Each
channel gets a count in each direction, so the swing splits into
contributions that add up to the total, with their own standard errors.

### 2. A static relevance pass (no simulation)

Before simulating, list for each game which channels are open at all:

- whether each team can still reach the playoffs (a maximum-wins bound
  against the current seventh seed), or the Super Bowl;
- whether each team is on the Bears' schedule (strength of schedule), was
  beaten by the Bears (strength of victory), or is a common opponent with an
  NFC contender;
- whether the game is in the NFC, in the NFC North, or in the AFC.

A channel that is closed here is closed for sure, which gives "doesn't
matter" explanations that are exact: "Both teams are AFC, eliminated, and
on no NFC contender's schedule, so this game can't change anything for the
Bears."

### 3. Enough simulations for small swings

Run more pairs for games whose swing is close to the "doesn't matter" line,
until the standard error is under a fifth of the line or a cap is reached.
Pairs are cheap (about 1.3 seconds per game per 100,000 today) and the build
has minutes to spare. Replace the yes/no cut with tiers: matters, small,
negligible (under 0.05 points with confidence), and none (every channel
closed by step 2).

### 4. Explanations from contributions

Write each game's explanation from the channel table, biggest contribution
first, with direction and size, and an explicit net line when channels
disagree:

> Saints at Bengals: two small effects pull opposite ways (illustrative
> split; today's net is measured, the split isn't yet).
> - A Saints win raises the Bears' strength of victory (they beat the Saints
>   in week 11), which decides the Bears' seed in a few seasons in 10,000.
> - A Saints loss keeps one more NFC team behind the Bears in the wild-card
>   race, but the 3-7 Saints rarely threaten it.
> - The Bengals are AFC, so their result only reaches the Bears through the
>   Super Bowl opponent, and they seldom get there.
> - Net: a Saints win, by 0.01 Super Bowl points. Doesn't matter.

`rooting.json` gets a `channels` list per goal (name, favors, contribution,
standard error, plain-language text) beside the current `why`, and the page
shows it under every game, not just picks.

### 5. Checks that keep it honest (evals)

- Contributions add up to the total swing within the standard error.
- Every game has a non-empty explanation (BEAR-003, BEAR-005).
- No two "doesn't matter" games share identical text unless their channel
  tables match.
- Golden weeks: 2023 week 18 and a few other past weeks where the right
  answer is known (clinching scenarios published by the NFL), checked
  against the explanations.
- A tiebreaker claim only appears when step 1 measured that step deciding a
  seed (BEAR-006).

## Order of work

Each is one PR, in this order:

1. Tiebreaker trace in `Standings::seeds`, tested against past seasons'
   real tiebreaks. Done: `Standings::seeds_traced` reports each
   `Tiebreak` (step, tied group, survivors).
2. Channel counts in the paired simulation, with the adding-up test. Done:
   `GameImpact::channels` splits the playoff and Super Bowl swings by
   channel (division title, wild card, seed, playoff path, Super Bowl
   opponent, and chance for leftover simulation noise) and, for the seeding
   channels, by cause: a record change, a coin flip, or the first
   tiebreaker step whose result differed between the two seasons. The
   shares add up to the swing exactly. It added about 5% to the weekly
   build (15 to 16 seconds locally).
3. Static relevance pass and exact "can't matter" cases. Done:
   `relevance` lists each game's open links (the Bears play, a team in the
   race, a possible Super Bowl opponent, a team on contenders' schedules).
   A game with none can't matter in any season. The elimination check uses
   records only, so it errs toward "still alive" (the 2023 Bears, out only
   on tiebreakers, still count as alive); in week 12 every game is open.
4. Adaptive pair counts and the tiers. Done: a game whose Super Bowl
   swing's two-standard-error range spans the 0.05-point line, with a
   standard error over a fifth of it, gets another batch of pairs, up to 4
   times the simulations. A game the relevance pass closes isn't simulated,
   so its swings are exactly zero. `GameImpact::tier` gives matters, small,
   negligible or none. In week 12, four games got extra pairs and the build
   went from about 16 to 20.5 seconds locally. The page doesn't use the
   tiers yet; step 5 does.
5. `channels` in `rooting.json`, explanation text and the page.
6. Golden-week evals.

## Open questions for Jim

- Is 0.05 points on Super Bowl odds the right "doesn't matter" line, or
  should it be relative to the Bears' current odds?
- Should the playoff-odds view get the same explanations, or is the Super
  Bowl view enough?
