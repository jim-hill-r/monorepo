---
kind: todo
priority: priority
tags: [agent-generated]
epic: Blue Eel parity
order: 20
added: 2026-11-27
---

Make the app installable like the live one. blue.eel.education has a web app
manifest and home-screen icons (`statics/icons/`, an
`apple-mobile-web-app-title` and a Windows tile), so it opens full screen
from a phone's home screen. Add a `manifest.webmanifest` (name "Blue Eel",
theme colour `#178CA4`, background `#F9F7F0`, `display: standalone`) and
icons, linked from `index.html`, and check it with a Playwright test.
