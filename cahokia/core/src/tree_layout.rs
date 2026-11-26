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
//! - Spouse edges run straight between the facing sides of the two boxes,
//!   left spouse first.
//! - Parent-child lines are routed per family, not per parent. A family is
//!   the children of one couple (two spouses sitting next to each other, who
//!   are both the child's parents) or of one single parent. Its line drops
//!   from the middle of the couple's spouse line, or from the bottom middle
//!   of the single parent's box, to a horizontal bar in the gap above the
//!   children, and from the bar into the top middle of each child's box.
//!   Each parent-child [`Edge`] is that family route for one parent and one
//!   child, so a couple's two edges to the same child share one route.
//! - Families whose bars would overlap in the same gap get their own bar
//!   heights, spread evenly down the gap, so you can always tell whose
//!   children are whose. Families whose bars don't overlap share the middle
//!   of the gap.
//! - When bad data puts a parent and child in rows that are not next to
//!   each other, the edge joins the facing sides of the two boxes and bends
//!   half a row gap from the child.
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
    /// Where the line starts: the `from` box, or for a couple's child the
    /// middle of the couple's spouse line.
    pub start: Point,
    /// Where the line reaches the `to` box.
    pub end: Point,
    /// The corners of the line from `start` to `end`, both included. Every
    /// segment is horizontal or vertical, except a spouse line between
    /// boxes in different rows (bad data).
    pub route: Vec<Point>,
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
/// assert_eq!(edge.route, vec![edge.start, edge.end]);
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
        let mut couples = HashMap::new();
        for (a, b) in links.spouses {
            let (Some(&a), Some(&b)) = (index.get(a), index.get(b)) else {
                continue;
            };
            let (left, right) = if nodes[a].placement.position <= nodes[b].placement.position {
                (a, b)
            } else {
                (b, a)
            };
            let (l, r) = (&nodes[left].placement, &nodes[right].placement);
            if l.generation == r.generation && l.position + 1 == r.position {
                couples.insert((left.min(right), left.max(right)), (left, right));
            }
            let edge = facing_edge(EdgeKind::Spouse, &nodes[left], &nodes[right]);
            edges.push(Edge {
                route: vec![edge.start, edge.end],
                ..edge
            });
        }

        let parent_child: Vec<(usize, usize)> = links
            .parent_child
            .iter()
            .filter_map(|(parent, child)| Some((*index.get(*parent)?, *index.get(*child)?)))
            .collect();
        let mut parents_of: HashMap<usize, Vec<usize>> = HashMap::new();
        for &(parent, child) in &parent_child {
            parents_of.entry(child).or_default().push(parent);
        }

        // Group the links into families keyed by who the line drops from.
        let stem_of = |parent: usize, child: usize| -> Stem {
            if let [a, b] = parents_of[&child][..]
                && let Some(&(left, right)) = couples.get(&(a.min(b), a.max(b)))
            {
                return Stem::Couple(left, right);
            }
            Stem::Single(parent)
        };
        let mut families: Vec<Family> = Vec::new();
        let mut family_of = HashMap::new();
        for &(parent, child) in &parent_child {
            let stem = stem_of(parent, child);
            let point = match stem {
                Stem::Couple(left, right) => Point {
                    x: (nodes[left].x + nodes[left].width + nodes[right].x) / 2.0,
                    y: nodes[left].y + nodes[left].height / 2.0,
                },
                Stem::Single(parent) => nodes[parent].bottom_middle(),
            };
            let generation = nodes[match stem {
                Stem::Couple(left, _) | Stem::Single(left) => left,
            }]
            .placement
            .generation;
            if nodes[child].placement.generation != generation + 1 {
                continue;
            }
            let family = *family_of.entry(stem).or_insert_with(|| {
                families.push(Family {
                    stem: point,
                    child_generation: generation + 1,
                    left: point.x,
                    right: point.x,
                    bar: 0.0,
                });
                families.len() - 1
            });
            let x = nodes[child].top_middle().x;
            let family = &mut families[family];
            family.left = family.left.min(x);
            family.right = family.right.max(x);
        }
        assign_bars(&mut families, spacing);

        for (parent, child) in parent_child {
            let (from, to) = (&nodes[parent], &nodes[child]);
            let route = match family_of.get(&stem_of(parent, child)) {
                Some(&family) if to.placement.generation == families[family].child_generation => {
                    let family = &families[family];
                    let end = to.top_middle();
                    if family.stem.x == end.x {
                        vec![family.stem, end]
                    } else {
                        vec![
                            family.stem,
                            Point {
                                x: family.stem.x,
                                y: family.bar,
                            },
                            Point {
                                x: end.x,
                                y: family.bar,
                            },
                            end,
                        ]
                    }
                }
                _ => {
                    let edge = facing_edge(EdgeKind::ParentChild, from, to);
                    stray_route(edge.start, edge.end, spacing.row_gap)
                }
            };
            edges.push(Edge {
                kind: EdgeKind::ParentChild,
                from: from.person_id.clone(),
                to: to.person_id.clone(),
                start: route[0],
                end: route[route.len() - 1],
                route,
            });
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

/// What a family's line drops from, by node index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Stem {
    /// The middle of the spouse line between two neighbouring spouses, left
    /// first.
    Couple(usize, usize),
    /// The bottom of one parent's box.
    Single(usize),
}

