# bear_down requirements

What the rooting guide must do for a Bears fan, in the format of
[standards/docs/evals.md](../../standards/docs/evals.md). Written
2026-11-26 from Jim's feedback on the first live version. How we get there is
in [plans/2026-11-robust-recommendations.md](plans/2026-11-robust-recommendations.md).
Since [decision 0010](decisions/0010-any-team.md) a fan can pick any team,
and BEAR-001 to BEAR-008 hold for that team's guide: read "the Bears" as
the team picked.

The hard part, and the reason the app exists: most games touch the Bears
through more than one channel, and the channels often pull opposite ways.
Take Saints at Bengals in week 12 of 2026:

- The Bears beat the Saints in week 11, so a Saints win raises the Bears'
  strength of victory, a tiebreaker.
- The Saints are an NFC team, so a Saints loss keeps one more NFC team
  behind the Bears in the wild-card race.
- The Bengals are an AFC team, so their result also changes the field the
  Bears could meet in the Super Bowl.

A fan can't weigh those by hand. The app has to, and then say in plain words
which effects exist, which way each one pulls, how big each is and what the
net is, including when the net is close to zero.

## BEAR-001: Every game this week gets a verdict

- Priority: must
- Why: a fan watching any game wants to know whom to cheer for, not only for
  the games the app finds interesting.
- Accept: every regular-season game left in the current week appears once,
  with either a team to root for or "doesn't matter", and the Bears' own game
  is always a pick.

## BEAR-002: A pick says how much it helps

- Priority: must
- Why: a 3-point swing and a 0.03-point swing are different kinds of
  important, and fans want to know which games to really care about.
- Accept: each pick shows the change in the Bears' Super Bowl odds (and
  playoff odds) between the two results, and games are sorted by that size.

## BEAR-003: Every game explains how it reaches the Bears

- Priority: must
- Why: Jim, 2026-11-26: the odds change alone isn't enough; the app is meant
  to sort out the tiebreaker, head-to-head, strength-of-schedule and
  seeding effects for its users.
- Accept: every game, picks and "doesn't matter" alike, lists each channel
  through which it can affect the Bears (division race, wild-card race,
  seeding, head-to-head, division and conference record, common games,
  strength of victory, strength of schedule, likely playoff and Super Bowl
  opponents), with the direction and rough size of each one that is open.

## BEAR-004: Conflicting effects are named and netted

- Priority: must
- Why: Saints at Bengals shows that one result can help through one channel
  and hurt through another; a fan needs to see both and the balance.
- Accept: when channels pull opposite ways, the explanation names each side,
  which team each favors and its size, then the net and which side wins.

## BEAR-005: "Doesn't matter" is earned and explained

- Priority: must
- Why: Jim, 2026-11-26: each "doesn't matter" should still say how the game
  touches the Bears, or why it can't.
- Accept: a game is "doesn't matter" only when its net swing on the Bears'
  Super Bowl odds is confidently under 0.05 points. Its explanation is
  specific to that game: it says which channels are closed and why (for
  example "neither team is on the Bears' schedule" or "both teams are
  eliminated") and how small the open ones are. No two games share the same
  explanation unless their situations are the same.

## BEAR-006: Tiebreaker claims are exact

- Priority: should
- Why: a wrong tiebreaker claim is worse than none, and fans check them.
- Accept: a tiebreaker named in an explanation is the step of the NFL
  procedure that actually separates the Bears from that team in the
  simulations where the game changed the Bears' seed, and the share of
  those simulations is shown.

## BEAR-007: Win chances use the best available odds

- Priority: should
- Why: Jim, 2026-11-26: the model leaned too heavily on Elo.
- Accept: games with a betting line use it; only games without one use Elo,
  and the page says which.

## BEAR-008: The page says how fresh it is

- Priority: must
- Why: the guide is rebuilt from a data file that can lag the games.
- Accept: the page shows when it was generated and the latest game included.

## BEAR-009: NFC North mode

- Priority: could
- Why: decision 0007, an opt-in goal of all four NFC North teams reaching
  the divisional round.
- Accept: a toggle reranks and re-explains the games for that goal.

## BEAR-010: Fans can say when a verdict looks wrong

- Priority: should
- Why: the "doesn't matter" line and the explanations were set by
  judgment, and fans are the ones who can tell (Jim, 2026-11-26;
  [plans/2026-11-fan-feedback.md](plans/2026-11-fan-feedback.md)).
- Accept: every game in every tab has a Yes / No / Confusing vote with an
  optional note, votes reach the repo weekly as files, and the summary
  shows each A/B arm's results.

## BEAR-011: A fan can pick any team

- Priority: should
- Why: Jim, 2026-11-28: "We should also make it so that you can pick any
  team in the bear_down app and get recommendations"
  ([decision 0010](decisions/0010-any-team.md)).
- Accept: the page offers all 32 teams; picking one shows that team's
  guide, built nightly like the Bears', with BEAR-001 to BEAR-008 holding
  for it and every explanation naming that team, never the Bears unless
  they play in the game. The choice is kept in the link and in the
  browser.
