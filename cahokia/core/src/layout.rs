//! Generation layout for drawing a [`FamilyTree`].
//!
//! [`FamilyTree::generation_layout`] places every person in a row (their
//! generation, with parents above children) and at a position within that
//! row. Renderers turn the rows into coordinates; the relationship walking
//! happens here so every platform draws the same tree.
//!
//! The rules:
//!
//! - Spouses are kept together as one "family unit". A unit sits one row
//!   below its lowest parent unit, so a person who married into the family
//!   shares their partner's row.
//! - A unit with no parents in the tree moves down to sit just above its
//!   highest child, so an unmarried co-parent lines up with the other parent.
//! - Within a row, units follow the order of their parents in the rows
//!   above, then the earliest birth date, then the smallest person id. Inside
//!   a unit, spouses are chained so that couples and remarriages (A-B, B-C)
//!   sit next to each other.
//! - The result never depends on `HashMap` iteration order, so it is stable
//!   across runs.
//! - Disconnected people get their own units. Relationship cycles (bad data,
//!   such as someone listed as their own grandparent) are broken at the
//!   smallest remaining unit instead of panicking.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::{FamilyTree, Relationship};

/// Where one person sits in a [`GenerationLayout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Placement {
    /// Row of the person, counted from 0 at the top (oldest generation).
    pub generation: usize,
    /// Position of the person within their row, counted from 0 at the left.
    pub position: usize,
}

/// People of a [`FamilyTree`] arranged into generation rows.
///
/// # Example
/// ```
/// # use core::{FamilyTree, Person, Placement, Sex};
/// #
/// let mut tree = FamilyTree::new();
/// for (id, name) in [("p1", "John"), ("p2", "Mary"), ("p3", "Jane")] {
///     tree.add_person(Person::new(id.to_string(), name.to_string(), Sex::Unknown, None, None))
///         .unwrap();
/// }
/// tree.add_spouse_relationship("p1".to_string(), "p2".to_string()).unwrap();
/// tree.add_parent_child_relationship("p1".to_string(), "p3".to_string()).unwrap();
///
/// let layout = tree.generation_layout();
/// assert_eq!(layout.rows(), &[vec!["p1".to_string(), "p2".to_string()], vec!["p3".to_string()]]);
/// assert_eq!(layout.placement("p3"), Some(Placement { generation: 1, position: 0 }));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GenerationLayout {
    rows: Vec<Vec<String>>,
    placements: HashMap<String, Placement>,
}

impl GenerationLayout {
    /// The person ids in each row, top row first, each row left to right.
    pub fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }

    /// Where the person with this id sits, or `None` if they are not in the tree.
    pub fn placement(&self, person_id: &str) -> Option<Placement> {
        self.placements.get(person_id).copied()
    }

    /// Number of people placed.
    pub fn len(&self) -> usize {
        self.placements.len()
    }

    /// True if no one is placed.
    pub fn is_empty(&self) -> bool {
        self.placements.is_empty()
    }
}

impl FamilyTree {
    /// Arranges everyone in the tree into generation rows.
    ///
    /// See the [`layout`](crate::layout) module for the rules.
    pub fn generation_layout(&self) -> GenerationLayout {
        Layouter::new(self).run()
    }
}

/// Working state for one layout. People are referred to by their index in
/// `ids`, which is sorted, so every tie-break by index is a tie-break by id.
struct Layouter<'a> {
    tree: &'a FamilyTree,
    ids: Vec<&'a str>,
    /// Spouse pairs, symmetric, sorted.
    spouses: Vec<BTreeSet<usize>>,
    /// Parent pairs as (parent, child).
    parent_child: BTreeSet<(usize, usize)>,
}