/// The line from one couple or single parent down to their children in the
/// next row.
struct Family {
    /// Where the line leaves the parents.
    stem: Point,
    /// Row of the children.
    child_generation: usize,
    /// Leftmost `x` the bar has to reach.
    left: f64,
    /// Rightmost `x` the bar has to reach.
    right: f64,
    /// Height of the bar.
    bar: f64,
}

/// Gives each family a bar height in the gap above its children. Families
/// whose bars overlap (or touch) get separate heights; the heights used in
/// a gap are spread evenly down it.
fn assign_bars(families: &mut [Family], spacing: Spacing) {
    let mut order: Vec<usize> = (0..families.len()).collect();
    order.sort_by(|&a, &b| {
        let (a, b) = (&families[a], &families[b]);
        a.child_generation
            .cmp(&b.child_generation)
            .then(a.left.total_cmp(&b.left))
            .then(a.right.total_cmp(&b.right))
            .then(a.stem.x.total_cmp(&b.stem.x))
            .then(a.stem.y.total_cmp(&b.stem.y))
    });
    let generation: Vec<usize> = families.iter().map(|f| f.child_generation).collect();
    for gap in order.chunk_by(|&a, &b| generation[a] == generation[b]) {
        // Greedy interval colouring: each family takes the highest track
        // whose last bar ends before this one starts.
        let mut track_ends: Vec<f64> = Vec::new();
        let mut tracks = Vec::with_capacity(gap.len());
        for &family in gap {
            let family = &families[family];
            let track = match track_ends.iter().position(|&end| end < family.left) {
                Some(track) => track,
                None => {
                    track_ends.push(f64::NEG_INFINITY);
                    track_ends.len() - 1
                }
            };
            track_ends[track] = family.right;
            tracks.push(track);
        }
        let step = spacing.row_gap / (track_ends.len() + 1) as f64;
        for (&family, track) in gap.iter().zip(tracks) {
            let family = &mut families[family];
            let gap_top = family.child_generation as f64 * (spacing.node_height + spacing.row_gap)
                - spacing.row_gap;
            family.bar = gap_top + step * (track + 1) as f64;
        }
    }
}

/// Route for a parent-child line between rows that are not next to each
/// other: straight when the ends line up or share a row, otherwise bending
/// half a row gap from the child.
fn stray_route(start: Point, end: Point, row_gap: f64) -> Vec<Point> {
    if start.x == end.x || start.y == end.y {
        return vec![start, end];
    }
    let direction = if end.y >= start.y { 1.0 } else { -1.0 };
    let bar = end.y - direction * row_gap / 2.0;
    vec![
        start,
        Point { x: start.x, y: bar },
        Point { x: end.x, y: bar },
        end,
    ]
}

