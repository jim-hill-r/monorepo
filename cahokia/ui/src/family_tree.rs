//! An SVG drawing of a [`FamilyTree`].
//!
//! [`FamilyTreeView`] draws the boxes and lines that
//! [`FamilyTree::tree_layout_with`] computes, so web, desktop and mobile show
//! the same tree. All layout maths stays in `core`; this module only turns
//! coordinates into SVG elements and people into labels.

use cahokia_core::{Edge, EdgeKind, FamilyTree, Node, Person, Sex, Spacing};
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

const BACKGROUND: &str = "#3d2f1f";
const LINE_COLOUR: &str = "#d4a574";
const TEXT_COLOUR: &str = "#f5e6d3";
const MUTED_TEXT_COLOUR: &str = "#d9c4a8";

/// Draws a family tree as inline SVG: a box per person with their name and
/// birth and death years, and lines between parents and children and
/// between spouses.
///
/// The drawing is shown at its natural size. Put it in a scrolling container
/// when the tree may be wider than the screen.
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
) -> Element {
    let layout = tree.tree_layout_with(spacing);
    if layout.nodes().is_empty() {
        return rsx! {
            p { class: "family-tree-empty", "No people in this family tree yet." }
        };
    }

    let width = layout.width() + 2.0 * MARGIN;
    let height = layout.height() + 2.0 * MARGIN;
    let view_box = format!("{} {} {width} {height}", -MARGIN, -MARGIN);

    rsx! {
        svg {
            class: "family-tree",
            xmlns: "http://www.w3.org/2000/svg",
            role: "img",
            "aria-label": "Family tree",
            width: "{width}",
            height: "{height}",
            view_box,
            style: "background-color: {BACKGROUND}; font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;",
            g { class: "family-tree-edges",
                for edge in layout.edges() {
                    EdgeLine { key: "{edge_key(edge)}", edge: edge.clone() }
                }
            }
            g { class: "family-tree-nodes",
                for node in layout.nodes() {
                    PersonBox {
                        key: "{node.person_id}",
                        node: node.clone(),
                        person: tree.get_person(&node.person_id).cloned(),
                    }
                }
            }
        }
    }
}

/// One line between two boxes.
#[component]
fn EdgeLine(edge: Edge) -> Element {
    let (class, dash) = match edge.kind {
        EdgeKind::ParentChild => ("family-tree-edge parent-child", None),
        EdgeKind::Spouse => ("family-tree-edge spouse", Some("6 4")),
    };
    rsx! {
        path {
            class,
            "data-from": "{edge.from}",
            "data-to": "{edge.to}",
            d: edge_path(&edge),
            fill: "none",
            stroke: LINE_COLOUR,
            stroke_width: "2",
            stroke_dasharray: dash,
        }
    }
}

/// One person's box with their name and years.
#[component]
fn PersonBox(node: Node, person: Option<Person>) -> Element {
    let name = person
        .as_ref()
        .map_or(node.person_id.as_str(), |person| person.name.as_str());
    let years = person.as_ref().and_then(lifespan);
    let sex = person.as_ref().map_or(&Sex::Unknown, |person| &person.sex);
    let (sex_class, fill) = match sex {
        Sex::Male => ("male", "#4f4a35"),
        Sex::Female => ("female", "#5a3d33"),
        Sex::Unknown => ("unknown", "#4d3b28"),
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

    rsx! {
        g {
            class: "family-tree-person {sex_class}",
            "data-person-id": "{node.person_id}",
            transform: "translate({node.x} {node.y})",
            title { "{name}" }
            rect {
                width: "{node.width}",
                height: "{node.height}",
                rx: "8",
                fill,
                stroke: LINE_COLOUR,
                stroke_width: "1.5",
            }
            text {
                class: "family-tree-name",
                x: "{center_x}",
                y: "{name_y}",
                text_anchor: "middle",
                font_size: "{NAME_FONT_SIZE}",
                font_weight: "600",
                fill: TEXT_COLOUR,
                "{label}"
            }
            if let Some(years) = years {
                text {
                    class: "family-tree-years",
                    x: "{center_x}",
                    y: "{years_y}",
                    text_anchor: "middle",
                    font_size: "{YEARS_FONT_SIZE}",
                    fill: MUTED_TEXT_COLOUR,
                    "{years}"
                }
            }
        }
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
    use cahokia_core::Point;

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
    fn sizes_the_drawing_to_the_layout_plus_margin() {
        let tree = family();
        let layout = tree.tree_layout();
        let html = render(tree);
        let (width, height) = (layout.width() + 32.0, layout.height() + 32.0);
        assert!(html.contains(&format!("width=\"{width}\" height=\"{height}\"")));
        assert!(html.contains(&format!("viewBox=\"-16 -16 {width} {height}\"")));
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