impl<'a> Layouter<'a> {
    fn new(tree: &'a FamilyTree) -> Self {
        let mut ids: Vec<&str> = tree.people.keys().map(String::as_str).collect();
        ids.sort_unstable();
        let index: HashMap<&str, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();

        let mut spouses = vec![BTreeSet::new(); ids.len()];
        let mut parent_child = BTreeSet::new();
        for (from, rels) in &tree.relationships {
            let Some(&a) = index.get(from.as_str()) else {
                continue;
            };
            for (to, relationship) in rels {
                let Some(&b) = index.get(to.as_str()) else {
                    continue;
                };
                if a == b {
                    continue;
                }
                match relationship {
                    Relationship::Spouse => {
                        spouses[a].insert(b);
                        spouses[b].insert(a);
                    }
                    Relationship::Parent => {
                        parent_child.insert((a, b));
                    }
                    Relationship::Child => {
                        parent_child.insert((b, a));
                    }
                    Relationship::Sibling => {}
                }
            }
        }

        Layouter {
            tree,
            ids,
            spouses,
            parent_child,
        }
    }

    fn run(self) -> GenerationLayout {
        let units = self.family_units();
        let unit_of = unit_lookup(&units, self.ids.len());

        let mut parent_units = vec![BTreeSet::new(); units.len()];
        let mut child_units = vec![BTreeSet::new(); units.len()];
        for &(parent, child) in &self.parent_child {
            let (p, c) = (unit_of[parent], unit_of[child]);
            if p != c {
                parent_units[c].insert(p);
                child_units[p].insert(c);
            }
        }

        let generations = assign_generations(&parent_units, &child_units);
        let rows = self.order_rows(&units, &generations, &parent_units);

        let mut placements = HashMap::with_capacity(self.ids.len());
        let rows: Vec<Vec<String>> = rows
            .into_iter()
            .enumerate()
            .map(|(generation, row)| {
                row.into_iter()
                    .enumerate()
                    .map(|(position, person)| {
                        let id = self.ids[person].to_string();
                        placements.insert(
                            id.clone(),
                            Placement {
                                generation,
                                position,
                            },
                        );
                        id
                    })
                    .collect()
            })
            .collect();

        GenerationLayout { rows, placements }
    }

    /// Groups people joined by spouse links into units, each listed in seat
    /// order. Units are sorted by their smallest member.
    fn family_units(&self) -> Vec<Vec<usize>> {
        let mut seen = vec![false; self.ids.len()];
        let mut units = Vec::new();
        for start in 0..self.ids.len() {
            if seen[start] {
                continue;
            }
            // Collect the connected component.
            let mut members = BTreeSet::new();
            let mut stack = vec![start];
            while let Some(person) = stack.pop() {
                if members.insert(person) {
                    seen[person] = true;
                    stack.extend(self.spouses[person].iter().copied());
                }
            }
            units.push(self.seat_order(&members));
        }
        units
    }

    /// Orders a unit's members by walking spouse links depth first from the
    /// member with the fewest spouses, so a chain of marriages stays a chain.
    fn seat_order(&self, members: &BTreeSet<usize>) -> Vec<usize> {
        let first = members
            .iter()
            .copied()
            .min_by_key(|&m| (self.spouses[m].len(), m))
            .unwrap_or_default();
        let mut order = Vec::with_capacity(members.len());
        let mut placed = BTreeSet::new();
        let mut stack = vec![first];
        while let Some(person) = stack.pop() {
            if placed.insert(person) {
                order.push(person);
                // Reverse so the smallest neighbour is visited first.
                stack.extend(self.spouses[person].iter().rev().copied());
            }
        }
        order
    }

    /// Splits units into rows and orders each row, top to bottom.
    fn order_rows(
        &self,
        units: &[Vec<usize>],
        generations: &[usize],
        parent_units: &[BTreeSet<usize>],
    ) -> Vec<Vec<usize>> {
        let mut by_generation: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for (unit, &generation) in generations.iter().enumerate() {
            by_generation.entry(generation).or_default().push(unit);
        }

        // Sequence number of each unit once its row is ordered, used to keep
        // children in the same left-to-right order as their parents.
        let mut sequence = vec![usize::MAX; units.len()];
        let mut next = 0;
        let mut rows = Vec::with_capacity(by_generation.len());
        for (_, mut row) in by_generation {
            row.sort_by_cached_key(|&unit| {
                let parent_order = parent_units[unit]
                    .iter()
                    .map(|&p| sequence[p])
                    .min()
                    .unwrap_or(usize::MAX);
                let birth = units[unit].iter().filter_map(|&m| self.birth_date(m)).min();
                let smallest = units[unit].iter().min().copied();
                (parent_order, birth.is_none(), birth, smallest)
            });
            for &unit in &row {
                sequence[unit] = next;
                next += 1;
            }
            rows.push(
                row.into_iter()
                    .flat_map(|unit| units[unit].iter().copied())
                    .collect(),
            );
        }
        rows
    }

