# design_system/preview

The design system's preview site: one panel per theme, scheme and mode
(and per custom app theme), each pinned to its combination with the
`data-theme`, `data-scheme` and `data-a11y` attributes, showing text, links,
buttons, inputs, focus rings, status colours, surfaces, the data palette,
the type scale and every colour token. The page itself follows your system
settings.

- Build it into `dist/`: `mise run release`
- Build and serve it on <http://localhost:8000>: `mise run dev`
- Deploy (CI does this on pushes to `main`): `mise run deploy`

It is a Cloudflare Pages project named `design-system` on the free plan,
created by `standards/scripts/free-tiers.sh` on the first deploy.
