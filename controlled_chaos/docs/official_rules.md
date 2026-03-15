# Controlled Chaos – Official Rules

This document codifies the rules as implemented in the game software.  It is
the authoritative reference for the game mechanics.  See `rulebook.md` for the
design brainstorming document that informed these rules.

---

## Overview

Controlled Chaos is a two-player turn-based card game.  Players compete to
build the happiest society without succumbing to existential crisis or losing
all of their civilians.

---

## Components

### Action Deck (52 cards)

| Category     | Count | Colour |
|--------------|-------|--------|
| Technology   | 13    | Blue   |
| Government   | 13    | Red    |
| Environment  | 13    | Green  |
| Economy      | 13    | Yellow |

### Crisis Deck (13 cards)

Problems that affect all players.  Each crisis card has a matching society
card that can avert it.

### Profession Deck (13 cards)

Roles that a player can adopt.  Each player is dealt cards at setup and
discards down to one.

### Civilian Cards (13 cards)

Support cards for a player's community.  Each civilian card has a population
value and an upkeep cost.

### Society Cards (13 cards)

Benefit all communities when played.  Required to prevent crisis cards.

---

## Setup

1. Shuffle each deck separately.
2. Slide one random crisis card approximately halfway through the action deck.
3. Place one crisis card on the bottom of the action deck.
4. Remove one society card at random and exclude it from the game.
5. Deal each player three profession cards; each player discards one (excluded
   from the game).
6. Deal each player three action cards.
7. Each player begins with **two civilian cards** (one face-up, one face-down)
   and a **happiness score of 50**.

---

## Turn Structure

Each player's turn consists of four phases in order:

### 1. Draw Phase

- Draw a card from the action deck into your hand (if you have five or fewer
  cards).

### 2. Play Phase

- Choose one card from your hand to play as the active card for this turn.

### 3. Battle Phase

- The active player's card is compared against the defending player's card by
  value.
- **Attacker wins**: defender's card has a lower value → defender loses 1 life point.
- **Defender wins**: attacker's card has a lower value → attacker loses 1 life point.
- **Draw**: equal values → no life points are lost.
- Winning a battle grants a small happiness boost (+5) to the winning player.

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

A player **loses immediately** when their civilian count reaches **zero**.

### Game End — Deck Exhaustion

When the action deck runs out:

- The game ends immediately.
- The player with the **highest happiness score wins**.
- If both players have the same happiness score the result is a **draw**.

---

## Happiness

- Happiness is tracked per-player on a scale of **0–100**, starting at **50**.
- Winning a battle: **+5 happiness** for the winning player.
- Other card effects may raise or lower happiness as specified on the card.
- Happiness cannot go below **0** or above **100**.

---

## Civilians

- Each player starts with **2 civilians**.
- Civilians are lost through certain card effects.
- A player with **0 civilians** loses automatically.
- Additional civilians can be gained through card effects.
