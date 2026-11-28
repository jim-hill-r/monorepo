# Homer mission

Written 2026-11-28 from Jim's ask (quoted in the [roadmap](roadmap.md)).
Changing this file needs Jim's review.

## Mission

Help anyone house hunting see, in seconds, whether a house fits what they
are looking for, and why it doesn't.

## Vision

homer is a small, free web app on `homer.jimhillr.com`. A house hunter
writes down their must-haves once (bedrooms, bathrooms, price, size), then
checks each house they find against them and keeps a shortlist to compare
side by side. The scoring runs in their browser, their preferences and
shortlist stay in their browser, and listing data is fetched with their own
free RapidAPI key, never ours, so homer costs nothing to run however many
people use it.

Three things follow from that:

1. **The web app is the product.** The CLI stays, for scripts and evals,
   but every new feature lands in the web app first.
2. **We never hold or resell Zillow data.** homer shows a listing to the
   person who looked it up and links back to Zillow; it keeps no copy on a
   server and never serves one person's lookups to another.
3. **Free at any scale.** No accounts, no database and no server-side key
   until a decision record says why one is worth it.
