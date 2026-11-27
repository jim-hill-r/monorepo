//! Pan, zoom and focus maths for an interactive family tree view.
//!
//! [`Viewport`] maps [`TreeLayout`](crate::TreeLayout) coordinates to screen
//! pixels, and [`Lineage`] says which people and lines to highlight around a
//! focus person. Renderers keep one of each in their state and only turn
//! them into transforms and colours, so every platform pans, zooms and
//! highlights the same way.
//!
//! The rules:
//!
//! - A screen point is `layout point × scale + offset`, as an SVG
//!   `translate(x y) scale(s)` transform draws it.
//! - The scale stays between [`Viewport::MIN_SCALE`] and
//!   [`Viewport::MAX_SCALE`]; zooming past either end stops at it.
//! - Zooming keeps the layout point under the anchor (the cursor, the middle
//!   of a pinch, or the middle of the view) where it is, so the tree grows or
//!   shrinks around it.
//! - Fitting shows the whole tree centred in the view, never larger than its
//!   natural size, with a margin on every side.
//! - A focus person's lineage is their ancestors and descendants. Spouses,
//!   siblings, cousins and in-laws are not part of it unless they are also
//!   an ancestor or descendant.
//! - A parent-child line is on the lineage when it joins two people on the
//!   same side of the focus person: both the focus person or ancestors, or
//!   both the focus person or descendants. Spouse lines never are.

use std::collections::HashSet;

use crate::{Edge, EdgeKind, FamilyTree, Point};

/// Where a tree is drawn on screen: how far it is zoomed and how far it is
/// moved.
///
/// # Example
/// ```
/// # use core::{Point, Viewport};
/// // A 1000 × 400 tree in a 500 × 400 view is shrunk to half size.
/// let view = Viewport::fit(1000.0, 400.0, 500.0, 400.0, 0.0);
/// assert_eq!(view.scale, 0.5);
///
/// // Zooming in around the middle of the view keeps it in place.
/// let middle = Point { x: 250.0, y: 200.0 };
/// let zoomed = view.zoomed_at(2.0, middle);
/// assert_eq!(zoomed.scale, 1.0);
/// assert_eq!(zoomed.to_layout(middle), view.to_layout(middle));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Screen pixels per layout unit.
    pub scale: f64,
    /// Screen position of the layout origin, from the left of the view.
    pub x: f64,
    /// Screen position of the layout origin, from the top of the view.
    pub y: f64,
}

