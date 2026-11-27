//! An SVG drawing of a [`FamilyTree`] that the user can pan, zoom and focus.
//!
//! [`FamilyTreeView`] draws the boxes and lines that
//! [`FamilyTree::tree_layout_with`] computes, so web, desktop and mobile show
//! the same tree. All layout, pan, zoom and lineage maths stays in `core`;
//! this module only turns it into SVG elements, labels and input handling.

use std::collections::HashMap;
use std::rc::Rc;

use cahokia_core::{
    Edge, EdgeKind, FamilyTree, LineageRole, Node, Person, Point, Sex, Spacing, Viewport,
};
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

/// Space around the drawing, so box borders and lines are not clipped.
const MARGIN: f64 = 16.0;
/// Font size of a person's name, in layout units.
const NAME_FONT_SIZE: f64 = 15.0;
/// Font size of a person's birth and death years, in layout units.
const YEARS_FONT_SIZE: f64 = 13.0;
/// Rough width of one character relative to the font size, used to decide
/// when a name needs shortening to fit its box.
const CHAR_WIDTH_RATIO: f64 = 0.56;
/// Room left between a name and the sides of its box.
const TEXT_PADDING: f64 = 10.0;

/// Distance in pixels a pointer may move between press and release and
/// still count as a click rather than a drag.
const CLICK_SLOP: f64 = 5.0;
/// How far the arrow keys pan the tree, in pixels.
const KEY_PAN_STEP: f64 = 60.0;
/// How much the zoom buttons and the `+` and `-` keys zoom in or out.
const ZOOM_STEP: f64 = 1.25;
/// How much one pixel of mouse wheel scrolling zooms.
const WHEEL_ZOOM_RATE: f64 = 0.0015;
/// Lowest and highest height of the drawing area, in pixels.
const MIN_VIEW_HEIGHT: f64 = 240.0;
const MAX_VIEW_HEIGHT: f64 = 640.0;

// Colours are the design system's --ds- tokens, so the tree follows the
// page's theme, light or dark, and accessible mode. They are set through
// `style` because SVG presentation attributes don't read CSS variables.
const BACKGROUND: &str = "var(--ds-color-surface)";
const LINE_COLOUR: &str = "var(--ds-color-border-strong)";
const HIGHLIGHT_COLOUR: &str = "var(--ds-color-accent)";
const TEXT_COLOUR: &str = "var(--ds-color-text)";
const MUTED_TEXT_COLOUR: &str = "var(--ds-color-text-muted)";
/// A box's fill: the surface tinted with one of the data colours, light
/// enough that the name and years keep their contrast.
const fn person_fill(sex: &Sex) -> &'static str {
    match sex {
        Sex::Male => {
            "color-mix(in srgb, var(--ds-color-data-4) 16%, var(--ds-color-surface-raised))"
        }
        Sex::Female => {
            "color-mix(in srgb, var(--ds-color-data-5) 16%, var(--ds-color-surface-raised))"
        }
        Sex::Unknown => {
            "color-mix(in srgb, var(--ds-color-data-3) 16%, var(--ds-color-surface-raised))"
        }
    }
}
/// Opacity of people and lines outside the focus person's lineage.
const DIMMED_OPACITY: &str = "0.3";

