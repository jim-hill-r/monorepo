# Notes on cahokia/core's issues

## Family tree visualization

Broken down from "Visualize the family tree (this may be platform specific?)".
It does not need to be platform specific: desktop and mobile render Dioxus
through a webview, so one SVG component in the shared `ui` crate can serve web,
desktop and mobile. Keep the layout maths in `core` so every renderer shares it.
