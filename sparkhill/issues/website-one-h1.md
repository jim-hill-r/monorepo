---
kind: fix
priority: backlog
tags: [agent-generated]
added: 2026-11-27
---

The website's home page has five `<h1>` elements: the tagline and each
section heading ("What we do", "Who we are", "How we do it",
"Testimonials"). A page should have one `h1`, with sections as `h2`, so
screen readers and search engines see its outline (WCAG 1.3.1). Make the
section headings `h2` with the same look, and update the unit test and the
`e2e/home.spec.ts` hydration test that count five level-1 headings.
