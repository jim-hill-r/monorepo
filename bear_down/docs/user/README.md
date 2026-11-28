# bear_down user guide

## Who it's for

bear_down is for Chicago Bears fans who watch the rest of the NFL on a
Sunday and wonder whom to cheer for. For every game on this week's slate it
tells you which team to root for to give the Bears the best chance of
winning the Super Bowl, and how much that game really matters. Fans of any
other NFL team can pick theirs and get the same guide for their team.

It works this out by playing the rest of the season and the playoffs
100,000 times on a computer, once with each result of each game, and
comparing how often the Bears win it all. The site is rebuilt every night,
so the odds include the latest scores.

## Getting started

1. Open <https://jimhillr-bear-down.pages.dev> in a browser on your phone,
   tablet or computer. Nothing needs installing and there is no sign-in.
2. The top of the page shows the Bears' record, the week and season, and
   their odds of winning the Super Bowl, making the playoffs, reaching the
   divisional round, and all four NFC North teams reaching the divisional
   round. When the Bears' most recent game was a win, a banner above
   celebrates it with the score and a line or two about where things stand
   now, such as their place in the NFC North, how much the win moved their
   Super Bowl odds, a winning streak or the next playoff round. It is
   rebuilt every night, and disappears after a loss.
3. Below that is this week's list of games, the most important first.

The page opens on the Bears. To follow another team, see "How to pick your
team" below.

## How to pick your team

At the top of the page, **Your team** lists all 32 NFL teams. Choose yours
and the page shows that team's guide: its record and odds, and which games
to root for to help that team win the Super Bowl or make the playoffs,
with every explanation written for that team.

- Your browser remembers the team, so the page opens on it next time.
- The address changes to include the team (for example `?team=DET` for
  the Lions), so you can bookmark it or send the link to a friend and it
  opens on that team.
- The look stays the Bears' navy and orange for every team (you can change
  the colours under Appearance).
- The **NFC North** button and the "all four NFC North teams" odds appear
  only for the Bears, Lions, Packers and Vikings.
- Vote buttons and **Past weeks** are on the Bears' guide only for now.

## How to read this week's rooting list

Each game in the list shows:

- **The matchup**, written "away team at home team", with the game day and
  the home team's chance of winning.
- **The verdict:** "Root for the Packers", for example, with how much that
  result helps, such as "+0.19% Super Bowl odds". That number is the
  difference in the Bears' chance between the two results of the game, so
  "+1.8%" means the Bears' Super Bowl chance is 1.8 points higher if your
  team wins than if it loses. On the Super Bowl list, a pick also shows how
  much it moves the Bears' playoff odds, which are bigger numbers and easier
  to feel.
- **Why**, in a few short lines: how the game reaches the Bears (a division
  rival, the wild-card race, a tiebreaker such as strength of victory, or
  who the Bears might meet in the Super Bowl), which way each effect pulls
  and how big it is, and the net when they pull opposite ways.

Games are sorted by the size of the swing, so the ones at the top are the
ones to really care about. A game with no pick says one of:

- **Doesn't matter:** the result changes the Bears' odds by less than about
  0.05 points either way. Enjoy the game.
- **Too close to call:** there is a tiny edge, but it is too small for the
  simulation to be sure which way it goes.
- **Can't matter:** neither result can change anything for the Bears this
  season.

If the Bears have been eliminated, the page says so instead of listing
games. The footer says when the page was last updated, how many seasons it
simulated, and the latest game it includes.

## How to change what the games are ranked by

Above the list are three buttons (two for a team outside the NFC North).
The one that's highlighted is in use:

- **Bears Super Bowl** (the default; it carries your team's name when you
  pick another team): ranks games by how much they move the Bears' chance
  of winning the Super Bowl.
- **Make the playoffs:** ranks games by how much they move the Bears'
  chance of making the playoffs at all. Use it when you just want to get
  in.
- **NFC North** (for NFC North teams only): ranks games by how much they
  move the chance that the Bears, Lions, Packers and Vikings all reach the
  divisional round. It's a long shot for division pride, so many games show
  "Doesn't matter" here.

The line under the buttons says which goal the list is using. Your choice
resets to Bears Super Bowl when you reload the page.

## How to vote on a verdict

Under each game is "Does this verdict look right?" with three buttons:

1. Press **Yes** if it looks right, **No** if you think it's wrong, or
   **Confusing** if you can't tell what it means.
2. A box appears asking "What's off? (optional)". Type a short note if you
   like and press **Send**, or just leave it.
3. The game then says "Thanks for your vote." Your browser remembers that
   you voted, so you vote once per game, per week and per ranking.

Votes help us find verdicts that are wrong or badly explained. A vote sends
the game, the verdict shown, your answer and any note to this site's own
server: no cookies, no sign-in and nothing about who you are. If your
browser has Do Not Track or Global Privacy Control turned on, the vote
buttons don't appear at all.

## How to look back at a past week

Under the list, **Past weeks** lets you pick an earlier week of the season
(it appears once the site has some to show). Choose one and the page shows
that week's guide in the same layout, from the results known before that
week: the Bears' odds then and which games to root for. A note at the top
says which week you're looking at, and **Back to this week** (or **This
week** in the picker) returns to the current guide.

A past week is rebuilt every night with today's model, so it can differ a
little from what the page said that week, for example after an
improvement to the odds or the explanations. Past weeks have no vote
buttons and no celebration banner.

## How to change the colours

At the bottom of the page, **Appearance** has three choices: **Theme**
(the Chicago Bears navy and orange, or one of the standard themes),
**Colours** (light, dark, or match your device) and **Accessible mode**
(stronger contrast, or match your device). Your browser remembers them.

## Getting help

- **"Loading this week's games…" never goes away, or the page says it
  couldn't load the rooting guide.** Reload the page. bear_down needs a
  recent Chrome, Edge, Firefox or Safari with JavaScript turned on.
- **The odds don't include last night's game.** The site rebuilds every
  night, so a game is included the morning after it's played. The footer
  says which game is the latest one included.
- **"No games this week."** There are no games left to play in the week
  the page is showing, such as between the end of the regular season and
  the next update. Check back after the nightly update.
- **I can't see the vote buttons.** Your browser has Do Not Track or Global
  Privacy Control on, so voting is off. Everything else still works.
- **The page shows a different team than I expected.** It opens on the
  team in the link, or else the team you picked last in this browser.
  Choose yours under **Your team**.
- **A friend sees a slightly different verdict on a close game.** We're
  testing two "doesn't matter" lines, so some browsers call a few more
  tiny edges than others. The big games are the same for everyone.

To report a problem or a verdict you disagree with, use the **No** or
**Confusing** vote on that game and say what's off in the note. For
anything else, tell the person who shared bear_down with you, with what you
saw and on which device.
