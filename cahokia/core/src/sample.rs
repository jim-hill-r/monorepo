//! A small made-up family used to show the family tree before a user has
//! entered their own.
//!
//! Every app (web, desktop and mobile) shows the same sample, so it lives
//! here rather than in one of them.

use crate::{FamilyTree, FamilyTreeResult, Person, Sex};

/// One sample person: id, name, sex, birth year, death year.
type SamplePerson = (
    &'static str,
    &'static str,
    Sex,
    Option<&'static str>,
    Option<&'static str>,
);

/// The sample people.
const PEOPLE: &[SamplePerson] = &[
    (
        "thomas",
        "Thomas Reed",
        Sex::Male,
        Some("1890"),
        Some("1962"),
    ),
    (
        "clara",
        "Clara Reed",
        Sex::Female,
        Some("1894"),
        Some("1971"),
    ),
    ("henry", "Henry Reed", Sex::Male, Some("1918"), Some("1990")),
    (
        "alice",
        "Alice Moore",
        Sex::Female,
        Some("1921"),
        Some("2005"),
    ),
    (
        "ruth",
        "Ruth Price",
        Sex::Female,
        Some("1920"),
        Some("2001"),
    ),
    (
        "samuel",
        "Samuel Price",
        Sex::Male,
        Some("1915"),
        Some("1988"),
    ),
    ("daniel", "Daniel Reed", Sex::Male, Some("1946"), None),
    ("margaret", "Margaret Reed", Sex::Female, Some("1949"), None),
    ("eleanor", "Eleanor Price", Sex::Female, Some("1950"), None),
    ("sam", "Sam Price", Sex::Unknown, None, None),
];

/// The sample couples.
const SPOUSES: &[(&str, &str)] = &[("thomas", "clara"), ("henry", "alice"), ("samuel", "ruth")];

/// The sample parent and child pairs.
const PARENT_CHILD: &[(&str, &str)] = &[
    ("thomas", "henry"),
    ("clara", "henry"),
    ("thomas", "ruth"),
    ("clara", "ruth"),
    ("henry", "daniel"),
    ("alice", "daniel"),
    ("henry", "margaret"),
    ("alice", "margaret"),
    ("samuel", "eleanor"),
    ("ruth", "eleanor"),
    ("samuel", "sam"),
    ("ruth", "sam"),
];

/// Returns a made-up three-generation family of ten people: a couple, their
/// two married children, and four grandchildren across two families.
///
/// # Example
/// ```
/// let tree = core::sample::sample_family_tree();
/// assert_eq!(tree.people.len(), 10);
/// assert_eq!(tree.get_children("thomas").len(), 2);
/// ```
pub fn sample_family_tree() -> FamilyTree {
    // The data above is fixed and valid, so building it cannot fail; an
    // empty tree is a safe fallback all the same.
    build().unwrap_or_default()
}

fn build() -> FamilyTreeResult<FamilyTree> {
    let mut tree = FamilyTree::new();
    for (id, name, sex, birth, death) in PEOPLE {
        tree.add_person(Person::new(
            (*id).to_string(),
            (*name).to_string(),
            sex.clone(),
            birth.map(str::to_string),
            death.map(str::to_string),
        ))?;
    }
    for (a, b) in SPOUSES {
        tree.add_spouse_relationship((*a).to_string(), (*b).to_string())?;
    }
    for (parent, child) in PARENT_CHILD {
        tree.add_parent_child_relationship((*parent).to_string(), (*child).to_string())?;
    }
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EdgeKind;

    #[test]
    fn builds_without_errors() {
        assert!(build().is_ok());
    }

    #[test]
    fn has_every_sample_person() {
        let tree = sample_family_tree();
        assert_eq!(tree.people.len(), PEOPLE.len());
        for (id, name, ..) in PEOPLE {
            assert_eq!(tree.get_person(id).map(|p| p.name.as_str()), Some(*name));
        }
    }

    #[test]
    fn spans_three_generations() {
        let tree = sample_family_tree();
        assert_eq!(tree.generation_layout().rows().len(), 3);
        assert_eq!(tree.get_descendants("thomas").len(), 6);
        assert!(tree.get_parents("thomas").is_empty());
    }

    #[test]
    fn layout_has_a_line_per_relationship() {
        let layout = sample_family_tree().tree_layout();
        let count = |kind: EdgeKind| layout.edges().iter().filter(|e| e.kind == kind).count();
        assert_eq!(layout.nodes().len(), PEOPLE.len());
        assert_eq!(count(EdgeKind::Spouse), SPOUSES.len());
        assert_eq!(count(EdgeKind::ParentChild), PARENT_CHILD.len());
    }

    #[test]
    fn includes_a_person_with_no_dates() {
        let tree = sample_family_tree();
        let sam = tree.get_person("sam");
        assert!(sam.is_some_and(|p| p.birth_date.is_none() && p.death_date.is_none()));
    }
}
