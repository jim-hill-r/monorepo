---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

The website's Perseus template (`website/src/templates/index.rs`) renders only
the tagline. The full page (nav, "What we do", "Who we are", testimonials,
footer) is a static Astro export in `website/static/index.html` that Perseus
serves only as a file under `.perseus/static/`. Port it into the template.
While there: its "Contact Us" links go to a `/contact` page that doesn't
exist, two sections share the id `how-we-do-it`, and the testimonials are
placeholder text ("Ms. Smith", "Ms. Jones") that a person should replace or
remove.
