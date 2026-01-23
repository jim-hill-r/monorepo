# Cahokia Core

This crate provides core data structures and functionality for managing genealogy and family tree data in the Cahokia application.

## Features

- **Person Management**: Define individuals with basic biographical information including name, sex, birth/death dates
- **Relationship Tracking**: Model family relationships (Parent, Child, Spouse, Sibling)
- **Ancestry Events**: Track significant life events (Birth, Death, Marriage, Divorce, Adoption, etc.)
- **Family Tree**: Build and query a graph-based family tree structure

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

See `ISSUES.md` for planned features including:
- Divorce event handling
- Ancestor/descendant traversal queries
- Family tree visualization
