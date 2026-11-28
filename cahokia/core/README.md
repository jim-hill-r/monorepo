# core

The data model and layout maths behind the Cahokia genealogy app: people,
relationships, life events and the family tree graph, plus the layout,
viewport and lineage logic that the `ui` crate draws on web, desktop and
mobile. It has no dependencies and no UI code.

## Features

- **Person Management**: Define individuals with basic biographical information including name, sex, birth/death dates
- **Relationship Tracking**: Model family relationships (Parent, Child, Spouse, Sibling)
- **Ancestry Events**: Track significant life events (Birth, Death, Marriage, Divorce, Adoption, etc.)
- **Family Tree**: Build and query a graph-based family tree structure
- **Layout**: Arrange a family tree into generation rows, then into boxes and lines with coordinates for drawing
- **Pan, zoom and focus**: `Viewport` maps layout coordinates to the screen (pan, zoom around a point, fit, centre on a person), and `FamilyTree::lineage()` gives a focus person's ancestors and descendants and which lines join them, for highlighting
- **Sample family**: `sample::sample_family_tree()` returns a made-up three-generation family of ten people, which the apps show until a user enters their own

## Usage

### Creating a Family Tree

```rust
use core::{FamilyTree, Person, Sex, Relationship};

let mut tree = FamilyTree::new();

// Add people to the tree
let parent = Person::new(
    "p1".to_string(),
    "John Doe".to_string(),
    Sex::Male,
    Some("1950-01-01".to_string()),
    None,
);
let child = Person::new(
    "p2".to_string(),
    "Jane Doe".to_string(),
    Sex::Female,
    Some("1980-01-01".to_string()),
    None,
);

tree.add_person(parent).unwrap();
tree.add_person(child).unwrap();

// Establish relationships
tree.add_parent_child_relationship("p1".to_string(), "p2".to_string()).unwrap();

// Query the tree
let children = tree.get_children("p1");
assert_eq!(children.len(), 1);
```

### Processing Ancestry Events

```rust
use core::{FamilyTree, Person, Sex, AncestryEvent};

let mut tree = FamilyTree::new();

// Add people
let person1 = Person::new("p1".to_string(), "John".to_string(), Sex::Male, None, None);
let person2 = Person::new("p2".to_string(), "Mary".to_string(), Sex::Female, None, None);

tree.add_person(person1).unwrap();
tree.add_person(person2).unwrap();

// Process a marriage event
let marriage_event = AncestryEvent::Marriage {
    spouse_id: "p2".to_string(),
    date: Some("1975-06-15".to_string()),
    location: Some("New York".to_string()),
};

tree.process_event("p1", &marriage_event).unwrap();

// Query spouses
let spouses = tree.get_spouses("p1");
assert_eq!(spouses.len(), 1);
```

### Querying Relationships

The FamilyTree provides several helper methods for querying relationships:

- `get_parents(person_id)` - Get all parents of a person
- `get_children(person_id)` - Get all children of a person
- `get_spouses(person_id)` - Get all spouses of a person
- `get_siblings(person_id)` - Get all siblings of a person (people who share at least one parent)

### Laying Out the Tree for Drawing

`FamilyTree::generation_layout()` places everyone in a row (their generation,
parents above children) and at a position within that row, so renderers never
walk relationships themselves:

```rust
use core::{FamilyTree, Person, Placement, Sex};

let mut tree = FamilyTree::new();
for id in ["dad", "mom", "kid"] {
    tree.add_person(Person::new(id.to_string(), id.to_string(), Sex::Unknown, None, None)).unwrap();
}
tree.add_spouse_relationship("dad".to_string(), "mom".to_string()).unwrap();
tree.add_parent_child_relationship("dad".to_string(), "kid".to_string()).unwrap();

let layout = tree.generation_layout();
assert_eq!(layout.rows()[0], vec!["dad", "mom"]);
assert_eq!(layout.placement("kid"), Some(Placement { generation: 1, position: 0 }));
```

Spouses sit next to each other in the same row, children follow the order of
their parents and then birth date, and the result is the same on every run.
Disconnected people get their own place, and relationship cycles in bad data
are broken instead of panicking. See the `layout` module docs for the full
rules.

`FamilyTree::tree_layout()` turns those rows into what a renderer draws: a box
with coordinates for every person and a line for every parent-child and spouse
link, with its route already worked out. Parent-child lines are routed per
family: a couple's children hang from the middle of their spouse line, and a
single parent's from the bottom of their box. Each family gets its own bar in
the gap above the children, and families whose bars would overlap get
different bar heights, so neighbouring families never merge into one line:

```rust
use core::{EdgeKind, FamilyTree, Person, Point, Sex};

let mut tree = FamilyTree::new();
for id in ["dad", "kid"] {
    tree.add_person(Person::new(id.to_string(), id.to_string(), Sex::Unknown, None, None)).unwrap();
}
tree.add_parent_child_relationship("dad".to_string(), "kid".to_string()).unwrap();

let layout = tree.tree_layout();
for node in layout.nodes() {
    // Draw a node.width x node.height box at (node.x, node.y).
}
let edge = &layout.edges()[0];
assert_eq!(edge.kind, EdgeKind::ParentChild);
assert_eq!(edge.start, Point { x: 90.0, y: 60.0 }); // bottom middle of dad's box
assert_eq!(edge.end, Point { x: 90.0, y: 140.0 }); // top middle of kid's box
assert_eq!(edge.route, vec![edge.start, edge.end]); // corners to draw through
```

Coordinates use SVG's convention (origin top left, `y` down), and
`layout.width()` and `layout.height()` give the area to draw into. Pass a
`Spacing` to `tree_layout_with()` to change the box size and gaps. See the
`tree_layout` module docs for the full rules.

## Data Structures

### Person

Represents an individual with:
- Unique identifier
- Name
- Sex (Male, Female, Unknown)
- Optional birth date
- Optional death date

### Relationship

Defines relationship types:
- Parent
- Child
- Spouse
- Sibling

### AncestryEvent

Tracks life events:
- Birth, Death
- Marriage, Divorce
- Adoption
- Baptism
- Immigration, Emigration
- Burial
- Census
- Military Service
- Education
- Occupation
- Residence

### FamilyTree

A graph-based structure that:
- Stores people by ID
- Tracks directional relationships between people
- Validates data integrity (prevents duplicate persons, invalid relationships)
- Provides query methods for traversing the tree

## Error Handling

The crate uses `FamilyTreeResult<T>` which can return:
- `FamilyTreeError::PersonAlreadyExists` - Attempting to add a duplicate person
- `FamilyTreeError::PersonNotFound` - Referencing a non-existent person
- `FamilyTreeError::InvalidRelationship` - Invalid relationship operation

## Future Enhancements

See the issue files in `docs/issues/` for planned features including:
- Divorce event handling
- Family tree visualization
