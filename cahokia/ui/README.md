# ui

Dioxus components shared by the cahokia web, desktop and mobile apps.

| Component | What it shows |
| --- | --- |
| `Hero` | The home page banner. |
| `Navbar` | The top navigation bar. |
| `Sidebar` | A slide-out side panel that wraps its children. |
| `FamilyTreeView` | A family tree drawn as inline SVG that the user can pan, zoom and focus on one person in. |
| `FamilyTreePage` | The `/family-tree` page every app routes to: the sample family in a pan and zoom frame. |

## FamilyTreeView

Draws a `core::FamilyTree` with the coordinates from
`FamilyTree::tree_layout_with()`: a box per person with their name and birth
and death years, dashed lines between spouses, and lines from each couple or
single parent down to their children, following the routes `core` works out. Names too long for their box are shortened with `…` and shown
in full as a tooltip. An empty tree shows a short message instead.

```rust
use cahokia_core::FamilyTree;
use dioxus::prelude::*;
use ui::FamilyTreeView;

#[component]
fn TreePage(tree: FamilyTree) -> Element {
    rsx! { FamilyTreeView { tree } }
}
```

The drawing fills the width of its container and starts with the whole tree
fitted in view. The user moves around it like a map:

| To | Mouse or pen | Touch | Keyboard (drawing focused) |
| --- | --- | --- | --- |
| Pan | Drag | Drag with one finger | Arrow keys |
| Zoom | Wheel, or the `+` and `−` buttons | Pinch | `+` and `-` |
| Fit the whole tree | `Fit` button | `Fit` button | `0` |
| Pick a focus person | Click them, or the "Focus on" list | Tap them | Tab to them, then Enter or Space |
| Clear the focus | Click them again, or "No one" | Tap them again | Escape |

The focus person's ancestors and descendants and the lines between them are
outlined in gold and everyone else is dimmed; a line under the drawing says
how many of each there are. Choosing someone from the list also centres the
view on them. The zoom stays between 20% and 400%. All of the maths
(`core::Viewport` for pan and zoom, `FamilyTree::lineage()` and
`core::Lineage` for highlighting) lives in `core`, so every platform behaves
the same.

Pass `spacing` to change the box size and gaps (see `core::Spacing`), and
`initial_focus` to start with someone in focus.

![A three-generation family tree drawn by FamilyTreeView](docs/family-tree-view.png)

The `core` crate is a dependency under the name `cahokia_core`, because a
crate called `core` would shadow Rust's own `core` inside Dioxus macros.
