---
added: 2026-11-27
runbook: standards/docs/runbooks/playtest-controlled-chaos.md
---

## Requirement

Try the three nightly playtest builds of controlled_chaos (browser, macOS app, Android APK) on real devices, and report or file a `fix` issue in `controlled_chaos/issues/` for anything that doesn't run or play right. The Android APK is the first one with touch controls (#698).

## Why it can't be automated

CI only proves the builds compile and that the web build starts in headless Chromium. Whether the game plays right by touch on a real phone, or opens on a Mac (it isn't notarized, so macOS asks a person to approve it), needs a person holding the device.

## Runbook

[Playtest the controlled_chaos builds](../../standards/docs/runbooks/playtest-controlled-chaos.md)
