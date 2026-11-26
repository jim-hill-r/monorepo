//! Coordinates for drawing a [`FamilyTree`].
//!
//! [`FamilyTree::tree_layout`] turns the rows of
//! [`FamilyTree::generation_layout`] into boxes with coordinates, plus the
//! lines to draw between them, so a renderer only draws and never walks
//! relationships itself.
//!
//! The rules:
//!
//! - Coordinates are in abstract units with the origin at the top left, `x`
//!   growing to the right and `y` growing down, as in SVG. [`Spacing`] sets
//!   the box size and the gaps between boxes.
//! - Every person gets a box of the same size. Rows sit one box height plus
//!   one row gap apart, and each row is centred on the widest row.
//! - There is one [`Edge`] per parent-child pair and one per spouse pair,
//!   however many directions the relationship was recorded in. Sibling
//!   links are not drawn because siblings already share their parents' lines.
//! - Parent-child edges run from the bottom middle of the parent's box to
//!   the top middle of the child's box. Spouse edges run between the facing
//!   sides of the two boxes, left spouse first. When bad data puts two linked
//!   people in unexpected rows, the edge still joins the facing sides.
//! - Nodes are listed row by row, left to right, and edges spouses first,
//!   then parent-child, each sorted by id, so the result is the same on every
//!   run.

use std::collections::HashMap;

use crate::{FamilyTree, Placement};

/// Box size and gaps used to turn generation rows into coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacing {
    /// Width of every person's box.
    pub node_width: f64,
    /// Height of every person's box.
    pub node_height: f64,
    /// Horizontal gap between neighbouring boxes in a row.
    pub column_gap: f64,
    /// Vertical gap between rows.
    pub row_gap: f64,
}

impl Default for Spacing {
    /// Room for a name of about 19 characters above birth and death years,
    /// as `ui`'s `FamilyTreeView` draws them. Longer names are shortened.
    fn default() -> Self {
        Spacing {
            node_width: 180.0,
            node_height: 60.0,
            column_gap: 40.0,
            row_gap: 80.0,
        }
    }
}

/// A point in layout coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Distance from the left edge.
    pub x: f64,
    /// Distance from the top edge.
    pub y: f64,
}

/// One person's box in a [`TreeLayout`].
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Id of the person in the box.
    pub person_id: String,
    /// Row and position the box was placed at.
    pub placement: Placement,
    /// Left edge of the box.
    pub x: f64,
    /// Top edge of the box.
    pub y: f64,
    /// Width of the box.
    pub width: f64,
    /// Height of the box.
    pub height: f64,
}

impl Node {
    /// Middle of the box.
    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    fn top_middle(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y,
        }
    }

    fn bottom_middle(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height,
        }
    }

    fn left_middle(&self) -> Point {
        Point {
            x: self.x,
            y: self.y + self.height / 2.0,
        }
    }

    fn right_middle(&self) -> Point {
        Point {
            x: self.x + self.width,
            y: self.y + self.height / 2.0,
        }
    }
}

/// What an [`Edge`] stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeKind {
    /// `from` is a parent of `to`.
    ParentChild,
    /// `from` and `to` are spouses; `from` is the one further left.
    Spouse,
}

/// A line to draw between two people's boxes.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// What the line stands for.
    pub kind: EdgeKind,
    /// Id of the parent, or of the left spouse.
    pub from: String,
    /// Id of the child, or of the right spouse.
    pub to: String,
    /// Where the line leaves the `from` box.
    pub start: Point,
    /// Where the line reaches the `to` box.
    pub end: Point,
}

/// Everything a renderer needs to draw a [`FamilyTree`]: a box per person
/// and a line per relationship.
///
/// # Example
/// ```
/// # use core::{EdgeKind, FamilyTree, Person, Point, Sex};
/// #
/// let mut tree = FamilyTree::new();
/// for id in ["dad", "kid"] {
///     tree.add_person(Person::new(id.to_string(), id.to_string(), Sex::Unknown, None, None))
///         .unwrap();
/// }
/// tree.add_parent_child_relationship("dad".to_string(), "kid".to_string()).unwrap();
///
/// let layout = tree.tree_layout();
/// let kid = layout.node("kid").unwrap();
/// assert_eq!((kid.x, kid.y), (0.0, 140.0));
///
/// let edge = &layout.edges()[0];
/// assert_eq!(edge.kind, EdgeKind::ParentChild);
/// assert_eq!((edge.from.as_str(), edge.to.as_str()), ("dad", "kid"));
/// assert_eq!(edge.start, Point { x: 90.0, y: 60.0 });
/// assert_eq!(edge.end, Point { x: 90.0, y: 140.0 });
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeLayout {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    index: HashMap<String, usize>,
    width: f64,
    height: f64,
}

