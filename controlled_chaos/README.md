# controlled_chaos

A digital tabletop card game built with [Bevy](https://bevyengine.org/), featuring turn-based card game mechanics and deck-building gameplay.

## Overview

Controlled Chaos is a digital tabletop card game where players build decks and compete against each other. The game uses Bevy's ECS architecture to manage game state, cards, and player interactions.

## Running

```bash
cargo run
```

## Playtesting

| Where | How |
| --- | --- |
| Browser | <https://controlled-chaos-5v4.pages.dev>. Tap or click to play; on a keyboard, click the page once so it gets the keys. On phones and other high-DPI screens the page is zoomed in for now (see [issues/](issues/)). Redeployed on every merge to `main` that changes the game, and every night. |
| macOS | [controlled_chaos-macos.zip](https://github.com/jimhillr/monorepo/releases/download/controlled_chaos-nightly/controlled_chaos-macos.zip). Unzip and open "Controlled Chaos". It isn't notarized, so the first time macOS refuses: open System Settings > Privacy & Security, click **Open Anyway**, and confirm. (Or run `xattr -dr com.apple.quarantine "Controlled Chaos.app"`.) |
| Android | [controlled_chaos-android.apk](https://github.com/jimhillr/monorepo/releases/download/controlled_chaos-nightly/controlled_chaos-android.apk). Open the link on the phone and allow your browser to install unknown apps. Until a release key is set up, uninstall the old build before installing a new one. |
| Your computer | `mise run dev` (or `cargo run`) in this folder. |

The macOS and Android builds come from the
[controlled_chaos-nightly release](https://github.com/jimhillr/monorepo/releases/tag/controlled_chaos-nightly),
rebuilt at 2:30 AM Pacific each night from the latest `main` that passed CI
(the Nightly downloads workflow; run it by hand from the Actions tab for a
build now). Build them yourself with `mise run release:web`,
`mise run release:macos` (on a Mac) or `mise run release:android` (needs the
Android SDK and NDK). See "Game builds" in `standards/docs/mise.md`.

Controls: tap or click the cards in your hand to play or to defend with
one, and the **Draw card** and **End turn** buttons to move the turn on. The
keyboard works too: **Space** draws a card and ends your turn, and **1** to
**8** pick a card from your hand. The game shows what to do next on screen.

## Development

```bash
# Run with hot reloading
cargo run --features bevy/dynamic_linking

# Run tests
cargo test
```