/// Draws a family tree as inline SVG that the user can pan and zoom: a box
/// per person with their name and birth and death years, and lines between
/// parents and children and between spouses.
///
/// - Drag (mouse, pen or one finger) to move the tree, and use the mouse
///   wheel, a two-finger pinch, the zoom buttons or the `+` and `-` keys to
///   zoom. The arrow keys pan and `0` fits the whole tree in view.
/// - Click or tap a person, press Enter on them, or choose them from the
///   focus list to make them the focus person. Their ancestors and
///   descendants and the lines between them are highlighted and everyone
///   else is dimmed. Click them again, choose "No one" or press Escape to
///   clear it.
///
/// It starts with the whole tree fitted in view. The pan, zoom and
/// highlighting maths lives in `cahokia_core` ([`Viewport`] and
/// [`cahokia_core::Lineage`]).
///
/// # Example
/// ```
/// # use dioxus::prelude::*;
/// # use ui::FamilyTreeView;
/// # use cahokia_core::FamilyTree;
/// #[component]
/// fn TreePage(tree: FamilyTree) -> Element {
///     rsx! { FamilyTreeView { tree } }
/// }
/// ```
#[component]
pub fn FamilyTreeView(
    /// The tree to draw.
    tree: FamilyTree,
    /// Box size and gaps; defaults to [`Spacing::default`].
    #[props(default)]
    spacing: Spacing,
    /// Person to focus on when the view first shows, if any.
    #[props(default)]
    initial_focus: Option<String>,
) -> Element {
    let layout = tree.tree_layout_with(spacing);
    let content_width = layout.width() + 2.0 * MARGIN;
    let content_height = layout.height() + 2.0 * MARGIN;

    let mut viewport = use_signal(|| Viewport::at(MARGIN, MARGIN));
    let mut focus = use_signal(move || initial_focus);
    // Pointers that are down, by id, at their last position on screen.
    let mut pointers = use_signal(HashMap::<i32, Point>::new);
    // How far the pointers have moved since the first went down.
    let mut travel = use_signal(|| 0.0_f64);
    let mut drawing = use_signal(|| None::<Rc<MountedData>>);
    // Top left of the drawing on screen and its size, once known.
    let mut origin = use_signal(|| Point { x: 0.0, y: 0.0 });
    let mut view_size = use_signal(|| None::<(f64, f64)>);
    // Whether the user has moved or zoomed the tree since it was last
    // fitted. Until they do, it is fitted again whenever the drawing
    // changes size, such as when the page's styles load or a phone turns.
    let mut moved_by_user = use_signal(|| false);

    if layout.nodes().is_empty() {
        return rsx! {
            p { class: "family-tree-empty", "No people in this family tree yet." }
        };
    }

    let lineage = focus.read().as_deref().and_then(|id| tree.lineage(id));
    let view = viewport();
    let view_height = content_height.clamp(MIN_VIEW_HEIGHT, MAX_VIEW_HEIGHT);
    let zoom_percent = (view.scale * 100.0).round();

    // The middle of the drawing on screen, for the zoom buttons and keys.
    let middle = move || {
        let (width, height) = view_size().unwrap_or((content_width, view_height));
        Point {
            x: width / 2.0,
            y: height / 2.0,
        }
    };
    let mut fit = move || {
        moved_by_user.set(false);
        if let Some((width, height)) = view_size() {
            viewport.set(Viewport::fit(
                content_width - 2.0 * MARGIN,
                content_height - 2.0 * MARGIN,
                width,
                height,
                MARGIN,
            ));
        } else {
            viewport.set(Viewport::at(MARGIN, MARGIN));
        }
    };
    let mut move_to = move |next: Viewport| {
        moved_by_user.set(true);
        viewport.set(next);
    };
    // Re-reads where the drawing is on screen, as the page may have
    // scrolled or resized since.
    let measure = move || async move {
        let element = drawing()?;
        let rect = element.get_client_rect().await.ok()?;
        origin.set(Point {
            x: rect.origin.x,
            y: rect.origin.y,
        });
        view_size.set(Some((rect.size.width, rect.size.height)));
        Some(())
    };

    let pick = move |person_id: String| {
        if travel() > CLICK_SLOP {
            return;
        }
        let same = focus.read().as_deref() == Some(person_id.as_str());
        focus.set(if same { None } else { Some(person_id) });
    };

    let mut people: Vec<&Person> = tree.people.values().collect();
    people.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));
    let status = match (&lineage, focus.read().as_deref()) {
        (Some(lineage), _) => {
            let name = tree
                .get_person(lineage.focus())
                .map_or(lineage.focus(), |person| person.name.as_str());
            format!(
                "Highlighting {name}'s {} and {}.",
                count(lineage.ancestor_count(), "ancestor", "ancestors"),
                count(lineage.descendant_count(), "descendant", "descendants"),
            )
        }
        _ => "Click or tap a person to highlight their ancestors and descendants.".to_string(),
    };
    let (dimmed_edges, lineage_edges): (Vec<&Edge>, Vec<&Edge>) = layout
        .edges()
        .iter()
        .partition(|edge| !lineage.as_ref().is_some_and(|l| l.contains_edge(edge)));
    let has_focus = lineage.is_some();

    rsx! {
        div { class: "family-tree-view",
            div {
                class: "family-tree-toolbar",
                style: "display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin-bottom: 8px;",
                button {
                    r#type: "button",
                    class: "family-tree-zoom-in",
                    "aria-label": "Zoom in",
                    onclick: move |_| move_to(viewport().zoomed_at(ZOOM_STEP, middle())),
                    "+"
                }
                button {
                    r#type: "button",
                    class: "family-tree-zoom-out",
                    "aria-label": "Zoom out",
                    onclick: move |_| move_to(viewport().zoomed_at(1.0 / ZOOM_STEP, middle())),
                    "−"
                }
                button {
                    r#type: "button",
                    class: "family-tree-fit",
                    onclick: move |_| fit(),
                    "Fit"
                }
                span { class: "family-tree-zoom-level", "aria-live": "polite", "{zoom_percent}%" }
                label {
                    "Focus on "
                    select {
                        class: "family-tree-focus",
                        onchange: move |evt| {
                            let id = evt.value();
                            if id.is_empty() {
                                focus.set(None);
                                return;
                            }
                            if let Some(node) = tree.tree_layout_with(spacing).node(&id) {
                                let (width, height) = view_size().unwrap_or((content_width, view_height));
                                move_to(viewport().centred_on(node.center(), width, height));
                            }
                            focus.set(Some(id));
                        },
                        option { value: "", selected: !has_focus, "No one" }
                        for person in people {
                            option {
                                key: "{person.id}",
                                value: "{person.id}",
                                selected: focus.read().as_deref() == Some(person.id.as_str()),
                                "{person.name}"
                            }
                        }
                    }
                }
            }
            svg {
                class: "family-tree",
                xmlns: "http://www.w3.org/2000/svg",
                // A group, not an image: an image hides the person buttons inside it.
                role: "group",
                "aria-label": "Family tree",
                tabindex: "0",
                width: "100%",
                height: "{view_height}",
                style: "display: block; background-color: {BACKGROUND}; border: var(--ds-border-width) solid var(--ds-color-border); font-family: var(--ds-font-body); touch-action: none; cursor: grab; user-select: none; -webkit-user-select: none; max-height: 75vh;",
                onmounted: move |evt| async move {
                    drawing.set(Some(evt.data()));
                    if measure().await.is_some() {
                        fit();
                    }
                },
                onresize: move |evt| {
                    if let Ok(size) = evt.data().get_border_box_size() {
                        view_size.set(Some((size.width, size.height)));
                        if !moved_by_user() {
                            fit();
                        }
                    }
                },
                onpointerdown: move |evt| async move {
                    let point = client_point(&evt);
                    if pointers.read().is_empty() {
                        travel.set(0.0);
                    }
                    pointers.write().insert(evt.pointer_id(), point);
                    measure().await;
                },
                onpointermove: move |evt| {
                    let id = evt.pointer_id();
                    let Some(last) = pointers.read().get(&id).copied() else {
                        return;
                    };
                    let point = client_point(&evt);
                    let others: Vec<Point> = pointers
                        .read()
                        .iter()
                        .filter(|(other, _)| **other != id)
                        .map(|(_, p)| *p)
                        .collect();
                    travel.set(travel() + distance(last, point));
                    let next = match others.first() {
                        // Two fingers: zoom by how much they spread, around
                        // their middle, and pan with the middle.
                        Some(&other) => {
                            let before = distance(last, other);
                            let after = distance(point, other);
                            let o = origin();
                            let mid_before = midpoint(last, other);
                            let mid_after = midpoint(point, other);
                            let anchor = Point {
                                x: mid_after.x - o.x,
                                y: mid_after.y - o.y,
                            };
                            let factor = if before > 0.0 { after / before } else { 1.0 };
                            viewport()
                                .panned(mid_after.x - mid_before.x, mid_after.y - mid_before.y)
                                .zoomed_at(factor, anchor)
                        }
                        None => viewport().panned(point.x - last.x, point.y - last.y),
                    };
                    move_to(next);
                    pointers.write().insert(id, point);
                },
                onpointerup: move |evt| {
                    pointers.write().remove(&evt.pointer_id());
                },
                onpointercancel: move |evt| {
                    pointers.write().remove(&evt.pointer_id());
                },
                onpointerleave: move |evt| {
                    pointers.write().remove(&evt.pointer_id());
                },
                onwheel: move |evt| async move {
                    evt.prevent_default();
                    let delta = evt.delta();
                    let pixels = match delta {
                        WheelDelta::Pixels(v) => v.y,
                        WheelDelta::Lines(v) => v.y * 16.0,
                        WheelDelta::Pages(v) => v.y * 400.0,
                    };
                    let client = evt.client_coordinates();
                    measure().await;
                    let o = origin();
                    let anchor = Point {
                        x: client.x - o.x,
                        y: client.y - o.y,
                    };
                    move_to(viewport().zoomed_at((-pixels * WHEEL_ZOOM_RATE).exp(), anchor));
                },
                onkeydown: move |evt| {
                    let view = viewport();
                    let next = match evt.key() {
                        Key::ArrowLeft => view.panned(KEY_PAN_STEP, 0.0),
                        Key::ArrowRight => view.panned(-KEY_PAN_STEP, 0.0),
                        Key::ArrowUp => view.panned(0.0, KEY_PAN_STEP),
                        Key::ArrowDown => view.panned(0.0, -KEY_PAN_STEP),
                        Key::Escape => {
                            focus.set(None);
                            return;
                        }
                        Key::Character(c) if c == "+" || c == "=" => view.zoomed_at(ZOOM_STEP, middle()),
                        Key::Character(c) if c == "-" || c == "_" => view.zoomed_at(1.0 / ZOOM_STEP, middle()),
                        Key::Character(c) if c == "0" => {
                            evt.prevent_default();
                            fit();
                            return;
                        }
                        _ => return,
                    };
                    evt.prevent_default();
                    move_to(next);
                },
                g {
                    class: "family-tree-content",
                    transform: "translate({view.x} {view.y}) scale({view.scale})",
                    g { class: "family-tree-edges",
                        for edge in dimmed_edges {
                            EdgeLine {
                                key: "{edge_key(edge)}",
                                edge: edge.clone(),
                                highlight: if has_focus { Highlight::Dimmed } else { Highlight::Normal },
                            }
                        }
                        for edge in lineage_edges {
                            EdgeLine {
                                key: "{edge_key(edge)}",
                                edge: edge.clone(),
                                highlight: Highlight::Lineage,
                            }
                        }
                    }
                    g { class: "family-tree-nodes",
                        for node in layout.nodes() {
                            PersonBox {
                                key: "{node.person_id}",
                                node: node.clone(),
                                person: tree.get_person(&node.person_id).cloned(),
                                role: lineage.as_ref().map(|l| l.role(&node.person_id)),
                                on_pick: pick,
                            }
                        }
                    }
                }
            }
            p { class: "family-tree-status", "aria-live": "polite", "{status}" }
        }
    }
}

