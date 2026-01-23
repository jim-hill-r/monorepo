/// Represents a person's sex in a family tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Sex {
    /// Male sex
    Male,
    /// Female sex
    Female,
    /// Unknown or unspecified sex
    Unknown,
}

/// Represents an individual person in a family tree.
///
/// A person has a unique identifier, name, sex, and optional birth/death dates.
/// The dates are stored as optional strings to accommodate various date formats and
/// partial date information (e.g., "1920", "June 1920", "1920-06-15").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Person {
    /// Unique identifier for this person
    pub id: String,
    /// Full name of the person
    pub name: String,
    /// Sex of the person
    pub sex: Sex,
    /// Birth date in string format (e.g., "1920-06-15" or "June 1920")
    pub birth_date: Option<String>,
    /// Death date in string format, None if person is still living or date is unknown
    pub death_date: Option<String>,
}

impl Person {
    /// Creates a new Person with the specified details.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this person
    /// * `name` - Full name of the person
    /// * `sex` - Sex of the person
    /// * `birth_date` - Optional birth date as a string
    /// * `death_date` - Optional death date as a string
    ///
    /// # Example
    /// ```
    /// # use core::{Person, Sex};
    /// #
    /// let person = Person::new(
    ///     "person1".to_string(),
    ///     "John Doe".to_string(),
    ///     Sex::Male,
    ///     Some("1920-06-15".to_string()),
    ///     None,
    /// );
    /// assert_eq!(person.name, "John Doe");
    /// assert_eq!(person.sex, Sex::Male);
    /// ```
    pub fn new(
        id: String,
        name: String,
        sex: Sex,
        birth_date: Option<String>,
        death_date: Option<String>,
    ) -> Self {
        Person {
            id,
            name,
            sex,
            birth_date,
            death_date,
        }
    }

    /// Returns true if the person is deceased (has a death date).
    pub fn is_deceased(&self) -> bool {
        self.death_date.is_some()
    }
}

/// Represents significant events in a person's ancestry or genealogy.
/// These events are used to build and track family trees and relationships.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AncestryEvent {
    /// Birth event with date and location information
    Birth {
        date: Option<String>,
        location: Option<String>,
    },
    /// Death event with date and location information
    Death {
        date: Option<String>,
        location: Option<String>,
    },
    /// Marriage event linking two individuals
    Marriage {
        spouse_id: String,
        date: Option<String>,
        location: Option<String>,
    },
    /// Divorce event ending a marriage
    Divorce {
        spouse_id: String,
        date: Option<String>,
    },
    /// Adoption event establishing a parent-child relationship
    Adoption {
        adoptive_parent_id: String,
        date: Option<String>,
    },
    /// Baptism or christening event
    Baptism {
        date: Option<String>,
        location: Option<String>,
    },
    /// Immigration to a new country
    Immigration {
        from_country: Option<String>,
        to_country: String,
        date: Option<String>,
    },
    /// Emigration from a country
    Emigration {
        from_country: String,
        to_country: Option<String>,
        date: Option<String>,
    },
    /// Burial event
    Burial {
        date: Option<String>,
        location: Option<String>,
    },
    /// Census record
    Census { year: u32, location: String },
    /// Military service event (use separate events for start and end)
    MilitaryService {
        branch: Option<String>,
        date: Option<String>,
    },
    /// Education milestone
    Education {
        institution: String,
        degree: Option<String>,
        date: Option<String>,
    },
    /// Occupation event (use separate events for starting and ending a job)
    Occupation {
        title: String,
        employer: Option<String>,
        date: Option<String>,
    },
    /// Residence event (use separate events for moving in and moving out)
    Residence {
        location: String,
        date: Option<String>,
    },
}

impl AncestryEvent {
    /// Returns a human-readable description of the event type
    pub fn event_type(&self) -> &'static str {
        match self {
            AncestryEvent::Birth { .. } => "Birth",
            AncestryEvent::Death { .. } => "Death",
            AncestryEvent::Marriage { .. } => "Marriage",
            AncestryEvent::Divorce { .. } => "Divorce",
            AncestryEvent::Adoption { .. } => "Adoption",
            AncestryEvent::Baptism { .. } => "Baptism",
            AncestryEvent::Immigration { .. } => "Immigration",
            AncestryEvent::Emigration { .. } => "Emigration",
            AncestryEvent::Burial { .. } => "Burial",
            AncestryEvent::Census { .. } => "Census",
            AncestryEvent::MilitaryService { .. } => "Military Service",
            AncestryEvent::Education { .. } => "Education",
            AncestryEvent::Occupation { .. } => "Occupation",
            AncestryEvent::Residence { .. } => "Residence",
        }
    }

    /// Returns the date associated with the event, if available
    pub fn date(&self) -> Option<&str> {
        match self {
            AncestryEvent::Birth { date, .. }
            | AncestryEvent::Death { date, .. }
            | AncestryEvent::Marriage { date, .. }
            | AncestryEvent::Divorce { date, .. }
            | AncestryEvent::Adoption { date, .. }
            | AncestryEvent::Baptism { date, .. }
            | AncestryEvent::Immigration { date, .. }
            | AncestryEvent::Emigration { date, .. }
            | AncestryEvent::Burial { date, .. }
            | AncestryEvent::Education { date, .. }
            | AncestryEvent::MilitaryService { date, .. }
            | AncestryEvent::Occupation { date, .. }
            | AncestryEvent::Residence { date, .. } => date.as_deref(),
            AncestryEvent::Census { .. } => None, // Census year is stored separately
        }
    }
}