    fn birth_date(&self, person: usize) -> Option<&'a str> {
        self.tree
            .people
            .get(self.ids[person])
            .and_then(|p| p.birth_date.as_deref())
    }
}

/// Maps each person index to the index of their unit.
fn unit_lookup(units: &[Vec<usize>], people: usize) -> Vec<usize> {
    let mut unit_of = vec![0; people];
    for (unit, members) in units.iter().enumerate() {
        for &member in members {
            unit_of[member] = unit;
        }
    }
    unit_of
}

/// Gives each unit a generation one below its lowest parent unit (longest
/// path layering), breaking cycles at the smallest unprocessed unit, then
/// moves parentless units down to sit just above their highest child.
fn assign_generations(
    parent_units: &[BTreeSet<usize>],
    child_units: &[BTreeSet<usize>],
) -> Vec<usize> {
    let count = parent_units.len();
    let mut waiting_on: Vec<usize> = parent_units.iter().map(BTreeSet::len).collect();
    let mut generation: Vec<usize> = vec![0; count];
    let mut done = vec![false; count];
    let mut ready: BTreeSet<usize> = (0..count).filter(|&u| waiting_on[u] == 0).collect();
    let mut next_unforced = 0;

    for _ in 0..count {
        let unit = match ready.pop_first() {
            Some(unit) => unit,
            None => {
                // Only cycles remain: break one at the smallest unit left.
                while done[next_unforced] {
                    next_unforced += 1;
                }
                next_unforced
            }
        };
        done[unit] = true;
        generation[unit] = parent_units[unit]
            .iter()
            .filter(|&&p| done[p])
            .map(|&p| generation[p] + 1)
            .max()
            .unwrap_or(0);
        for &child in &child_units[unit] {
            if !done[child] {
                waiting_on[child] -= 1;
                if waiting_on[child] == 0 {
                    ready.insert(child);
                }
            }
        }
    }

    for unit in 0..count {
        if parent_units[unit].is_empty()
            && let Some(highest_child) = child_units[unit].iter().map(|&c| generation[c]).min()
        {
            generation[unit] = generation[unit].max(highest_child.saturating_sub(1));
        }
    }

    generation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Person, Sex};

    fn person(id: &str, birth: Option<&str>) -> Person {
        Person::new(
            id.to_string(),
            id.to_uppercase(),
            Sex::Unknown,
            birth.map(str::to_string),
            None,
        )
    }

    fn tree_of(ids: &[&str]) -> FamilyTree {
        let mut tree = FamilyTree::new();
        for id in ids {
            tree.add_person(person(id, None)).unwrap();
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

    fn rows(tree: &FamilyTree) -> Vec<Vec<String>> {
        tree.generation_layout().rows().to_vec()
    }

    fn generation(tree: &FamilyTree, id: &str) -> usize {
        tree.generation_layout().placement(id).unwrap().generation
    }

    fn assert_everyone_placed_once(tree: &FamilyTree) {
        let layout = tree.generation_layout();
        let placed: Vec<&String> = layout.rows().iter().flatten().collect();
        let unique: BTreeSet<&String> = placed.iter().copied().collect();
        assert_eq!(placed.len(), tree.people.len());
        assert_eq!(unique.len(), tree.people.len());
        assert_eq!(layout.len(), tree.people.len());
        for (generation, row) in layout.rows().iter().enumerate() {
            assert!(!row.is_empty(), "row {generation} is empty");
            for (position, id) in row.iter().enumerate() {
                assert_eq!(
                    layout.placement(id),
                    Some(Placement {
                        generation,
                        position
                    })
                );
            }
        }
    }

    fn assert_adjacent(tree: &FamilyTree, a: &str, b: &str) {
        let layout = tree.generation_layout();
        let (pa, pb) = (layout.placement(a).unwrap(), layout.placement(b).unwrap());
        assert_eq!(pa.generation, pb.generation, "{a} and {b} not in same row");
        assert_eq!(
            pa.position.abs_diff(pb.position),
            1,
            "{a} and {b} not adjacent"
        );
    }

    #[test]
    fn empty_tree_has_no_rows() {
        let layout = FamilyTree::new().generation_layout();
        assert!(layout.is_empty());
        assert!(layout.rows().is_empty());
        assert_eq!(layout.placement("nobody"), None);
    }

    #[test]
    fn single_person_is_top_left() {
        let tree = tree_of(&["a"]);
        assert_eq!(rows(&tree), vec![vec!["a"]]);
    }

    #[test]
    fn parents_sit_above_children_across_three_generations() {
        let mut tree = tree_of(&["grandma", "mom", "kid"]);
        parent(&mut tree, "grandma", "mom");
        parent(&mut tree, "mom", "kid");
        assert_eq!(rows(&tree), vec![vec!["grandma"], vec!["mom"], vec!["kid"]]);
    }

    #[test]
    fn child_sits_below_lowest_parent() {
        // b is a grandchild of a and also a child of a directly.
        let mut tree = tree_of(&["a", "m", "b"]);
        parent(&mut tree, "a", "m");
        parent(&mut tree, "m", "b");
        parent(&mut tree, "a", "b");
        assert_eq!(generation(&tree, "b"), 2);
    }

    #[test]
    fn one_directional_relationships_are_understood() {
        let mut tree = tree_of(&["p", "c", "d"]);
        tree.add_relationship("p".to_string(), "c".to_string(), Relationship::Parent)
            .unwrap();
        tree.add_relationship("d".to_string(), "p".to_string(), Relationship::Child)
            .unwrap();
        tree.add_relationship("c".to_string(), "d".to_string(), Relationship::Sibling)
            .unwrap();
        assert_eq!(rows(&tree), vec![vec!["p"], vec!["c", "d"]]);
    }

    #[test]
    fn spouses_sit_next_to_each_other() {
        let mut tree = tree_of(&["a", "m", "z"]);
        marry(&mut tree, "a", "z");
        assert_adjacent(&tree, "a", "z");
        assert_eq!(rows(&tree), vec![vec!["a", "z", "m"]]);
    }

    #[test]
    fn married_in_spouse_shares_partners_row() {
        let mut tree = tree_of(&["gp", "dad", "mom", "kid"]);
        parent(&mut tree, "gp", "dad");
        marry(&mut tree, "dad", "mom");
        parent(&mut tree, "dad", "kid");
        parent(&mut tree, "mom", "kid");
        assert_eq!(
            rows(&tree),
            vec![vec!["gp"], vec!["dad", "mom"], vec!["kid"]]
        );
    }

    #[test]
    fn unmarried_co_parent_moves_down_to_other_parent() {
        let mut tree = tree_of(&["gp", "x", "y", "kid"]);
        parent(&mut tree, "gp", "x");
        parent(&mut tree, "x", "kid");
        parent(&mut tree, "y", "kid");
        assert_eq!(generation(&tree, "y"), 1);
        assert_eq!(generation(&tree, "kid"), 2);
    }

    #[test]
    fn remarriage_chain_stays_adjacent() {
        // b married a, then c. b must sit between them.
        let mut tree = tree_of(&["a", "b", "c"]);
        marry(&mut tree, "b", "a");
        marry(&mut tree, "b", "c");
        assert_eq!(rows(&tree), vec![vec!["a", "b", "c"]]);
        assert_adjacent(&tree, "a", "b");
        assert_adjacent(&tree, "b", "c");
    }

    #[test]
    fn siblings_order_by_birth_date_then_id() {
        let mut tree = FamilyTree::new();
        tree.add_person(person("p", None)).unwrap();
        tree.add_person(person("a", Some("1990-01-01"))).unwrap();
        tree.add_person(person("b", Some("1980-01-01"))).unwrap();
        tree.add_person(person("c", None)).unwrap();
        tree.add_person(person("d", Some("1990-01-01"))).unwrap();
        for child in ["a", "b", "c", "d"] {
            parent(&mut tree, "p", child);
        }
        assert_eq!(rows(&tree), vec![vec!["p"], vec!["b", "a", "d", "c"]]);
    }

    #[test]
    fn children_follow_their_parents_order() {
        // Family 1 (a) is left of family 2 (b), so a's child z is left of
        // b's child y even though y sorts first by id.
        let mut tree = tree_of(&["a", "b", "y", "z"]);
        parent(&mut tree, "a", "z");
        parent(&mut tree, "b", "y");
        assert_eq!(rows(&tree), vec![vec!["a", "b"], vec!["z", "y"]]);
    }

    #[test]
    fn disconnected_people_are_placed() {
        let mut tree = tree_of(&["loner", "p", "c", "other"]);
        parent(&mut tree, "p", "c");
        assert_everyone_placed_once(&tree);
        assert_eq!(generation(&tree, "loner"), 0);
        assert_eq!(generation(&tree, "other"), 0);
    }

    #[test]
    fn layout_does_not_depend_on_insertion_order() {
        let ids = ["gp", "dad", "mom", "kid1", "kid2", "aunt", "loner"];
        let build = |order: &[&str]| {
            let mut tree = tree_of(order);
            marry(&mut tree, "mom", "dad");
            parent(&mut tree, "gp", "dad");
            parent(&mut tree, "gp", "aunt");
            for kid in ["kid2", "kid1"] {
                parent(&mut tree, "dad", kid);
                parent(&mut tree, "mom", kid);
            }
            tree.generation_layout()
        };
        let forward = build(&ids);
        let mut reversed = ids;
        reversed.reverse();
        assert_eq!(forward, build(&reversed));
        for _ in 0..20 {
            assert_eq!(forward, build(&ids));
        }
    }

    #[test]
    fn parent_cycle_does_not_panic() {
        let mut tree = tree_of(&["a", "b", "c"]);
        parent(&mut tree, "a", "b");
        parent(&mut tree, "b", "c");
        parent(&mut tree, "c", "a");
        assert_everyone_placed_once(&tree);
        // Broken at the smallest id.
        assert_eq!(rows(&tree), vec![vec!["a"], vec!["b"], vec!["c"]]);
    }

    #[test]
    fn cycle_below_normal_ancestors_does_not_panic() {
        let mut tree = tree_of(&["root", "x", "y"]);
        parent(&mut tree, "root", "x");
        parent(&mut tree, "x", "y");
        parent(&mut tree, "y", "x");
        assert_everyone_placed_once(&tree);
    }

    #[test]
    fn own_parent_does_not_panic() {
        let mut tree = tree_of(&["a"]);
        parent(&mut tree, "a", "a");
        marry(&mut tree, "a", "a");
        assert_eq!(rows(&tree), vec![vec!["a"]]);
    }

    #[test]
    fn spouse_of_own_parent_does_not_panic() {
        let mut tree = tree_of(&["a", "b", "c"]);
        parent(&mut tree, "a", "b");
        marry(&mut tree, "a", "b");
        parent(&mut tree, "b", "c");
        assert_everyone_placed_once(&tree);
        assert_adjacent(&tree, "a", "b");
    }

    #[test]
    fn relationships_to_missing_people_are_ignored() {
        let mut tree = tree_of(&["a"]);
        tree.relationships
            .entry("a".to_string())
            .or_default()
            .push(("ghost".to_string(), Relationship::Parent));
        tree.relationships
            .entry("ghost".to_string())
            .or_default()
            .push(("a".to_string(), Relationship::Spouse));
        assert_eq!(rows(&tree), vec![vec!["a"]]);
    }
}