/// How an [`EdgeLine`] is drawn relative to the focus person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Highlight {
    /// No one is in focus.
    Normal,
    /// On the focus person's lineage.
    Lineage,
    /// Someone is in focus and this is not on their lineage.
    Dimmed,
}

/// One line between two boxes.
#[component]
fn EdgeLine(edge: Edge, highlight: Highlight) -> Element {
    let kind = match edge.kind {
        EdgeKind::ParentChild => "parent-child",
        EdgeKind::Spouse => "spouse",
    };
    let dash = (edge.kind == EdgeKind::Spouse).then_some("6 4");
    let (state, stroke, width, opacity) = match highlight {
        Highlight::Normal => ("", LINE_COLOUR, "2", None),
        Highlight::Lineage => (" on-lineage", HIGHLIGHT_COLOUR, "3.5", None),
        Highlight::Dimmed => (" dimmed", LINE_COLOUR, "2", Some(DIMMED_OPACITY)),
    };
    rsx! {
        path {
            class: "family-tree-edge {kind}{state}",
            "data-from": "{edge.from}",
            "data-to": "{edge.to}",
            d: edge_path(&edge),
            style: "fill: none; stroke: {stroke}; stroke-width: {width};",
            stroke_dasharray: dash,
            opacity,
        }
    }
}

