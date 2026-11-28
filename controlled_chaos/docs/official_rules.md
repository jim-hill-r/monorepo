# Controlled Chaos – Official Rules

This document codifies the rules as implemented in the game software.  It is
the authoritative reference for the game mechanics: when the game and this
file disagree, one of them has a bug.  See `rulebook.md` for the design
brainstorming document that informed these rules; the parts of it the game
doesn't play yet are listed under [Not built yet](#not-built-yet).

---

## Overview

Controlled Chaos is a two-player turn-based card game.  Players compete to
build the happiest society without losing all of their life or civilians.
In the game you play against the CPU.

---

## Components

### Action Deck (18 cards)

One shared deck of 14 different cards, with at most two copies of any card
(`src/card_library.rs`).  Each card has a **value** from 1 to 14 and a
**category**, which sets its colour on screen.

| Card          | Value | Category    | Copies |
|---------------|-------|-------------|--------|
| Ace of Spades | 14    | Technology  | 2      |
| NASA          | 13    | Government  | 2      |
| Biodome       | 12    | Environment | 2      |
| Stock Market  | 11    | Economy     | 2      |
| Asteroid      | 10    | Crisis      | 1      |
| Engineer      | 9     | Profession  | 1      |
| Suburb        | 8     | Civilian    | 1      |
| Space Station | 7     | Society     | 1      |
| Solar Panel   | 6     | Technology  | 1      |
| City Hall     | 5     | Government  | 1      |
| Forest        | 4     | Environment | 1      |
| Trade Route   | 3     | Economy     | 1      |
| Flood         | 2     | Crisis      | 1      |
| Farmer        | 1     | Profession  | 1      |

Every card plays the same way, by its value.  The Crisis, Profession,
Civilian and Society categories have no effects of their own yet.

---

## Setup

1. Shuffle the action deck.  A game started with a seed deals the same
   cards every time.
2. Deal each player three action cards.
3. Each player begins with **20 life points**, **two civilians** and a
   **happiness score of 50**.
4. Player 1 takes the first turn.

---

## Turn Structure

Each player's turn consists of four phases in order:

### 1. Draw Phase

- Draw a card from the action deck into your hand if you have five or fewer
  cards.
- With six or more cards in hand you skip the draw, and the turn moves on to
  the Play phase.
- If you may draw and the action deck is empty, the game ends (see
  [Deck Exhaustion](#game-end--deck-exhaustion)).

### 2. Play Phase

- Choose one card from your hand to play as the active (attacking) card for
  this turn.

### 3. Battle Phase

- The defending player chooses one card from their hand to defend with.  A
  defender with no cards in hand defends with a value of 0, so the attacker
  wins.
- The active player's card is compared against the defending player's card by
  value.
- **Attacker wins**: defender's card has a lower value → defender loses 1 life point.
- **Defender wins**: attacker's card has a lower value → attacker loses 1 life point.
- **Draw**: equal values → no life points are lost.
- Winning a battle grants a small happiness boost (+5) to the winning player.
- Both played cards leave the game.

> **Note**: *Life points* and *civilians* are separate resources.  Losing battles
> depletes life points.  Losing all civilians (through card effects) triggers an
> immediate loss regardless of life points remaining.  A player is considered
> alive as long as they have at least 1 life point **and** at least 1 civilian.

### 4. End Phase

- Control passes to the other player.
- The round counter increments.

---

## Win and Loss Conditions

### Immediate Loss

A player **loses immediately** when their life points or their civilian
count reaches **zero**.  This is checked before deck exhaustion.

### Game End — Deck Exhaustion

When a player may draw but the action deck is empty:

- The game ends immediately.
- The player with the **highest happiness score wins**.
- If both players have the same happiness score the result is a **draw**.

---

## Happiness

- Happiness is tracked per-player on a scale of **0–100**, starting at **50**.
- Winning a battle: **+5 happiness** for the winning player.
- Happiness cannot go below **0** or above **100**.

---

## Civilians

- Each player starts with **2 civilians**.
- A player with **0 civilians** loses automatically.
- No card gains or loses civilians yet.

---

## The CPU opponent

- The CPU attacks with its highest-value card.
- It defends with the lowest-value card that beats the attacker, or its
  lowest card when none does.
- Its hand is never shown to the player.

---

## Not built yet

These rules are in `rulebook.md` but the game doesn't play them.  Each has an
issue file in [`../issues/`](../issues/):

- **Crisis and society cards**: one crisis card shuffled halfway into the
  deck and one at the bottom, one society card removed at random, and a
  crisis that ends the game for everyone unless its matching society card
  averts it (`rules-crisis-and-society-cards.md`).
- **Profession cards**: each player is dealt three and keeps one
  (`rules-profession-cards.md`).
- **Civilian cards**: population, upkeep and card effects that gain or lose
  civilians and happiness (`rules-civilian-cards.md`).
- **The full 52-card action deck**, with each card's action, passive and
  power-ups (`rules-full-action-deck.md`).