use std::collections::HashMap;

/// Represents different types of relationships between people in a family tree.
///
/// This enum defines the primary relationship types that connect individuals
/// in a genealogical context. Each variant represents a directional relationship
/// from one person to another.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Relationship {
    /// One person is a parent of another person.
    /// This relationship is directional: from parent to child.
    Parent,
    /// One person is a child of another person.
    /// This relationship is directional: from child to parent.
    Child,
    /// Two people are married or in a spousal relationship.
    /// This relationship is typically bidirectional.
    Spouse,
    /// Two people are siblings (share at least one parent).
    /// This relationship is typically bidirectional.
    Sibling,
}

impl Relationship {
    /// Returns a human-readable description of the relationship type.
    pub fn description(&self) -> &'static str {
        match self {
            Relationship::Parent => "Parent",
            Relationship::Child => "Child",
            Relationship::Spouse => "Spouse",
            Relationship::Sibling => "Sibling",
        }
    }

    /// Returns the inverse relationship.
    /// For example, the inverse of Parent is Child, and vice versa.
    /// Spouse and Sibling are their own inverses (symmetric relationships).
    pub fn inverse(&self) -> Relationship {
        match self {
            Relationship::Parent => Relationship::Child,
            Relationship::Child => Relationship::Parent,
            Relationship::Spouse => Relationship::Spouse,
            Relationship::Sibling => Relationship::Sibling,
        }
    }

    /// Returns true if the relationship is symmetric (bidirectional).
    /// Spouse and Sibling relationships are symmetric.
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Relationship::Spouse | Relationship::Sibling)
    }
}

/// Error type for FamilyTree operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyTreeError {
    /// Person already exists in the tree
    PersonAlreadyExists(String),
    /// Person not found in the tree
    PersonNotFound(String),
    /// Invalid relationship (e.g., adding a relationship to a person that doesn't exist)
    InvalidRelationship(String),
}

impl std::fmt::Display for FamilyTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FamilyTreeError::PersonAlreadyExists(id) => {
                write!(
                    f,
                    "Person with id '{}' already exists in the family tree",
                    id
                )
            }
            FamilyTreeError::PersonNotFound(id) => {
                write!(f, "Person with id '{}' not found in the family tree", id)
            }
            FamilyTreeError::InvalidRelationship(msg) => {
                write!(f, "Invalid relationship: {}", msg)
            }
        }
    }
}

impl std::error::Error for FamilyTreeError {}

/// Result type for FamilyTree operations
pub type FamilyTreeResult<T> = Result<T, FamilyTreeError>;

/// Represents a family tree as a graph of people and their relationships.
///
/// The FamilyTree stores people and their relationships in a directed graph structure.
/// Each person is identified by a unique ID, and relationships are directional
/// (e.g., Parent->Child, though some relationships like Spouse can be bidirectional).
///
/// # Example
/// ```
/// # use core::{FamilyTree, Person, Sex, Relationship};
/// #
/// let mut tree = FamilyTree::new();
/// let parent = Person::new(
///     "p1".to_string(),
///     "John Doe".to_string(),
///     Sex::Male,
///     Some("1950-01-01".to_string()),
///     None,
/// );
/// let child = Person::new(
///     "p2".to_string(),
///     "Jane Doe".to_string(),
///     Sex::Female,
///     Some("1980-01-01".to_string()),
///     None,
/// );
///
/// tree.add_person(parent).unwrap();
/// tree.add_person(child).unwrap();
/// tree.add_relationship("p1".to_string(), "p2".to_string(), Relationship::Parent).unwrap();
///
/// let children = tree.get_related_people("p1", &Relationship::Parent);
/// assert_eq!(children.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct FamilyTree {
    /// Map of person ID to Person
    pub people: HashMap<String, Person>,
    /// Map of person ID to a list of (related_person_id, relationship_type) tuples
    pub relationships: HashMap<String, Vec<(String, Relationship)>>,
}

