/// Represents a person's gender in a family tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Gender {
    /// Male gender
    Male,
    /// Female gender
    Female,
    /// Unknown or unspecified gender
    Unknown,
}

/// Represents an individual person in a family tree.
///
/// A person has a unique identifier, name, gender, and optional birth/death dates.
/// The dates are stored as optional strings to accommodate various date formats and
/// partial date information (e.g., "1920", "June 1920", "1920-06-15").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Person {
    /// Unique identifier for this person
    pub id: String,
    /// Full name of the person
    pub name: String,
    /// Gender of the person
    pub gender: Gender,
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
    /// * `gender` - Gender of the person
    /// * `birth_date` - Optional birth date as a string
    /// * `death_date` - Optional death date as a string
    ///
    /// # Example
    /// ```
    /// use core::{Person, Gender};
    ///
    /// let person = Person::new(
    ///     "person1".to_string(),
    ///     "John Doe".to_string(),
    ///     Gender::Male,
    ///     Some("1920-06-15".to_string()),
    ///     None,
    /// );
    /// assert_eq!(person.name, "John Doe");
    /// assert_eq!(person.gender, Gender::Male);
    /// ```
    pub fn new(
        id: String,
        name: String,
        gender: Gender,
        birth_date: Option<String>,
        death_date: Option<String>,
    ) -> Self {
        Person {
            id,
            name,
            gender,
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
            Gender::Male,
            Some("1920-06-15".to_string()),
            None,
        );
        assert_eq!(person.id, "person1");
        assert_eq!(person.name, "John Doe");
        assert_eq!(person.gender, Gender::Male);
        assert_eq!(person.birth_date, Some("1920-06-15".to_string()));
        assert_eq!(person.death_date, None);
    }

    #[test]
    fn test_person_with_death_date() {
        let person = Person::new(
            "person2".to_string(),
            "Jane Smith".to_string(),
            Gender::Female,
            Some("1925-03-10".to_string()),
            Some("2010-12-25".to_string()),
        );
        assert_eq!(person.id, "person2");
        assert_eq!(person.name, "Jane Smith");
        assert_eq!(person.gender, Gender::Female);
        assert_eq!(person.birth_date, Some("1925-03-10".to_string()));
        assert_eq!(person.death_date, Some("2010-12-25".to_string()));
        assert!(person.is_deceased());
    }

    #[test]
    fn test_person_is_deceased() {
        let living = Person::new(
            "person3".to_string(),
            "Bob Johnson".to_string(),
            Gender::Male,
            Some("1990-01-01".to_string()),
            None,
        );
        assert!(!living.is_deceased());

        let deceased = Person::new(
            "person4".to_string(),
            "Alice Brown".to_string(),
            Gender::Female,
            Some("1900-01-01".to_string()),
            Some("1980-01-01".to_string()),
        );
        assert!(deceased.is_deceased());
    }

    #[test]
    fn test_person_with_unknown_gender() {
        let person = Person::new(
            "person5".to_string(),
            "Pat Wilson".to_string(),
            Gender::Unknown,
            Some("1995-05-15".to_string()),
            None,
        );
        assert_eq!(person.gender, Gender::Unknown);
    }

    #[test]
    fn test_person_with_no_dates() {
        let person = Person::new(
            "person6".to_string(),
            "Unknown Ancestor".to_string(),
            Gender::Unknown,
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
            Gender::Male,
            Some("1950-01-01".to_string()),
            None,
        );
        let person2 = Person::new(
            "person7".to_string(),
            "Test Person".to_string(),
            Gender::Male,
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
            Gender::Female,
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
            Gender::Male,
            Some("circa 1800".to_string()),
            Some("1875".to_string()),
        );
        assert_eq!(person.birth_date, Some("circa 1800".to_string()));
        assert_eq!(person.death_date, Some("1875".to_string()));
    }

    // Gender tests
    #[test]
    fn test_gender_equality() {
        assert_eq!(Gender::Male, Gender::Male);
        assert_eq!(Gender::Female, Gender::Female);
        assert_eq!(Gender::Unknown, Gender::Unknown);
        assert_ne!(Gender::Male, Gender::Female);
    }

    #[test]
    fn test_gender_clone() {
        let gender = Gender::Male;
        let cloned = gender.clone();
        assert_eq!(gender, cloned);
    }
}
