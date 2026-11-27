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
| Browser | <https://jimhillr-controlled-chaos.pages.dev>. Tap or click to play; on a keyboard, click the page once so it gets the keys. Redeployed on every merge to `main` that changes the game, and every night. |
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

## Look and display settings

The game uses the design system's **gamey** theme
([design_system](../design_system/README.md)) through the
`design_system_palette` crate: every colour on screen is a token, so the
design system's contrast checks cover the game too. The eight card
categories are drawn in the theme's categorical colours, `data-1`
(Technology) to `data-8` (Society), as the card frame and artwork block;
card text sits on `surface`, never on the category colour.

The start screen has two settings:

- **Colours: Dark / Light** switches the colour scheme. Dark is the default.
- **Accessible: Off / On** turns on accessible mode: WCAG AAA contrast,
  text one step larger, and the design system's accessible palette. It
  starts on when nothing is saved yet and the browser asks for more
  contrast (`prefers-contrast: more`). Bevy doesn't expose that setting on
  desktop or Android, so there it starts off.

The game remembers the choice between launches, in a few lines of TOML
kept on the player's device, never on a server:

| Platform | Where |
| --- | --- |
| Web | the browser's `localStorage`, key `controlled_chaos.settings` |
| macOS | `~/Library/Application Support/controlled_chaos/settings.toml` |
| Linux | `$XDG_CONFIG_HOME/controlled_chaos/settings.toml` (or `~/.config/...`) |
| Windows | `%APPDATA%\controlled_chaos\settings.toml` |
| Android | `settings.toml` in the app's internal data directory |

Delete the file (or clear the site's data) to go back to the defaults. A
setting the game can't read keeps its default, so a damaged file never
stops the game from starting.

`src/theme.rs` holds the choice (`GameTheme`) and has the tests that check
every button and text colour against its background in all four
combinations. `src/settings.rs` loads and saves it.

## Development

```bash
# Run with hot reloading
cargo run --features bevy/dynamic_linking

# Run tests
cargo test
```

### Screenshots of the web build on phone-sized screens

Playwright's `deviceScaleFactor` (and Chrome DevTools' device mode) only
fakes `devicePixelRatio`: the canvas's `devicePixelContentBoxSize`, which
winit sizes the game from, still uses the real screen's ratio. Bevy then
lays the game out at half size and you see its top-left quarter, zoomed in.
Real phones and Retina screens don't do this. To check a high-DPI screen
headlessly, make the ratio real by launching Chromium with
`--force-device-scale-factor=<ratio>` as well as setting `deviceScaleFactor`
to the same value.
