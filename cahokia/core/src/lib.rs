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
    /// Military service
    MilitaryService {
        branch: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    },
    /// Education milestone
    Education {
        institution: String,
        degree: Option<String>,
        date: Option<String>,
    },
    /// Occupation change or record
    Occupation {
        title: String,
        employer: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    },
    /// Residence change or record
    Residence {
        location: String,
        start_date: Option<String>,
        end_date: Option<String>,
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
            | AncestryEvent::Education { date, .. } => date.as_deref(),
            AncestryEvent::Census { .. } => None, // Census year is stored separately
            AncestryEvent::MilitaryService { start_date, .. }
            | AncestryEvent::Occupation { start_date, .. }
            | AncestryEvent::Residence { start_date, .. } => start_date.as_deref(),
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
            start_date: Some("1942-01-01".to_string()),
            end_date: Some("1945-12-31".to_string()),
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
            start_date: Some("2015-01-01".to_string()),
            end_date: Some("2020-12-31".to_string()),
        };
        assert_eq!(event.event_type(), "Occupation");
        assert_eq!(event.date(), Some("2015-01-01"));
    }

    #[test]
    fn test_residence_event() {
        let event = AncestryEvent::Residence {
            location: "123 Main St, Boston, MA".to_string(),
            start_date: Some("2010-06-01".to_string()),
            end_date: Some("2015-08-31".to_string()),
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
}