impl Default for Viewport {
    /// Natural size, with the layout origin at the top left of the view.
    fn default() -> Self {
        Viewport {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Viewport {
    /// The smallest scale: a fifth of natural size, enough to see a few
    /// hundred people at once.
    pub const MIN_SCALE: f64 = 0.2;
    /// The largest scale: four times natural size.
    pub const MAX_SCALE: f64 = 4.0;

    /// Natural size with the layout origin at `(x, y)` on screen.
    pub fn at(x: f64, y: f64) -> Self {
        Viewport { scale: 1.0, x, y }
    }

    /// The viewport that shows a `content_width × content_height` layout
    /// whole and centred in a `view_width × view_height` view, leaving
    /// `margin` pixels on every side. It never zooms in past natural size,
    /// and a view with no room left falls back to [`Viewport::default`].
    pub fn fit(
        content_width: f64,
        content_height: f64,
        view_width: f64,
        view_height: f64,
        margin: f64,
    ) -> Self {
        let room_x = view_width - 2.0 * margin;
        let room_y = view_height - 2.0 * margin;
        if room_x <= 0.0 || room_y <= 0.0 {
            return Viewport::default();
        }
        let mut scale = 1.0_f64;
        if content_width > 0.0 {
            scale = scale.min(room_x / content_width);
        }
        if content_height > 0.0 {
            scale = scale.min(room_y / content_height);
        }
        let scale = clamp_scale(scale);
        Viewport {
            scale,
            x: (view_width - content_width * scale) / 2.0,
            y: (view_height - content_height * scale) / 2.0,
        }
    }

    /// The viewport moved by `dx`, `dy` screen pixels, as when dragging.
    pub fn panned(self, dx: f64, dy: f64) -> Self {
        Viewport {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }

    /// The viewport zoomed by `factor` (above 1 zooms in) around the screen
    /// point `anchor`, which stays over the same part of the tree. The
    /// scale stops at [`Viewport::MIN_SCALE`] and [`Viewport::MAX_SCALE`];
    /// a factor that is not a positive number leaves the viewport as it is.
    pub fn zoomed_at(self, factor: f64, anchor: Point) -> Self {
        if !(factor.is_finite() && factor > 0.0) {
            return self;
        }
        let scale = clamp_scale(self.scale * factor);
        let under = self.to_layout(anchor);
        Viewport {
            scale,
            x: anchor.x - under.x * scale,
            y: anchor.y - under.y * scale,
        }
    }

    /// The viewport moved, at the same scale, so the layout point `target`
    /// sits in the middle of a `view_width × view_height` view.
    pub fn centred_on(self, target: Point, view_width: f64, view_height: f64) -> Self {
        Viewport {
            x: view_width / 2.0 - target.x * self.scale,
            y: view_height / 2.0 - target.y * self.scale,
            ..self
        }
    }

    /// The layout point drawn at the screen point `screen`.
    pub fn to_layout(self, screen: Point) -> Point {
        Point {
            x: (screen.x - self.x) / self.scale,
            y: (screen.y - self.y) / self.scale,
        }
    }

    /// The screen point the layout point `layout` is drawn at.
    pub fn to_screen(self, layout: Point) -> Point {
        Point {
            x: layout.x * self.scale + self.x,
            y: layout.y * self.scale + self.y,
        }
    }
}

fn clamp_scale(scale: f64) -> f64 {
    scale.clamp(Viewport::MIN_SCALE, Viewport::MAX_SCALE)
}

/// How a person relates to the focus person of a [`Lineage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineageRole {
    /// The focus person themselves.
    Focus,
    /// A parent, grandparent and so on of the focus person.
    Ancestor,
    /// A child, grandchild and so on of the focus person.
    Descendant,
    /// Anyone else.
    Other,
}

/// A focus person with their ancestors and descendants, for highlighting.
///
/// # Example
/// ```
/// # use core::{LineageRole, sample::sample_family_tree};
/// let tree = sample_family_tree();
/// let lineage = tree.lineage("henry").unwrap();
/// assert_eq!(lineage.role("henry"), LineageRole::Focus);
/// assert_eq!(lineage.role("thomas"), LineageRole::Ancestor);
/// assert_eq!(lineage.role("daniel"), LineageRole::Descendant);
/// // A sister is related, but not an ancestor or descendant.
/// assert_eq!(lineage.role("ruth"), LineageRole::Other);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    focus: String,
    ancestors: HashSet<String>,
    descendants: HashSet<String>,
}

impl Lineage {
    /// Id of the focus person.
    pub fn focus(&self) -> &str {
        &self.focus
    }

    /// How many ancestors the focus person has in the tree.
    pub fn ancestor_count(&self) -> usize {
        self.ancestors.len()
    }

    /// How many descendants the focus person has in the tree.
    pub fn descendant_count(&self) -> usize {
        self.descendants.len()
    }

    /// How `person_id` relates to the focus person. Someone who is both an
    /// ancestor and a descendant (bad data with a loop) counts as an
    /// ancestor.
    pub fn role(&self, person_id: &str) -> LineageRole {
        if person_id == self.focus {
            LineageRole::Focus
        } else if self.ancestors.contains(person_id) {
            LineageRole::Ancestor
        } else if self.descendants.contains(person_id) {
            LineageRole::Descendant
        } else {
            LineageRole::Other
        }
    }

    /// Whether `edge` is a line from the focus person up to an ancestor or
    /// down to a descendant.
    pub fn contains_edge(&self, edge: &Edge) -> bool {
        if edge.kind != EdgeKind::ParentChild {
            return false;
        }
        let up = |id: &str| id == self.focus || self.ancestors.contains(id);
        let down = |id: &str| id == self.focus || self.descendants.contains(id);
        (up(&edge.from) && up(&edge.to)) || (down(&edge.from) && down(&edge.to))
    }
}

impl FamilyTree {
    /// The lineage of `person_id`: their ancestors and descendants, or
    /// `None` when no such person is in the tree.
    pub fn lineage(&self, person_id: &str) -> Option<Lineage> {
        self.get_person(person_id)?;
        let ids = |people: Vec<&crate::Person>| people.into_iter().map(|p| p.id.clone()).collect();
        Some(Lineage {
            focus: person_id.to_string(),
            ancestors: ids(self.get_ancestors(person_id)),
            descendants: ids(self.get_descendants(person_id)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sample::sample_family_tree;

    fn point(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    fn assert_close(a: Point, b: Point) {
        assert!(
            (a.x - b.x).abs() < 1e-9 && (a.y - b.y).abs() < 1e-9,
            "{a:?} != {b:?}"
        );
    }

    #[test]
    fn default_is_natural_size_at_the_origin() {
        let view = Viewport::default();
        assert_eq!(view.to_screen(point(10.0, 20.0)), point(10.0, 20.0));
        assert_eq!(
            Viewport::at(5.0, 6.0).to_screen(point(0.0, 0.0)),
            point(5.0, 6.0)
        );
    }

    #[test]
    fn screen_and_layout_points_round_trip() {
        let view = Viewport {
            scale: 1.5,
            x: 30.0,
            y: -12.0,
        };
        let p = point(123.0, 456.0);
        assert_close(view.to_layout(view.to_screen(p)), p);
    }

    #[test]
    fn panning_moves_the_tree_by_the_drag_distance() {
        let view = Viewport::at(10.0, 10.0).panned(25.0, -5.0);
        assert_eq!(view, Viewport::at(35.0, 5.0));
    }

    #[test]
    fn zoom_keeps_the_anchor_over_the_same_layout_point() {
        let view = Viewport::at(40.0, 20.0);
        let anchor = point(300.0, 150.0);
        let before = view.to_layout(anchor);
        let zoomed = view.zoomed_at(1.25, anchor);
        assert_eq!(zoomed.scale, 1.25);
        assert_close(zoomed.to_layout(anchor), before);
    }

    #[test]
    fn zoom_stops_at_the_limits_and_still_keeps_the_anchor() {
        let anchor = point(100.0, 100.0);
        let big = Viewport::default().zoomed_at(100.0, anchor);
        assert_eq!(big.scale, Viewport::MAX_SCALE);
        assert_close(big.to_layout(anchor), anchor);
        let small = Viewport::default().zoomed_at(0.001, anchor);
        assert_eq!(small.scale, Viewport::MIN_SCALE);
    }

    #[test]
    fn zoom_ignores_nonsense_factors() {
        let view = Viewport::at(1.0, 2.0);
        for factor in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(view.zoomed_at(factor, point(5.0, 5.0)), view);
        }
    }

    #[test]
    fn fit_shrinks_a_big_tree_and_centres_it() {
        let view = Viewport::fit(1000.0, 200.0, 520.0, 300.0, 10.0);
        assert_eq!(view.scale, 0.5);
        // 500 × 100 drawn in a 520 × 300 view.
        assert_eq!((view.x, view.y), (10.0, 100.0));
    }

    #[test]
    fn fit_never_enlarges_a_small_tree() {
        let view = Viewport::fit(100.0, 50.0, 500.0, 300.0, 10.0);
        assert_eq!(view.scale, 1.0);
        assert_eq!((view.x, view.y), (200.0, 125.0));
    }

    #[test]
    fn fit_stops_at_the_smallest_scale() {
        let view = Viewport::fit(100_000.0, 50.0, 500.0, 300.0, 0.0);
        assert_eq!(view.scale, Viewport::MIN_SCALE);
    }

    #[test]
    fn fit_in_a_view_with_no_room_falls_back_to_default() {
        assert_eq!(
            Viewport::fit(100.0, 100.0, 10.0, 10.0, 20.0),
            Viewport::default()
        );
        assert_eq!(
            Viewport::fit(100.0, 100.0, 0.0, 0.0, 0.0),
            Viewport::default()
        );
    }

    #[test]
    fn centred_on_puts_the_target_in_the_middle() {
        let view = Viewport {
            scale: 2.0,
            x: 0.0,
            y: 0.0,
        }
        .centred_on(point(50.0, 25.0), 400.0, 200.0);
        assert_eq!(view.to_screen(point(50.0, 25.0)), point(200.0, 100.0));
        assert_eq!(view.scale, 2.0);
    }

    #[test]
    fn lineage_of_a_grandparent_is_every_descendant() {
        let tree = sample_family_tree();
        let lineage = tree.lineage("thomas").unwrap();
        assert_eq!(lineage.focus(), "thomas");
        assert_eq!(lineage.ancestor_count(), 0);
        assert_eq!(lineage.descendant_count(), 6);
        assert_eq!(lineage.role("clara"), LineageRole::Other);
        assert_eq!(lineage.role("sam"), LineageRole::Descendant);
        assert_eq!(lineage.role("alice"), LineageRole::Other);
    }

    #[test]
    fn lineage_of_a_grandchild_is_their_ancestors() {
        let tree = sample_family_tree();
        let lineage = tree.lineage("eleanor").unwrap();
        assert_eq!(lineage.ancestor_count(), 4);
        for id in ["samuel", "ruth", "thomas", "clara"] {
            assert_eq!(lineage.role(id), LineageRole::Ancestor, "{id}");
        }
        for id in ["sam", "henry", "daniel"] {
            assert_eq!(lineage.role(id), LineageRole::Other, "{id}");
        }
    }

    #[test]
    fn lineage_of_someone_missing_is_none() {
        assert_eq!(sample_family_tree().lineage("nobody"), None);
    }

    #[test]
    fn only_lines_along_the_lineage_are_highlighted() {
        let tree = sample_family_tree();
        let layout = tree.tree_layout();
        let lineage = tree.lineage("henry").unwrap();
        let mut on: Vec<(&str, &str)> = layout
            .edges()
            .iter()
            .filter(|edge| lineage.contains_edge(edge))
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        on.sort_unstable();
        assert_eq!(
            on,
            vec![
                ("clara", "henry"),
                ("henry", "daniel"),
                ("henry", "margaret"),
                ("thomas", "henry"),
            ]
        );
    }

    #[test]
    fn lines_between_ancestors_are_on_the_lineage() {
        let tree = sample_family_tree();
        let lineage = tree.lineage("daniel").unwrap();
        let edge = tree
            .tree_layout()
            .edges()
            .iter()
            .find(|edge| edge.from == "thomas" && edge.to == "henry")
            .cloned()
            .unwrap();
        assert!(lineage.contains_edge(&edge));
        // But not thomas to his other child, daniel's aunt.
        let aunt = tree
            .tree_layout()
            .edges()
            .iter()
            .find(|edge| edge.from == "thomas" && edge.to == "ruth")
            .cloned()
            .unwrap();
        assert!(!lineage.contains_edge(&aunt));
    }

    #[test]
    fn spouse_lines_are_never_on_the_lineage() {
        let tree = sample_family_tree();
        let lineage = tree.lineage("daniel").unwrap();
        for edge in tree.tree_layout().edges() {
            if edge.kind == EdgeKind::Spouse {
                assert!(!lineage.contains_edge(edge), "{edge:?}");
            }
        }
    }
}