impl TreeLayout {
    /// Every person's box, row by row from the top, each row left to right.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Every line to draw: spouse links first, then parent-child links.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// The box of the person with this id, or `None` if they are not in the tree.
    pub fn node(&self, person_id: &str) -> Option<&Node> {
        self.index.get(person_id).map(|&i| &self.nodes[i])
    }

    /// Width of the area the boxes cover, starting at `x = 0`.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// Height of the area the boxes cover, starting at `y = 0`.
    pub fn height(&self) -> f64 {
        self.height
    }
}

impl FamilyTree {
    /// Lays the tree out for drawing with the default [`Spacing`].
    ///
    /// See the [`tree_layout`](crate::tree_layout) module for the rules.
    pub fn tree_layout(&self) -> TreeLayout {
        self.tree_layout_with(Spacing::default())
    }

    /// Lays the tree out for drawing with the given box size and gaps.
    pub fn tree_layout_with(&self, spacing: Spacing) -> TreeLayout {
        let generations = self.generation_layout();
        let rows = generations.rows();

        let row_width = |people: usize| -> f64 {
            if people == 0 {
                return 0.0;
            }
            let people = people as f64;
            people * spacing.node_width + (people - 1.0) * spacing.column_gap
        };
        let width = rows
            .iter()
            .map(|row| row_width(row.len()))
            .fold(0.0, f64::max);
        let height = if rows.is_empty() {
            0.0
        } else {
            let rows = rows.len() as f64;
            rows * spacing.node_height + (rows - 1.0) * spacing.row_gap
        };

        let mut nodes = Vec::with_capacity(generations.len());
        let mut index = HashMap::with_capacity(generations.len());
        for (generation, row) in rows.iter().enumerate() {
            let left = (width - row_width(row.len())) / 2.0;
            let y = generation as f64 * (spacing.node_height + spacing.row_gap);
            for (position, person_id) in row.iter().enumerate() {
                index.insert(person_id.clone(), nodes.len());
                nodes.push(Node {
                    person_id: person_id.clone(),
                    placement: Placement {
                        generation,
                        position,
                    },
                    x: left + position as f64 * (spacing.node_width + spacing.column_gap),
                    y,
                    width: spacing.node_width,
                    height: spacing.node_height,
                });
            }
        }

        let links = self.drawable_links();
        let mut edges = Vec::with_capacity(links.spouses.len() + links.parent_child.len());
        for (a, b) in links.spouses {
            let (Some(&a), Some(&b)) = (index.get(a), index.get(b)) else {
                continue;
            };
            let (left, right) = if nodes[a].placement.position <= nodes[b].placement.position {
                (&nodes[a], &nodes[b])
            } else {
                (&nodes[b], &nodes[a])
            };
            edges.push(edge(EdgeKind::Spouse, left, right));
        }
        for (parent, child) in links.parent_child {
            let (Some(&parent), Some(&child)) = (index.get(parent), index.get(child)) else {
                continue;
            };
            edges.push(edge(EdgeKind::ParentChild, &nodes[parent], &nodes[child]));
        }

        TreeLayout {
            nodes,
            edges,
            index,
            width,
            height,
        }
    }
}