impl FamilyTree {
    /// Creates a new empty FamilyTree.
    pub fn new() -> Self {
        FamilyTree {
            people: HashMap::new(),
            relationships: HashMap::new(),
        }
    }

    /// Adds a person to the family tree.
    ///
    /// # Arguments
    /// * `person` - The Person to add to the tree
    ///
    /// # Errors
    /// Returns `FamilyTreeError::PersonAlreadyExists` if a person with the same ID already exists.
    ///
    /// # Example
    /// ```
    /// # use core::{FamilyTree, Person, Sex};
    /// #
    /// let mut tree = FamilyTree::new();
    /// let person = Person::new(
    ///     "p1".to_string(),
    ///     "John Doe".to_string(),
    ///     Sex::Male,
    ///     Some("1950-01-01".to_string()),
    ///     None,
    /// );
    /// tree.add_person(person).unwrap();
    /// ```
    pub fn add_person(&mut self, person: Person) -> FamilyTreeResult<()> {
        if self.people.contains_key(&person.id) {
            return Err(FamilyTreeError::PersonAlreadyExists(person.id.clone()));
        }
        self.people.insert(person.id.clone(), person);
        Ok(())
    }

    /// Adds a relationship between two people in the family tree.
    ///
    /// # Arguments
    /// * `from_id` - The ID of the person the relationship starts from
    /// * `to_id` - The ID of the person the relationship points to
    /// * `relationship` - The type of relationship
    ///
    /// # Errors
    /// Returns `FamilyTreeError::PersonNotFound` if either person doesn't exist in the tree.
    ///
    /// # Example
    /// ```
    /// # use core::{FamilyTree, Person, Sex, Relationship};
    /// #
    /// let mut tree = FamilyTree::new();
    /// let parent = Person::new("p1".to_string(), "Parent".to_string(), Sex::Male, None, None);
    /// let child = Person::new("p2".to_string(), "Child".to_string(), Sex::Female, None, None);
    /// tree.add_person(parent).unwrap();
    /// tree.add_person(child).unwrap();
    /// tree.add_relationship("p1".to_string(), "p2".to_string(), Relationship::Parent).unwrap();
    /// ```
    pub fn add_relationship(
        &mut self,
        from_id: String,
        to_id: String,
        relationship: Relationship,
    ) -> FamilyTreeResult<()> {
        if !self.people.contains_key(&from_id) {
            return Err(FamilyTreeError::PersonNotFound(from_id));
        }
        if !self.people.contains_key(&to_id) {
            return Err(FamilyTreeError::PersonNotFound(to_id));
        }

        self.relationships
            .entry(from_id)
            .or_default()
            .push((to_id, relationship));

        Ok(())
    }

