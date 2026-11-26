---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-26
---

The web build is zoomed in 2x and cropped on high-DPI screens (phones, Retina
Macs), so the Start Game button is off screen. Seen in headless Chromium at
800x360 with `devicePixelRatio` 2: the canvas's drawing buffer stays
800x360 (CSS pixels) while Bevy uses a scale factor of 2, so the UI is laid
out for 400x180 logical pixels and only its top-left quarter shows. At a
ratio of 1 it renders correctly. Likely Bevy 0.14's `fit_canvas_to_parent`
on winit 0.30 (`src/lib.rs`); check a real phone browser, then fix it (for
example by upgrading Bevy, or keeping the canvas buffer at CSS size times
`devicePixelRatio`). The Android APK doesn't use the canvas and isn't affected.