/// One person's box with their name and years. `role` is how they relate
/// to the focus person, or `None` when no one is in focus.
#[component]
fn PersonBox(
    node: Node,
    person: Option<Person>,
    role: Option<LineageRole>,
    on_pick: EventHandler<String>,
) -> Element {
    let name = person
        .as_ref()
        .map_or(node.person_id.as_str(), |person| person.name.as_str());
    let years = person.as_ref().and_then(lifespan);
    let sex = person.as_ref().map_or(&Sex::Unknown, |person| &person.sex);
    let sex_class = match sex {
        Sex::Male => "male",
        Sex::Female => "female",
        Sex::Unknown => "unknown",
    };
    let fill = person_fill(sex);
    let (role_class, stroke, stroke_width, opacity) = match role {
        None => ("", LINE_COLOUR, "1.5", None),
        Some(LineageRole::Focus) => (" focus", HIGHLIGHT_COLOUR, "4", None),
        Some(LineageRole::Ancestor) => (" ancestor", HIGHLIGHT_COLOUR, "2.5", None),
        Some(LineageRole::Descendant) => (" descendant", HIGHLIGHT_COLOUR, "2.5", None),
        Some(LineageRole::Other) => (" dimmed", LINE_COLOUR, "1.5", Some(DIMMED_OPACITY)),
    };
    let max_chars = ((node.width - 2.0 * TEXT_PADDING) / (NAME_FONT_SIZE * CHAR_WIDTH_RATIO))
        .floor()
        .max(1.0) as usize;
    let label = fit_label(name, max_chars);
    let center_x = node.width / 2.0;
    // Two lines sit either side of the middle; one line sits on it.
    let (name_y, years_y) = if years.is_some() {
        (
            node.height / 2.0 - 4.0,
            node.height / 2.0 + YEARS_FONT_SIZE + 2.0,
        )
    } else {
        (node.height / 2.0 + NAME_FONT_SIZE / 3.0, 0.0)
    };
    let pressed = if role == Some(LineageRole::Focus) {
        "true"
    } else {
        "false"
    };
    let id = node.person_id.clone();
    let key_id = node.person_id.clone();

    rsx! {
        g {
            class: "family-tree-person {sex_class}{role_class}",
            "data-person-id": "{node.person_id}",
            transform: "translate({node.x} {node.y})",
            role: "button",
            tabindex: "0",
            "aria-pressed": pressed,
            "aria-label": "{name}",
            style: "cursor: pointer;",
            opacity,
            onclick: move |_| on_pick.call(id.clone()),
            onkeydown: move |evt| {
                if evt.key() == Key::Enter || evt.key() == Key::Character(" ".into()) {
                    evt.prevent_default();
                    evt.stop_propagation();
                    on_pick.call(key_id.clone());
                }
            },
            title { "{name}" }
            rect {
                width: "{node.width}",
                height: "{node.height}",
                rx: "8",
                style: "fill: {fill}; stroke: {stroke}; stroke-width: {stroke_width};",
            }
            text {
                class: "family-tree-name",
                x: "{center_x}",
                y: "{name_y}",
                text_anchor: "middle",
                font_size: "{NAME_FONT_SIZE}",
                font_weight: "600",
                style: "fill: {TEXT_COLOUR};",
                "{label}"
            }
            if let Some(years) = years {
                text {
                    class: "family-tree-years",
                    x: "{center_x}",
                    y: "{years_y}",
                    text_anchor: "middle",
                    font_size: "{YEARS_FONT_SIZE}",
                    style: "fill: {MUTED_TEXT_COLOUR};",
                    "{years}"
                }
            }
        }
    }
}

