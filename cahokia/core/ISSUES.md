# Priority Issues

## Family tree visualization

Broken down from "Visualize the family tree (this may be platform specific?)".
It does not need to be platform specific: desktop and mobile render Dioxus
through a webview, so one SVG component in the shared `ui` crate can serve web,
desktop and mobile. Keep the layout maths in `core` so every renderer shares it.

- TODO (agent-generated): [Phase 3b] core, ui - Route parent-child lines per family instead of per parent. Today every parent-child line bends at the middle of the row gap, so when two couples' children sit side by side their bars join into one line and you can't tell whose children are whose. Draw one line down from the middle of each couple's spouse line (or from a single parent), give each family its own bar height in the gap, and drop from that bar to each child. Keep the routing in `core` next to `tree_layout`.
- TODO (agent-generated): [Phase 4] web - Add a family tree route that shows `FamilyTreeView` with a sample tree, link it from the sidebar, and add a Playwright test that checks people and links render.
- TODO (agent-generated): [Phase 5] desktop, mobile - Add the same family tree route to the desktop and mobile apps.
- TODO (agent-generated): [Phase 6] ui - Let the user pan and zoom the tree and pick a focus person whose ancestors and descendants are highlighted.

TODO (agent-generated): [CFG-001] core - Project 'core' is missing royalty.toml. All projects MUST include a royalty.toml for use with the royalty project.

TODO (agent-generated): [DOC-002] core - Project 'core' README.md should include a section with the project name (core) which includes a short description.

TODO (agent-generated): [DOC-003] core - Project 'core' is missing CONTRIBUTING.md. All projects MUST include a CONTRIBUTING.md.
# Backlog