/// Builds an edge joining the facing sides of two boxes, with no route yet.
fn facing_edge(kind: EdgeKind, from: &Node, to: &Node) -> Edge {
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
        route: Vec::new(),
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

    fn route<'a>(layout: &'a TreeLayout, from: &str, to: &str) -> &'a [Point] {
        &layout
            .edges()
            .iter()
            .find(|e| e.kind == EdgeKind::ParentChild && e.from == from && e.to == to)
            .unwrap()
            .route
    }

    fn pt(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn single_parent_line_drops_from_the_bottom_of_the_box() {
        let mut tree = tree_of(&[("p", None), ("a", Some("2000")), ("b", Some("2001"))]);
        parent(&mut tree, "p", "a");
        parent(&mut tree, "p", "b");
        let layout = tree.tree_layout();
        // p is centred over a (x 0) and b (x 220), so its middle is at 200.
        assert_eq!(
            route(&layout, "p", "a"),
            [
                pt(200.0, 60.0),
                pt(200.0, 100.0),
                pt(90.0, 100.0),
                pt(90.0, 140.0)
            ]
        );
        assert_eq!(
            route(&layout, "p", "b"),
            [
                pt(200.0, 60.0),
                pt(200.0, 100.0),
                pt(310.0, 100.0),
                pt(310.0, 140.0)
            ]
        );
    }

    #[test]
    fn couple_line_drops_from_the_middle_of_the_spouse_line() {
        let layout = three_generations().tree_layout();
        // dad (x 0) and mom (x 220) sit side by side, so their line leaves
        // the middle of the spouse line at x 200, box middle height 170.
        let to_anna = [
            pt(200.0, 170.0),
            pt(200.0, 240.0),
            pt(90.0, 240.0),
            pt(90.0, 280.0),
        ];
        let to_ben = [
            pt(200.0, 170.0),
            pt(200.0, 240.0),
            pt(310.0, 240.0),
            pt(310.0, 280.0),
        ];
        assert_eq!(route(&layout, "dad", "anna"), to_anna);
        assert_eq!(route(&layout, "mom", "anna"), to_anna);
        assert_eq!(route(&layout, "dad", "ben"), to_ben);
        assert_eq!(route(&layout, "mom", "ben"), to_ben);
        let edge = layout
            .edges()
            .iter()
            .find(|e| e.from == "mom" && e.to == "ben")
            .unwrap();
        assert_eq!((edge.start, edge.end), (pt(200.0, 170.0), pt(310.0, 280.0)));
    }

    #[test]
    fn child_directly_below_the_stem_gets_a_straight_line() {
        let mut tree = tree_of(&[("p", None), ("kid", None)]);
        parent(&mut tree, "p", "kid");
        let layout = tree.tree_layout();
        assert_eq!(
            route(&layout, "p", "kid"),
            [pt(90.0, 60.0), pt(90.0, 140.0)]
        );
    }

    /// Two married couples side by side, each with two children, so the
    /// children's row holds both families next to each other.
    fn two_couples() -> FamilyTree {
        let mut tree = tree_of(&[
            ("a1", None),
            ("a2", None),
            ("b1", None),
            ("b2", None),
            ("ak1", Some("2000")),
            ("ak2", Some("2001")),
            ("bk1", Some("2002")),
            ("bk2", Some("2003")),
        ]);
        marry(&mut tree, "a1", "a2");
        marry(&mut tree, "b1", "b2");
        for (parents, kids) in [
            (["a1", "a2"], ["ak1", "ak2"]),
            (["b1", "b2"], ["bk1", "bk2"]),
        ] {
            for p in parents {
                for k in kids {
                    parent(&mut tree, p, k);
                }
            }
        }
        tree
    }

    /// Horizontal bars of all parent-child routes as (y, left x, right x).
    fn bars(layout: &TreeLayout) -> Vec<(f64, f64, f64)> {
        let mut bars: Vec<_> = layout
            .edges()
            .iter()
            .filter(|e| e.kind == EdgeKind::ParentChild)
            .flat_map(|e| e.route.windows(2).map(|w| (w[0], w[1])).collect::<Vec<_>>())
            .filter(|(a, b)| a.y == b.y)
            .map(|(a, b)| (a.y, a.x.min(b.x), a.x.max(b.x)))
            .collect();
        bars.sort_by(|a, b| a.partial_cmp(b).unwrap());
        bars.dedup();
        bars
    }

    #[test]
    fn neighbouring_families_get_separate_bars() {
        let layout = two_couples().tree_layout();
        // Each family's bar spans only its own stem and children, with a
        // gap between the two, so they can't read as one line.
        let bars = bars(&layout);
        let a_stem = route(&layout, "a1", "ak1")[0].x;
        let b_stem = route(&layout, "b1", "bk1")[0].x;
        let ak = [&layout.node("ak1").unwrap(), &layout.node("ak2").unwrap()];
        let bk = [&layout.node("bk1").unwrap(), &layout.node("bk2").unwrap()];
        let a_span = (ak[0].center().x.min(a_stem), ak[1].center().x.max(a_stem));
        let b_span = (bk[0].center().x.min(b_stem), bk[1].center().x.max(b_stem));
        assert!(a_span.1 < b_span.0, "{a_span:?} overlaps {b_span:?}");
        for bar in &bars {
            let inside_a = bar.1 >= a_span.0 && bar.2 <= a_span.1;
            let inside_b = bar.1 >= b_span.0 && bar.2 <= b_span.1;
            assert!(inside_a ^ inside_b, "{bar:?} crosses families");
        }
    }

    #[test]
    fn overlapping_families_get_their_own_bar_heights() {
        // b is married to both a and c (a remarriage), so the row reads
        // a, b, c. a+b's children are born between c+b's, so their bars
        // would overlap at one height.
        let mut tree = tree_of(&[
            ("a", None),
            ("b", None),
            ("c", None),
            ("k1", Some("2000")),
            ("k2", Some("2001")),
            ("k3", Some("2002")),
            ("k4", Some("2003")),
        ]);
        marry(&mut tree, "a", "b");
        marry(&mut tree, "b", "c");
        for k in ["k1", "k3"] {
            parent(&mut tree, "a", k);
            parent(&mut tree, "b", k);
        }
        for k in ["k2", "k4"] {
            parent(&mut tree, "b", k);
            parent(&mut tree, "c", k);
        }
        let layout = tree.tree_layout();
        let ab = route(&layout, "a", "k1");
        let bc = route(&layout, "c", "k2");
        assert_ne!(ab[0], bc[0], "the two couples have their own stems");
        let (ab_bar, bc_bar) = (ab[1].y, bc[1].y);
        assert_ne!(ab_bar, bc_bar);
        // Both bars sit inside the gap between rows 0 and 1 (y 60 to 140),
        // splitting it into thirds.
        let mut heights = [ab_bar, bc_bar];
        heights.sort_by(f64::total_cmp);
        let third = 80.0 / 3.0;
        assert_eq!(heights, [60.0 + third, 60.0 + 2.0 * third]);
        // Every child of a family hangs off that family's bar.
        for (p, k) in [("a", "k1"), ("b", "k3")] {
            assert_eq!(route(&layout, p, k)[1].y, ab_bar);
        }
        for (p, k) in [("b", "k2"), ("c", "k4")] {
            assert_eq!(route(&layout, p, k)[1].y, bc_bar);
        }
    }

    #[test]
    fn co_parents_who_are_not_spouses_each_drop_their_own_line() {
        let mut tree = tree_of(&[("a", None), ("b", None), ("kid", None)]);
        parent(&mut tree, "a", "kid");
        parent(&mut tree, "b", "kid");
        let layout = tree.tree_layout();
        let from_a = route(&layout, "a", "kid");
        let from_b = route(&layout, "b", "kid");
        assert_eq!(from_a[0], layout.node("a").unwrap().bottom_middle());
        assert_eq!(from_b[0], layout.node("b").unwrap().bottom_middle());
        // They meet at the child, so their bars must not share a height.
        assert_ne!(from_a[1].y, from_b[1].y);
    }

    #[test]
    fn routes_bend_at_right_angles_and_end_at_the_boxes() {
        let layout = two_couples().tree_layout();
        for edge in layout.edges() {
            assert_eq!(edge.route.first(), Some(&edge.start));
            assert_eq!(edge.route.last(), Some(&edge.end));
            for w in edge.route.windows(2) {
                assert!(w[0].x == w[1].x || w[0].y == w[1].y, "{edge:?}");
            }
            if edge.kind == EdgeKind::ParentChild {
                assert_eq!(edge.end, layout.node(&edge.to).unwrap().top_middle());
            }
        }
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
        assert_eq!(edge.route, vec![edge.start, edge.end]);
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
            assert_eq!(edge.route.first(), Some(&edge.start));
            assert_eq!(edge.route.last(), Some(&edge.end));
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