    /// Gets all people related to a person by a specific relationship type.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person to query relationships for
    /// * `relationship` - The type of relationship to filter by
    ///
    /// # Returns
    /// A vector of references to Person objects that have the specified relationship.
    ///
    /// # Example
    /// ```
    /// # use core::{FamilyTree, Person, Sex, Relationship};
    /// #
    /// let mut tree = FamilyTree::new();
    /// let parent = Person::new("p1".to_string(), "Parent".to_string(), Sex::Male, None, None);
    /// let child = Person::new("p2".to_string(), "Child".to_string(), Sex::Female, None, None);
    /// tree.add_person(parent).unwrap();
    /// tree.add_person(child).unwrap();
    /// tree.add_relationship("p1".to_string(), "p2".to_string(), Relationship::Parent).unwrap();
    /// let children = tree.get_related_people("p1", &Relationship::Parent);
    /// assert_eq!(children.len(), 1);
    /// ```
    pub fn get_related_people(&self, person_id: &str, relationship: &Relationship) -> Vec<&Person> {
        self.relationships
            .get(person_id)
            .map(|rels| {
                rels.iter()
                    .filter(|(_, rel)| rel == relationship)
                    .filter_map(|(id, _)| self.people.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets a person by their ID.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person to retrieve
    ///
    /// # Returns
    /// An Option containing a reference to the Person if found.
    pub fn get_person(&self, person_id: &str) -> Option<&Person> {
        self.people.get(person_id)
    }

    /// Adds a bidirectional spouse relationship between two people.
    ///
    /// # Arguments
    /// * `person1_id` - The ID of the first person
    /// * `person2_id` - The ID of the second person
    ///
    /// # Errors
    /// Returns `FamilyTreeError::PersonNotFound` if either person doesn't exist in the tree.
    pub fn add_spouse_relationship(
        &mut self,
        person1_id: String,
        person2_id: String,
    ) -> FamilyTreeResult<()> {
        self.add_relationship(person1_id.clone(), person2_id.clone(), Relationship::Spouse)?;
        self.add_relationship(person2_id, person1_id, Relationship::Spouse)?;
        Ok(())
    }

    /// Adds a bidirectional parent-child relationship between two people.
    ///
    /// # Arguments
    /// * `parent_id` - The ID of the parent
    /// * `child_id` - The ID of the child
    ///
    /// # Errors
    /// Returns `FamilyTreeError::PersonNotFound` if either person doesn't exist in the tree.
    pub fn add_parent_child_relationship(
        &mut self,
        parent_id: String,
        child_id: String,
    ) -> FamilyTreeResult<()> {
        self.add_relationship(parent_id.clone(), child_id.clone(), Relationship::Parent)?;
        self.add_relationship(child_id, parent_id, Relationship::Child)?;
        Ok(())
    }

    /// Processes an AncestryEvent and updates the family tree accordingly.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person associated with this event
    /// * `event` - The AncestryEvent to process
    ///
    /// # Errors
    /// Returns `FamilyTreeError` if the person doesn't exist or relationships are invalid.
    pub fn process_event(
        &mut self,
        person_id: &str,
        event: &AncestryEvent,
    ) -> FamilyTreeResult<()> {
        match event {
            AncestryEvent::Marriage { spouse_id, .. } => {
                self.add_spouse_relationship(person_id.to_string(), spouse_id.clone())?;
            }
            AncestryEvent::Adoption {
                adoptive_parent_id, ..
            } => {
                self.add_parent_child_relationship(
                    adoptive_parent_id.clone(),
                    person_id.to_string(),
                )?;
            }
            // Other events don't directly create relationships
            _ => {}
        }
        Ok(())
    }

    /// Gets all parents of a person.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person
    ///
    /// # Returns
    /// A vector of references to Person objects that are parents of the specified person.
    pub fn get_parents(&self, person_id: &str) -> Vec<&Person> {
        self.get_related_people(person_id, &Relationship::Child)
    }

    /// Gets all children of a person.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person
    ///
    /// # Returns
    /// A vector of references to Person objects that are children of the specified person.
    pub fn get_children(&self, person_id: &str) -> Vec<&Person> {
        self.get_related_people(person_id, &Relationship::Parent)
    }

    /// Gets all spouses of a person.
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person
    ///
    /// # Returns
    /// A vector of references to Person objects that are spouses of the specified person.
    pub fn get_spouses(&self, person_id: &str) -> Vec<&Person> {
        self.get_related_people(person_id, &Relationship::Spouse)
    }

    /// Gets all siblings of a person (people who share at least one parent).
    ///
    /// # Arguments
    /// * `person_id` - The ID of the person
    ///
    /// # Returns
    /// A vector of references to Person objects that are siblings of the specified person.
    pub fn get_siblings(&self, person_id: &str) -> Vec<&Person> {
        let parents = self.get_parents(person_id);
        let mut siblings = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for parent in parents {
            let children = self.get_children(&parent.id);
            for child in children {
                if child.id != person_id && seen.insert(child.id.clone()) {
                    siblings.push(child);
                }
            }
        }

        siblings
    }
}

impl Default for FamilyTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_birth_event() {
        let event = AncestryEvent::Birth {
            date: Some("1990-01-15".to_string()),
            location: Some("New York, NY".to_string()),
        };
        assert_eq!(event.event_type(), "Birth");
        assert_eq!(event.date(), Some("1990-01-15"));
    }

    #[test]
    fn test_death_event() {
        let event = AncestryEvent::Death {
            date: Some("2050-12-31".to_string()),
            location: Some("Los Angeles, CA".to_string()),
        };
        assert_eq!(event.event_type(), "Death");
        assert_eq!(event.date(), Some("2050-12-31"));
    }

    #[test]
    fn test_marriage_event() {
        let event = AncestryEvent::Marriage {
            spouse_id: "spouse123".to_string(),
            date: Some("2015-06-20".to_string()),
            location: Some("Chicago, IL".to_string()),
        };
        assert_eq!(event.event_type(), "Marriage");
        assert_eq!(event.date(), Some("2015-06-20"));
    }

    #[test]
    fn test_divorce_event() {
        let event = AncestryEvent::Divorce {
            spouse_id: "spouse123".to_string(),
            date: Some("2020-03-10".to_string()),
        };
        assert_eq!(event.event_type(), "Divorce");
        assert_eq!(event.date(), Some("2020-03-10"));
    }

    #[test]
    fn test_adoption_event() {
        let event = AncestryEvent::Adoption {
            adoptive_parent_id: "parent456".to_string(),
            date: Some("2005-08-25".to_string()),
        };
        assert_eq!(event.event_type(), "Adoption");
        assert_eq!(event.date(), Some("2005-08-25"));
    }

    #[test]
    fn test_baptism_event() {
        let event = AncestryEvent::Baptism {
            date: Some("1990-03-01".to_string()),
            location: Some("St. Mary's Church".to_string()),
        };
        assert_eq!(event.event_type(), "Baptism");
    }

    #[test]
    fn test_immigration_event() {
        let event = AncestryEvent::Immigration {
            from_country: Some("Ireland".to_string()),
            to_country: "United States".to_string(),
            date: Some("1920-05-15".to_string()),
        };
        assert_eq!(event.event_type(), "Immigration");
        assert_eq!(event.date(), Some("1920-05-15"));
    }

    #[test]
    fn test_emigration_event() {
        let event = AncestryEvent::Emigration {
            from_country: "Germany".to_string(),
            to_country: Some("Brazil".to_string()),
            date: Some("1935-11-20".to_string()),
        };
        assert_eq!(event.event_type(), "Emigration");
    }

    #[test]
    fn test_burial_event() {
        let event = AncestryEvent::Burial {
            date: Some("2050-01-05".to_string()),
            location: Some("Oak Hill Cemetery".to_string()),
        };
        assert_eq!(event.event_type(), "Burial");
    }

    #[test]
    fn test_census_event() {
        let event = AncestryEvent::Census {
            year: 1920,
            location: "Brooklyn, New York".to_string(),
        };
        assert_eq!(event.event_type(), "Census");
        assert_eq!(event.date(), None); // Census year is stored separately, not as a date string
    }

    #[test]
    fn test_military_service_event() {
        let event = AncestryEvent::MilitaryService {
            branch: Some("Army".to_string()),
            date: Some("1942-01-01".to_string()),
        };
        assert_eq!(event.event_type(), "Military Service");
        assert_eq!(event.date(), Some("1942-01-01"));
    }

    #[test]
    fn test_education_event() {
        let event = AncestryEvent::Education {
            institution: "Harvard University".to_string(),
            degree: Some("Bachelor of Arts".to_string()),
            date: Some("2012-05-15".to_string()),
        };
        assert_eq!(event.event_type(), "Education");
        assert_eq!(event.date(), Some("2012-05-15"));
    }

    #[test]
    fn test_occupation_event() {
        let event = AncestryEvent::Occupation {
            title: "Software Engineer".to_string(),
            employer: Some("Tech Corp".to_string()),
            date: Some("2015-01-01".to_string()),
        };
        assert_eq!(event.event_type(), "Occupation");
        assert_eq!(event.date(), Some("2015-01-01"));
    }

    #[test]
    fn test_residence_event() {
        let event = AncestryEvent::Residence {
            location: "123 Main St, Boston, MA".to_string(),
            date: Some("2010-06-01".to_string()),
        };
        assert_eq!(event.event_type(), "Residence");
        assert_eq!(event.date(), Some("2010-06-01"));
    }

    #[test]
    fn test_event_with_no_date() {
        let event = AncestryEvent::Birth {
            date: None,
            location: Some("Unknown".to_string()),
        };
        assert_eq!(event.date(), None);
    }

    #[test]
    fn test_event_equality() {
        let event1 = AncestryEvent::Birth {
            date: Some("1990-01-15".to_string()),
            location: Some("New York".to_string()),
        };
        let event2 = AncestryEvent::Birth {
            date: Some("1990-01-15".to_string()),
            location: Some("New York".to_string()),
        };
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_event_clone() {
        let event = AncestryEvent::Marriage {
            spouse_id: "spouse123".to_string(),
            date: Some("2015-06-20".to_string()),
            location: Some("Chicago, IL".to_string()),
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }

    // Person struct tests
    #[test]
    fn test_person_new() {
        let person = Person::new(
            "person1".to_string(),
            "John Doe".to_string(),
            Sex::Male,
            Some("1920-06-15".to_string()),
            None,
        );
        assert_eq!(person.id, "person1");
        assert_eq!(person.name, "John Doe");
        assert_eq!(person.sex, Sex::Male);
        assert_eq!(person.birth_date, Some("1920-06-15".to_string()));
        assert_eq!(person.death_date, None);
    }

    #[test]
    fn test_person_with_death_date() {
        let person = Person::new(
            "person2".to_string(),
            "Jane Smith".to_string(),
            Sex::Female,
            Some("1925-03-10".to_string()),
            Some("2010-12-25".to_string()),
        );
        assert_eq!(person.id, "person2");
        assert_eq!(person.name, "Jane Smith");
        assert_eq!(person.sex, Sex::Female);
        assert_eq!(person.birth_date, Some("1925-03-10".to_string()));
        assert_eq!(person.death_date, Some("2010-12-25".to_string()));
        assert!(person.is_deceased());
    }

    #[test]
    fn test_person_is_deceased() {
        let living = Person::new(
            "person3".to_string(),
            "Bob Johnson".to_string(),
            Sex::Male,
            Some("1990-01-01".to_string()),
            None,
        );
        assert!(!living.is_deceased());

        let deceased = Person::new(
            "person4".to_string(),
            "Alice Brown".to_string(),
            Sex::Female,
            Some("1900-01-01".to_string()),
            Some("1980-01-01".to_string()),
        );
        assert!(deceased.is_deceased());
    }

    #[test]
    fn test_person_with_unknown_sex() {
        let person = Person::new(
            "person5".to_string(),
            "Pat Wilson".to_string(),
            Sex::Unknown,
            Some("1995-05-15".to_string()),
            None,
        );
        assert_eq!(person.sex, Sex::Unknown);
    }

    #[test]
    fn test_person_with_no_dates() {
        let person = Person::new(
            "person6".to_string(),
            "Unknown Ancestor".to_string(),
            Sex::Unknown,
            None,
            None,
        );
        assert_eq!(person.birth_date, None);
        assert_eq!(person.death_date, None);
        assert!(!person.is_deceased());
    }

    #[test]
    fn test_person_equality() {
        let person1 = Person::new(
            "person7".to_string(),
            "Test Person".to_string(),
            Sex::Male,
            Some("1950-01-01".to_string()),
            None,
        );
        let person2 = Person::new(
            "person7".to_string(),
            "Test Person".to_string(),
            Sex::Male,
            Some("1950-01-01".to_string()),
            None,
        );
        assert_eq!(person1, person2);
    }

    #[test]
    fn test_person_clone() {
        let person = Person::new(
            "person8".to_string(),
            "Clone Test".to_string(),
            Sex::Female,
            Some("1960-06-15".to_string()),
            Some("2020-03-20".to_string()),
        );
        let cloned = person.clone();
        assert_eq!(person, cloned);
    }

    #[test]
    fn test_person_with_partial_date() {
        // Test that we can store partial date information
        let person = Person::new(
            "person9".to_string(),
            "Historical Figure".to_string(),
            Sex::Male,
            Some("circa 1800".to_string()),
            Some("1875".to_string()),
        );
        assert_eq!(person.birth_date, Some("circa 1800".to_string()));
        assert_eq!(person.death_date, Some("1875".to_string()));
    }

    // Sex tests
    #[test]
    fn test_sex_equality() {
        assert_eq!(Sex::Male, Sex::Male);
        assert_eq!(Sex::Female, Sex::Female);
        assert_eq!(Sex::Unknown, Sex::Unknown);
        assert_ne!(Sex::Male, Sex::Female);
    }

    #[test]
    fn test_sex_clone() {
        let sex = Sex::Male;
        let cloned = sex.clone();
        assert_eq!(sex, cloned);
    }

    // Relationship tests
    #[test]
    fn test_relationship_description() {
        assert_eq!(Relationship::Parent.description(), "Parent");
        assert_eq!(Relationship::Child.description(), "Child");
        assert_eq!(Relationship::Spouse.description(), "Spouse");
        assert_eq!(Relationship::Sibling.description(), "Sibling");
    }

    #[test]
    fn test_relationship_inverse() {
        assert_eq!(Relationship::Parent.inverse(), Relationship::Child);
        assert_eq!(Relationship::Child.inverse(), Relationship::Parent);
        assert_eq!(Relationship::Spouse.inverse(), Relationship::Spouse);
        assert_eq!(Relationship::Sibling.inverse(), Relationship::Sibling);
    }

    #[test]
    fn test_relationship_is_symmetric() {
        assert!(!Relationship::Parent.is_symmetric());
        assert!(!Relationship::Child.is_symmetric());
        assert!(Relationship::Spouse.is_symmetric());
        assert!(Relationship::Sibling.is_symmetric());
    }

    #[test]
    fn test_relationship_equality() {
        assert_eq!(Relationship::Parent, Relationship::Parent);
        assert_eq!(Relationship::Child, Relationship::Child);
        assert_eq!(Relationship::Spouse, Relationship::Spouse);
        assert_eq!(Relationship::Sibling, Relationship::Sibling);
        assert_ne!(Relationship::Parent, Relationship::Child);
        assert_ne!(Relationship::Spouse, Relationship::Sibling);
    }

    #[test]
    fn test_relationship_clone() {
        let rel = Relationship::Parent;
        let cloned = rel.clone();
        assert_eq!(rel, cloned);
    }

    #[test]
    fn test_relationship_debug() {
        // Ensure Debug trait is working
        let rel = Relationship::Spouse;
        let debug_str = format!("{:?}", rel);
        assert!(debug_str.contains("Spouse"));
    }

    // FamilyTree tests
    #[test]
    fn test_family_tree_new() {
        let tree = FamilyTree::new();
        assert_eq!(tree.people.len(), 0);
        assert_eq!(tree.relationships.len(), 0);
    }

    #[test]
    fn test_add_person() {
        let mut tree = FamilyTree::new();
        let person = Person::new(
            "person1".to_string(),
            "John Doe".to_string(),
            Sex::Male,
            Some("1920-06-15".to_string()),
            None,
        );

        tree.add_person(person.clone()).unwrap();
        assert_eq!(tree.people.len(), 1);
        assert_eq!(tree.people.get("person1").unwrap(), &person);
    }

    #[test]
    fn test_add_duplicate_person() {
        let mut tree = FamilyTree::new();
        let person = Person::new(
            "person1".to_string(),
            "John Doe".to_string(),
            Sex::Male,
            Some("1920-06-15".to_string()),
            None,
        );

        tree.add_person(person.clone()).unwrap();
        let result = tree.add_person(person);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_relationship() {
        let mut tree = FamilyTree::new();
        let parent = Person::new(
            "parent1".to_string(),
            "Parent".to_string(),
            Sex::Male,
            Some("1900-01-01".to_string()),
            None,
        );
        let child = Person::new(
            "child1".to_string(),
            "Child".to_string(),
            Sex::Female,
            Some("1930-01-01".to_string()),
            None,
        );

        tree.add_person(parent).unwrap();
        tree.add_person(child).unwrap();
        tree.add_relationship(
            "parent1".to_string(),
            "child1".to_string(),
            Relationship::Parent,
        )
        .unwrap();

        assert_eq!(tree.relationships.len(), 1);
    }

    #[test]
    fn test_get_related_people() {
        let mut tree = FamilyTree::new();
        let parent = Person::new(
            "parent1".to_string(),
            "Parent".to_string(),
            Sex::Male,
            Some("1900-01-01".to_string()),
            None,
        );
        let child1 = Person::new(
            "child1".to_string(),
            "Child 1".to_string(),
            Sex::Female,
            Some("1930-01-01".to_string()),
            None,
        );
        let child2 = Person::new(
            "child2".to_string(),
            "Child 2".to_string(),
            Sex::Male,
            Some("1932-01-01".to_string()),
            None,
        );

        tree.add_person(parent).unwrap();
        tree.add_person(child1).unwrap();
        tree.add_person(child2).unwrap();
        tree.add_relationship(
            "parent1".to_string(),
            "child1".to_string(),
            Relationship::Parent,
        )
        .unwrap();
        tree.add_relationship(
            "parent1".to_string(),
            "child2".to_string(),
            Relationship::Parent,
        )
        .unwrap();

        let children = tree.get_related_people("parent1", &Relationship::Parent);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_add_spouse_relationship() {
        let mut tree = FamilyTree::new();
        let spouse1 = Person::new(
            "spouse1".to_string(),
            "Spouse 1".to_string(),
            Sex::Male,
            Some("1900-01-01".to_string()),
            None,
        );
        let spouse2 = Person::new(
            "spouse2".to_string(),
            "Spouse 2".to_string(),
            Sex::Female,
            Some("1902-01-01".to_string()),
            None,
        );

        tree.add_person(spouse1).unwrap();
        tree.add_person(spouse2).unwrap();
        tree.add_spouse_relationship("spouse1".to_string(), "spouse2".to_string())
            .unwrap();

        // Spouse relationships should be bidirectional
        let spouses1 = tree.get_related_people("spouse1", &Relationship::Spouse);
        assert_eq!(spouses1.len(), 1);
        let spouses2 = tree.get_related_people("spouse2", &Relationship::Spouse);
        assert_eq!(spouses2.len(), 1);
    }

    #[test]
    fn test_add_parent_child_relationship() {
        let mut tree = FamilyTree::new();
        let parent = Person::new(
            "parent1".to_string(),
            "Parent".to_string(),
            Sex::Male,
            Some("1900-01-01".to_string()),
            None,
        );
        let child = Person::new(
            "child1".to_string(),
            "Child".to_string(),
            Sex::Female,
            Some("1930-01-01".to_string()),
            None,
        );

        tree.add_person(parent).unwrap();
        tree.add_person(child).unwrap();
        tree.add_parent_child_relationship("parent1".to_string(), "child1".to_string())
            .unwrap();

        // Should have bidirectional relationship
        let children = tree.get_related_people("parent1", &Relationship::Parent);
        assert_eq!(children.len(), 1);
        let parents = tree.get_related_people("child1", &Relationship::Child);
        assert_eq!(parents.len(), 1);
    }

    #[test]
    fn test_process_marriage_event() {
        let mut tree = FamilyTree::new();
        let person1 = Person::new(
            "person1".to_string(),
            "Person 1".to_string(),
            Sex::Male,
            Some("1900-01-01".to_string()),
            None,
        );
        let person2 = Person::new(
            "person2".to_string(),
            "Person 2".to_string(),
            Sex::Female,
            Some("1902-01-01".to_string()),
            None,
        );

        tree.add_person(person1).unwrap();
        tree.add_person(person2).unwrap();

        let event = AncestryEvent::Marriage {
            spouse_id: "person2".to_string(),
            date: Some("1925-06-15".to_string()),
            location: Some("New York".to_string()),
        };

        tree.process_event("person1", &event).unwrap();

        let spouses = tree.get_related_people("person1", &Relationship::Spouse);
        assert_eq!(spouses.len(), 1);
    }

    #[test]
    fn test_get_parents() {
        let mut tree = FamilyTree::new();
        let parent1 = Person::new(
            "p1".to_string(),
            "Parent 1".to_string(),
            Sex::Male,
            None,
            None,
        );
        let parent2 = Person::new(
            "p2".to_string(),
            "Parent 2".to_string(),
            Sex::Female,
            None,
            None,
        );
        let child = Person::new("c1".to_string(), "Child".to_string(), Sex::Male, None, None);

        tree.add_person(parent1).unwrap();
        tree.add_person(parent2).unwrap();
        tree.add_person(child).unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c1".to_string())
            .unwrap();
        tree.add_parent_child_relationship("p2".to_string(), "c1".to_string())
            .unwrap();

        let parents = tree.get_parents("c1");
        assert_eq!(parents.len(), 2);
    }

    #[test]
    fn test_get_children() {
        let mut tree = FamilyTree::new();
        let parent = Person::new(
            "p1".to_string(),
            "Parent".to_string(),
            Sex::Male,
            None,
            None,
        );
        let child1 = Person::new(
            "c1".to_string(),
            "Child 1".to_string(),
            Sex::Male,
            None,
            None,
        );
        let child2 = Person::new(
            "c2".to_string(),
            "Child 2".to_string(),
            Sex::Female,
            None,
            None,
        );

        tree.add_person(parent).unwrap();
        tree.add_person(child1).unwrap();
        tree.add_person(child2).unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c1".to_string())
            .unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c2".to_string())
            .unwrap();

        let children = tree.get_children("p1");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_get_spouses() {
        let mut tree = FamilyTree::new();
        let person = Person::new(
            "p1".to_string(),
            "Person".to_string(),
            Sex::Male,
            None,
            None,
        );
        let spouse1 = Person::new(
            "s1".to_string(),
            "Spouse 1".to_string(),
            Sex::Female,
            None,
            None,
        );
        let spouse2 = Person::new(
            "s2".to_string(),
            "Spouse 2".to_string(),
            Sex::Female,
            None,
            None,
        );

        tree.add_person(person).unwrap();
        tree.add_person(spouse1).unwrap();
        tree.add_person(spouse2).unwrap();
        tree.add_spouse_relationship("p1".to_string(), "s1".to_string())
            .unwrap();
        tree.add_spouse_relationship("p1".to_string(), "s2".to_string())
            .unwrap();

        let spouses = tree.get_spouses("p1");
        assert_eq!(spouses.len(), 2);
    }

    #[test]
    fn test_get_siblings() {
        let mut tree = FamilyTree::new();
        let parent = Person::new(
            "p1".to_string(),
            "Parent".to_string(),
            Sex::Male,
            None,
            None,
        );
        let child1 = Person::new(
            "c1".to_string(),
            "Child 1".to_string(),
            Sex::Male,
            None,
            None,
        );
        let child2 = Person::new(
            "c2".to_string(),
            "Child 2".to_string(),
            Sex::Female,
            None,
            None,
        );
        let child3 = Person::new(
            "c3".to_string(),
            "Child 3".to_string(),
            Sex::Male,
            None,
            None,
        );

        tree.add_person(parent).unwrap();
        tree.add_person(child1).unwrap();
        tree.add_person(child2).unwrap();
        tree.add_person(child3).unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c1".to_string())
            .unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c2".to_string())
            .unwrap();
        tree.add_parent_child_relationship("p1".to_string(), "c3".to_string())
            .unwrap();

        let siblings = tree.get_siblings("c1");
        assert_eq!(siblings.len(), 2);
        assert!(siblings.iter().any(|p| p.id == "c2"));
        assert!(siblings.iter().any(|p| p.id == "c3"));
    }
}
