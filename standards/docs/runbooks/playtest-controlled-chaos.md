# Playtest the controlled_chaos builds

- **Kind:** on demand (after a change players will notice, such as new controls or a new platform)
- **Status:** open (issues/human/playtest-controlled-chaos-builds.md)
- **Time:** about 20 minutes
- **Cost:** $0 (Cloudflare Pages and GitHub Releases free tiers; the builds already exist)

## What it's for

Confirms that the nightly builds of controlled_chaos start and can be played
on real devices: a browser, a Mac and an Android phone. CI proves they build
and that the web build starts in headless Chromium. It can't tell whether
the game is fun or usable by touch on a real screen.

## Why a person

Judging whether a game plays right on real hardware needs a person holding
the device. CI has no Android phone or signed-in Mac, and macOS asks a
person to approve an un-notarized app (notarization needs a paid Apple
account, which the free-tiers rule rules out).

## Before you start

- A browser, a Mac (Apple silicon or Intel, macOS 11 or later) and an
  Android phone (Android 8 or later).
- Nothing to sign in to: the links are public.

## Steps

1. **Browser:** open the `DEPLOY_URL` in `controlled_chaos/mise.toml`
   (`https://jimhillr-controlled-chaos.pages.dev` once the jimhillr- rename
   is live). Click the page once so it gets the keyboard, then play a round.
   Try it on the phone's browser too.
2. **macOS:** download
   [controlled_chaos-macos.zip](https://github.com/jimhillr/monorepo/releases/download/controlled_chaos-nightly/controlled_chaos-macos.zip),
   unzip it and open "Controlled Chaos". The first time macOS refuses: open
   System Settings > Privacy & Security, click **Open Anyway**, and confirm.
   Play a round.
3. **Android:** on the phone, open
   [controlled_chaos-android.apk](https://github.com/jimhillr/monorepo/releases/download/controlled_chaos-nightly/controlled_chaos-android.apk),
   allow the browser to install unknown apps, and install it. Uninstall an
   older build first unless the release key is set up
   ([android-release-key.md](android-release-key.md)). Play a round by touch.
4. For anything that doesn't start, looks wrong or can't be played, tell an
   agent thread what you saw and on which device, or add a `kind: fix` file
   in `controlled_chaos/issues/`.

## Where secrets go

None: playtesting makes no secret.

## Verify

Each of the three builds started and you finished a round. Tell any thread
"done" (with any problems found) so it deletes the issue file and files the
fixes.

## Undo or rotate

None: playtesting changes nothing. Uninstall the app from the phone or
delete the Mac app when you're finished.