/// Where a pointer event happened on screen.
fn client_point(evt: &Event<PointerData>) -> Point {
    let client = evt.client_coordinates();
    Point {
        x: client.x,
        y: client.y,
    }
}

fn distance(a: Point, b: Point) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn midpoint(a: Point, b: Point) -> Point {
    Point {
        x: (a.x + b.x) / 2.0,
        y: (a.y + b.y) / 2.0,
    }
}

/// `1 ancestor`, `3 ancestors`, `no ancestors`.
fn count(n: usize, one: &str, many: &str) -> String {
    match n {
        0 => format!("no {many}"),
        1 => format!("1 {one}"),
        n => format!("{n} {many}"),
    }
}

/// A key that is unique per edge, for Dioxus list diffing.
fn edge_key(edge: &Edge) -> String {
    let kind = match edge.kind {
        EdgeKind::ParentChild => "parent",
        EdgeKind::Spouse => "spouse",
    };
    format!("{kind}:{}:{}", edge.from, edge.to)
}

/// SVG path data for an edge: a line through the corners of the route
/// `core` worked out, so siblings hang off their own family's bar.
fn edge_path(edge: &Edge) -> String {
    edge.route
        .iter()
        .enumerate()
        .map(|(i, point)| {
            let command = if i == 0 { "M" } else { "L" };
            format!("{command} {} {}", point.x, point.y)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Birth and death years for a box, such as `1920 – 1990`, `b. 1920` or
/// `d. 1990`, or `None` when neither date is known.
fn lifespan(person: &Person) -> Option<String> {
    let birth = person.birth_date.as_deref().map(year_of);
    let death = person.death_date.as_deref().map(year_of);
    match (birth, death) {
        (Some(birth), Some(death)) => Some(format!("{birth} – {death}")),
        (Some(birth), None) => Some(format!("b. {birth}")),
        (None, Some(death)) => Some(format!("d. {death}")),
        (None, None) => None,
    }
}

/// The first four-digit year in a free-form date such as `1920-06-15` or
/// `June 1920`, or the trimmed date itself when it holds no year.
fn year_of(date: &str) -> &str {
    let bytes = date.as_bytes();
    (0..bytes.len().saturating_sub(3))
        .find(|&i| {
            bytes[i..i + 4].iter().all(u8::is_ascii_digit)
                && (i == 0 || !bytes[i - 1].is_ascii_digit())
                && bytes.get(i + 4).is_none_or(|b| !b.is_ascii_digit())
        })
        .map_or(date.trim(), |i| &date[i..i + 4])
}

/// Shortens a name to at most `max_chars` characters, ending in `…` when cut.
fn fit_label(name: &str, max_chars: usize) -> String {
    if name.chars().count() <= max_chars {
        return name.to_string();
    }
    let kept: String = name.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cahokia_core::sample::sample_family_tree;

    fn person(id: &str, name: &str, sex: Sex, birth: Option<&str>, death: Option<&str>) -> Person {
        Person::new(
            id.to_string(),
            name.to_string(),
            sex,
            birth.map(str::to_string),
            death.map(str::to_string),
        )
    }

    /// Two parents and two children.
    fn family() -> FamilyTree {
        let mut tree = FamilyTree::new();
        for p in [
            person(
                "dad",
                "John Doe",
                Sex::Male,
                Some("1920-06-15"),
                Some("1990"),
            ),
            person("mum", "Mary Doe", Sex::Female, Some("June 1922"), None),
            person("kid1", "Jane Doe", Sex::Female, Some("1950"), None),
            person("kid2", "Jim Doe", Sex::Unknown, None, None),
        ] {
            tree.add_person(p).unwrap();
        }
        tree.add_spouse_relationship("dad".into(), "mum".into())
            .unwrap();
        for kid in ["kid1", "kid2"] {
            for parent in ["dad", "mum"] {
                tree.add_parent_child_relationship(parent.into(), kid.into())
                    .unwrap();
            }
        }
        tree
    }

    fn render(tree: FamilyTree) -> String {
        render_with(tree, Spacing::default())
    }

    fn render_with(tree: FamilyTree, spacing: Spacing) -> String {
        dioxus_ssr::render_element(rsx! { FamilyTreeView { tree, spacing } })
    }

    #[test]
    fn draws_a_box_per_person_with_name_and_years() {
        let html = render(family());
        assert_eq!(html.matches("class=\"family-tree-person").count(), 4);
        for name in ["John Doe", "Mary Doe", "Jane Doe", "Jim Doe"] {
            assert!(html.contains(&format!(">{name}</text>")), "{name} missing");
        }
        assert!(html.contains(">1920 – 1990</text>"));
        assert!(html.contains(">b. 1922</text>"));
        assert!(html.contains(">b. 1950</text>"));
    }

    #[test]
    fn person_without_dates_has_no_years_line() {
        let mut tree = FamilyTree::new();
        tree.add_person(person("p", "Jim Doe", Sex::Unknown, None, None))
            .unwrap();
        let html = render(tree);
        assert!(!html.contains("family-tree-years"));
    }

    #[test]
    fn draws_a_line_per_relationship() {
        let html = render(family());
        assert_eq!(html.matches("family-tree-edge spouse").count(), 1);
        assert_eq!(html.matches("family-tree-edge parent-child").count(), 4);
        assert!(html.contains("data-from=\"dad\" data-to=\"kid1\""));
    }

    #[test]
    fn places_boxes_at_layout_coordinates() {
        let tree = family();
        let layout = tree.tree_layout();
        let html = render(tree);
        for node in layout.nodes() {
            let expected = format!(
                "data-person-id=\"{}\" transform=\"translate({} {})\"",
                node.person_id, node.x, node.y
            );
            assert!(html.contains(&expected), "{expected} missing from {html}");
        }
    }

    #[test]
    fn fills_the_width_and_starts_at_natural_size() {
        let tree = family();
        let layout = tree.tree_layout();
        let html = render(tree);
        let height = (layout.height() + 32.0).clamp(MIN_VIEW_HEIGHT, MAX_VIEW_HEIGHT);
        assert!(html.contains(&format!("width=\"100%\" height=\"{height}\"")));
        assert!(html.contains("transform=\"translate(16 16) scale(1)\""));
        assert!(html.contains("touch-action: none"));
        assert!(html.contains(">100%</span>"));
    }

    #[test]
    fn has_zoom_controls_and_a_focus_list() {
        let html = render(family());
        for class in [
            "family-tree-zoom-in",
            "family-tree-zoom-out",
            "family-tree-fit",
        ] {
            assert!(
                html.contains(&format!("class=\"{class}\"")),
                "{class} missing"
            );
        }
        assert!(html.contains("class=\"family-tree-focus\""));
        // Everyone is in the focus list, sorted by name, after "No one".
        let options: Vec<&str> = html
            .split("<option")
            .skip(1)
            .filter_map(|o| o.split('>').nth(1))
            .filter_map(|o| o.strip_suffix("</option"))
            .collect();
        assert_eq!(
            options,
            ["No one", "Jane Doe", "Jim Doe", "John Doe", "Mary Doe"]
        );
    }

    #[test]
    fn people_are_buttons_for_keyboard_users() {
        let html = render(family());
        assert_eq!(html.matches("role=\"button\"").count(), 4);
        assert!(html.contains("aria-pressed=\"false\""));
        assert!(!html.contains("aria-pressed=\"true\""));
    }

    #[test]
    fn without_a_focus_nothing_is_highlighted_or_dimmed() {
        let html = render(family());
        for class in ["focus", "ancestor", "descendant", "dimmed", "on-lineage"] {
            assert!(!html.contains(&format!(" {class}\"")), "{class} found");
        }
        assert!(html.contains("Click or tap a person to highlight"));
    }

    fn render_focused(tree: FamilyTree, focus: &str) -> String {
        let initial_focus = Some(focus.to_string());
        // Undo the HTML escaping of apostrophes so tests read naturally.
        dioxus_ssr::render_element(rsx! { FamilyTreeView { tree, initial_focus } })
            .replace("&#39;", "'")
    }

    /// The `class` attribute of a person's box.
    fn person_class<'a>(html: &'a str, id: &str) -> &'a str {
        let at = html
            .find(&format!("data-person-id=\"{id}\""))
            .unwrap_or_else(|| panic!("{id} missing"));
        let start = html[..at].rfind("class=\"").map(|i| i + 7).unwrap_or(0);
        let end = start + html[start..].find('"').unwrap_or(0);
        &html[start..end]
    }

    #[test]
    fn focus_person_highlights_ancestors_and_descendants_and_dims_the_rest() {
        let html = render_focused(sample_family_tree(), "henry");
        assert!(person_class(&html, "henry").ends_with(" focus"));
        assert!(person_class(&html, "thomas").ends_with(" ancestor"));
        assert!(person_class(&html, "clara").ends_with(" ancestor"));
        assert!(person_class(&html, "daniel").ends_with(" descendant"));
        assert!(person_class(&html, "margaret").ends_with(" descendant"));
        for id in ["alice", "ruth", "samuel", "eleanor", "sam"] {
            assert!(person_class(&html, id).ends_with(" dimmed"), "{id}");
        }
        assert_eq!(html.matches("aria-pressed=\"true\"").count(), 1);
        assert_eq!(html.matches("on-lineage").count(), 4);
        assert!(html.contains("Highlighting Henry Reed's 2 ancestors and 2 descendants."));
    }

    #[test]
    fn lineage_lines_are_drawn_on_top() {
        let html = render_focused(sample_family_tree(), "henry");
        let last_dimmed = html.rfind("dimmed\" data-from").unwrap_or(0);
        let first_lineage = html.find("on-lineage").unwrap_or(0);
        assert!(first_lineage > last_dimmed);
    }

    #[test]
    fn focus_list_selects_the_focus_person() {
        let html = render_focused(sample_family_tree(), "ruth");
        assert!(html.contains("value=\"ruth\" selected"), "{html}");
        assert!(!html.contains("value=\"\" selected"));
        assert!(html.contains("Highlighting Ruth Price's 2 ancestors and 2 descendants."));
    }

    #[test]
    fn an_unknown_focus_person_is_ignored() {
        let html = render_focused(family(), "nobody");
        assert!(!html.contains(" dimmed\""));
        assert!(html.contains("Click or tap a person to highlight"));
    }

    #[test]
    fn count_words() {
        assert_eq!(count(0, "ancestor", "ancestors"), "no ancestors");
        assert_eq!(count(1, "ancestor", "ancestors"), "1 ancestor");
        assert_eq!(count(3, "ancestor", "ancestors"), "3 ancestors");
    }

    #[test]
    fn pinch_helpers() {
        let a = Point { x: 0.0, y: 0.0 };
        let b = Point { x: 3.0, y: 4.0 };
        assert_eq!(distance(a, b), 5.0);
        assert_eq!(midpoint(a, b), Point { x: 1.5, y: 2.0 });
    }

    #[test]
    fn uses_custom_spacing() {
        let spacing = Spacing {
            node_width: 100.0,
            node_height: 50.0,
            column_gap: 10.0,
            row_gap: 20.0,
        };
        let html = render_with(family(), spacing);
        assert!(html.contains("width=\"100\" height=\"50\""));
    }

    #[test]
    fn empty_tree_shows_a_message_instead_of_a_drawing() {
        let html = render(FamilyTree::new());
        assert!(html.contains("No people in this family tree yet."));
        assert!(!html.contains("<svg"));
    }

    #[test]
    fn escapes_names() {
        let mut tree = FamilyTree::new();
        tree.add_person(person("p", "<b>Bob</b>", Sex::Male, None, None))
            .unwrap();
        let html = render(tree);
        assert!(!html.contains("<b>"));
    }

    #[test]
    fn long_names_are_shortened_with_full_name_as_tooltip() {
        let mut tree = FamilyTree::new();
        let name = "Bartholomew Montgomery-Fitzwilliam";
        tree.add_person(person("p", name, Sex::Male, None, None))
            .unwrap();
        let html = render(tree);
        assert!(html.contains(&format!("<title>{name}</title>")));
        assert!(html.contains("…</text>"));
    }

    #[test]
    fn spouse_path_is_straight() {
        let tree = family();
        let layout = tree.tree_layout();
        let edge = layout
            .edges()
            .iter()
            .find(|edge| edge.kind == EdgeKind::Spouse)
            .unwrap();
        assert_eq!(
            edge_path(edge),
            format!(
                "M {} {} L {} {}",
                edge.start.x, edge.start.y, edge.end.x, edge.end.y
            )
        );
    }

    #[test]
    fn parent_child_path_follows_the_route() {
        let edge = Edge {
            kind: EdgeKind::ParentChild,
            from: "a".into(),
            to: "b".into(),
            start: Point { x: 200.0, y: 30.0 },
            end: Point { x: 90.0, y: 140.0 },
            route: vec![
                Point { x: 200.0, y: 30.0 },
                Point { x: 200.0, y: 100.0 },
                Point { x: 90.0, y: 100.0 },
                Point { x: 90.0, y: 140.0 },
            ],
        };
        assert_eq!(edge_path(&edge), "M 200 30 L 200 100 L 90 100 L 90 140");
    }

    #[test]
    fn couple_children_hang_from_the_spouse_line() {
        let html = render(family());
        // dad and mum sit side by side, so the line to each child drops
        // from the middle of their spouse line (x 200, y 30).
        assert!(html.contains("data-from=\"dad\" data-to=\"kid1\" d=\"M 200 30 L 200 100"));
    }

    #[test]
    fn year_of_finds_the_year_in_common_formats() {
        assert_eq!(year_of("1920-06-15"), "1920");
        assert_eq!(year_of("June 1920"), "1920");
        assert_eq!(year_of("15/06/1920"), "1920");
        assert_eq!(year_of("1920"), "1920");
        assert_eq!(year_of("abt. 1850s"), "1850");
        assert_eq!(year_of(" unknown "), "unknown");
        assert_eq!(year_of("19201"), "19201");
        assert_eq!(year_of(""), "");
    }

    #[test]
    fn lifespan_labels() {
        let both = person("p", "P", Sex::Unknown, Some("1920"), Some("1990-01-01"));
        let birth = person("p", "P", Sex::Unknown, Some("1920"), None);
        let death = person("p", "P", Sex::Unknown, None, Some("1990"));
        let neither = person("p", "P", Sex::Unknown, None, None);
        assert_eq!(lifespan(&both).as_deref(), Some("1920 – 1990"));
        assert_eq!(lifespan(&birth).as_deref(), Some("b. 1920"));
        assert_eq!(lifespan(&death).as_deref(), Some("d. 1990"));
        assert_eq!(lifespan(&neither), None);
    }

    #[test]
    fn fit_label_keeps_short_names_and_cuts_long_ones() {
        assert_eq!(fit_label("Jane", 10), "Jane");
        assert_eq!(fit_label("Jane Doe Smith", 9), "Jane Doe…");
        assert_eq!(fit_label("Zoë Åsa Ørsted", 5), "Zoë…");
        assert_eq!(fit_label("Anything", 1), "…");
    }
}
