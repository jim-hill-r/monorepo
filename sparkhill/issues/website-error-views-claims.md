---
kind: fix
priority: priority
tags: [agent-generated]
added: 2026-11-27
---

`website/src/error_views.rs` is still the Perseus example: its panic view
tells visitors the error "has been automatically reported to our team",
but the site reports nothing (it is static files with no server or error
tracking), and the 4xx/5xx views talk about "our server", which doesn't
exist. Rewrite the messages so they are true (suggest reloading, link to the
home page and the Contact Us page), drop the copied tutorial comments, and
add unit tests that render each view, as `templates/index.rs` does for the
home page.