/// Builds an edge joining the facing sides of two boxes.
fn edge(kind: EdgeKind, from: &Node, to: &Node) -> Edge {
    let (start, end) = match from.placement.generation.cmp(&to.placement.generation) {
        std::cmp::Ordering::Less => (from.bottom_middle(), to.top_middle()),
        std::cmp::Ordering::Greater => (from.top_middle(), to.bottom_middle()),
        std::cmp::Ordering::Equal if from.x <= to.x => (from.right_middle(), to.left_middle()),
        std::cmp::Ordering::Equal => (from.left_middle(), to.right_middle()),
    };
    Edge {
        kind,
        from: from.person_id.clone(),
        to: to.person_id.clone(),
        start,
        end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Person, Relationship, Sex};

    fn tree_of(people: &[(&str, Option<&str>)]) -> FamilyTree {
        let mut tree = FamilyTree::new();
        for (id, birth) in people {
            tree.add_person(Person::new(
                id.to_string(),
                id.to_uppercase(),
                Sex::Unknown,
                birth.map(str::to_string),
                None,
            ))
            .unwrap();
        }
        tree
    }

    fn parent(tree: &mut FamilyTree, parent: &str, child: &str) {
        tree.add_parent_child_relationship(parent.to_string(), child.to_string())
            .unwrap();
    }

    fn marry(tree: &mut FamilyTree, a: &str, b: &str) {
        tree.add_spouse_relationship(a.to_string(), b.to_string())
            .unwrap();
    }

    /// Grandparents, their son and his wife, and the couple's two children.
    fn three_generations() -> FamilyTree {
        let mut tree = tree_of(&[
            ("grandpa", None),
            ("grandma", None),
            ("dad", None),
            ("mom", None),
            ("anna", Some("2000-01-01")),
            ("ben", Some("2002-01-01")),
        ]);
        marry(&mut tree, "grandpa", "grandma");
        parent(&mut tree, "grandpa", "dad");
        parent(&mut tree, "grandma", "dad");
        marry(&mut tree, "dad", "mom");
        parent(&mut tree, "dad", "anna");
        parent(&mut tree, "mom", "anna");
        parent(&mut tree, "dad", "ben");
        parent(&mut tree, "mom", "ben");
        tree
    }

    fn links(layout: &TreeLayout) -> Vec<(EdgeKind, &str, &str)> {
        layout
            .edges()
            .iter()
            .map(|e| (e.kind, e.from.as_str(), e.to.as_str()))
            .collect()
    }

    fn top_left(layout: &TreeLayout, id: &str) -> (f64, f64) {
        let node = layout.node(id).unwrap();
        (node.x, node.y)
    }

    #[test]
    fn empty_tree_has_nothing_to_draw() {
        let layout = FamilyTree::new().tree_layout();
        assert!(layout.nodes().is_empty());
        assert!(layout.edges().is_empty());
        assert_eq!((layout.width(), layout.height()), (0.0, 0.0));
        assert_eq!(layout.node("nobody"), None);
    }

    #[test]
    fn single_person_fills_the_area() {
        let layout = tree_of(&[("solo", None)]).tree_layout();
        let node = layout.node("solo").unwrap();
        assert_eq!(
            (node.x, node.y, node.width, node.height),
            (0.0, 0.0, 180.0, 60.0)
        );
        assert_eq!((layout.width(), layout.height()), (180.0, 60.0));
        assert_eq!(node.center(), Point { x: 90.0, y: 30.0 });
    }

    #[test]
    fn three_generation_edges() {
        let layout = three_generations().tree_layout();
        assert_eq!(
            links(&layout),
            vec![
                (EdgeKind::Spouse, "dad", "mom"),
                (EdgeKind::Spouse, "grandma", "grandpa"),
                (EdgeKind::ParentChild, "dad", "anna"),
                (EdgeKind::ParentChild, "dad", "ben"),
                (EdgeKind::ParentChild, "grandma", "dad"),
                (EdgeKind::ParentChild, "grandpa", "dad"),
                (EdgeKind::ParentChild, "mom", "anna"),
                (EdgeKind::ParentChild, "mom", "ben"),
            ]
        );
    }

    #[test]
    fn three_generation_coordinates() {
        let layout = three_generations().tree_layout();
        // Every row holds two people, so no row is shifted.
        assert_eq!(top_left(&layout, "grandma"), (0.0, 0.0));
        assert_eq!(top_left(&layout, "grandpa"), (220.0, 0.0));
        assert_eq!(top_left(&layout, "dad"), (0.0, 140.0));
        assert_eq!(top_left(&layout, "mom"), (220.0, 140.0));
        assert_eq!(top_left(&layout, "anna"), (0.0, 280.0));
        assert_eq!(top_left(&layout, "ben"), (220.0, 280.0));
        assert_eq!((layout.width(), layout.height()), (400.0, 340.0));
    }

    #[test]
    fn nodes_follow_the_generation_rows() {
        let tree = three_generations();
        let layout = tree.tree_layout();
        let generations = tree.generation_layout();
        let order: Vec<&str> = layout
            .nodes()
            .iter()
            .map(|n| n.person_id.as_str())
            .collect();
        let rows: Vec<&str> = generations
            .rows()
            .iter()
            .flatten()
            .map(String::as_str)
            .collect();
        assert_eq!(order, rows);
        for node in layout.nodes() {
            assert_eq!(Some(node.placement), generations.placement(&node.person_id));
        }
    }

    #[test]
    fn parent_child_edge_joins_bottom_to_top() {
        let layout = three_generations().tree_layout();
        let edge = layout
            .edges()
            .iter()
            .find(|e| e.from == "mom" && e.to == "ben")
            .unwrap();
        assert_eq!(edge.start, Point { x: 310.0, y: 200.0 });
        assert_eq!(edge.end, Point { x: 310.0, y: 280.0 });
    }

    #[test]
    fn spouse_edge_joins_facing_sides_left_to_right() {
        let layout = three_generations().tree_layout();
        let edge = layout
            .edges()
            .iter()
            .find(|e| e.kind == EdgeKind::Spouse && e.to == "grandpa")
            .unwrap();
        // grandma sits left of grandpa, so she is `from` even though the
        // pair is sorted by id.
        assert_eq!(edge.from, "grandma");
        assert_eq!(edge.start, Point { x: 180.0, y: 30.0 });
        assert_eq!(edge.end, Point { x: 220.0, y: 30.0 });
    }

    #[test]
    fn narrower_rows_are_centred() {
        let mut tree = tree_of(&[("a", None), ("b", None), ("kid", None)]);
        marry(&mut tree, "a", "b");
        parent(&mut tree, "a", "kid");
        let layout = tree.tree_layout();
        assert_eq!(top_left(&layout, "a"), (0.0, 0.0));
        assert_eq!(top_left(&layout, "b"), (220.0, 0.0));
        assert_eq!(top_left(&layout, "kid"), (110.0, 140.0));
        assert_eq!(layout.width(), 400.0);
    }

    #[test]
    fn custom_spacing_is_used() {
        let spacing = Spacing {
            node_width: 10.0,
            node_height: 5.0,
            column_gap: 2.0,
            row_gap: 3.0,
        };
        let layout = three_generations().tree_layout_with(spacing);
        assert_eq!(top_left(&layout, "mom"), (12.0, 8.0));
        assert_eq!(top_left(&layout, "ben"), (12.0, 16.0));
        assert_eq!((layout.width(), layout.height()), (22.0, 21.0));
        let node = layout.node("ben").unwrap();
        assert_eq!((node.width, node.height), (10.0, 5.0));
    }

    #[test]
    fn each_relationship_is_drawn_once_whatever_its_direction() {
        let mut tree = tree_of(&[("p", None), ("c", None), ("x", None), ("y", None)]);
        // One direction only, the inverse only, and both directions.
        tree.add_relationship("p".to_string(), "c".to_string(), Relationship::Parent)
            .unwrap();
        tree.add_relationship("c".to_string(), "p".to_string(), Relationship::Child)
            .unwrap();
        tree.add_relationship("y".to_string(), "x".to_string(), Relationship::Spouse)
            .unwrap();
        let layout = tree.tree_layout();
        assert_eq!(
            links(&layout),
            vec![
                (EdgeKind::Spouse, "x", "y"),
                (EdgeKind::ParentChild, "p", "c")
            ]
        );
    }

    #[test]
    fn siblings_and_self_links_are_not_drawn() {
        let mut tree = tree_of(&[("a", None), ("b", None)]);
        tree.add_relationship("a".to_string(), "b".to_string(), Relationship::Sibling)
            .unwrap();
        tree.add_relationship("a".to_string(), "a".to_string(), Relationship::Spouse)
            .unwrap();
        assert!(tree.tree_layout().edges().is_empty());
    }

    #[test]
    fn relationships_to_missing_people_are_ignored() {
        let mut tree = tree_of(&[("a", None)]);
        tree.relationships
            .entry("a".to_string())
            .or_default()
            .push(("ghost".to_string(), Relationship::Parent));
        let layout = tree.tree_layout();
        assert_eq!(layout.nodes().len(), 1);
        assert!(layout.edges().is_empty());
    }

    #[test]
    fn parent_cycle_still_draws_every_edge() {
        let mut tree = tree_of(&[("a", None), ("b", None)]);
        parent(&mut tree, "a", "b");
        parent(&mut tree, "b", "a");
        let layout = tree.tree_layout();
        assert_eq!(layout.nodes().len(), 2);
        assert_eq!(layout.edges().len(), 2);
        for edge in layout.edges() {
            let from = layout.node(&edge.from).unwrap();
            let to = layout.node(&edge.to).unwrap();
            // Whichever way the edge points, it joins the facing sides.
            if from.y < to.y {
                assert_eq!(
                    (edge.start, edge.end),
                    (from.bottom_middle(), to.top_middle())
                );
            } else if from.y > to.y {
                assert_eq!(
                    (edge.start, edge.end),
                    (from.top_middle(), to.bottom_middle())
                );
            } else if from.x < to.x {
                assert_eq!(
                    (edge.start, edge.end),
                    (from.right_middle(), to.left_middle())
                );
            } else {
                assert_eq!(
                    (edge.start, edge.end),
                    (from.left_middle(), to.right_middle())
                );
            }
        }
    }

    #[test]
    fn layout_does_not_depend_on_insertion_order() {
        let forward = three_generations().tree_layout();
        let mut reversed = tree_of(&[
            ("ben", Some("2002-01-01")),
            ("anna", Some("2000-01-01")),
            ("mom", None),
            ("dad", None),
            ("grandma", None),
            ("grandpa", None),
        ]);
        parent(&mut reversed, "mom", "ben");
        parent(&mut reversed, "dad", "ben");
        parent(&mut reversed, "mom", "anna");
        parent(&mut reversed, "dad", "anna");
        marry(&mut reversed, "mom", "dad");
        parent(&mut reversed, "grandma", "dad");
        parent(&mut reversed, "grandpa", "dad");
        marry(&mut reversed, "grandma", "grandpa");
        assert_eq!(forward, reversed.tree_layout());
    }
}
