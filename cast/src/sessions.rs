use chrono::prelude::*;
use std::fmt;
use std::path::Path;
use std::{fs, io};
use thiserror::Error;
use uuid::Uuid;

const SESSIONS_DIRECTORY: &str = ".cast/sessions";

pub struct SessionStartOptions {
    pub(crate) name: Option<String>,
}

struct SessionEntry {
    session_id: Uuid,
    timestamp: DateTime<Utc>,
    kind: SessionEntryKind,
    name: Option<String>,
}

impl fmt::Display for SessionEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let postfix = if let Some(name) = &self.name {
            format!(",{}", name)
        } else {
            String::new()
        };
        write!(f, "{},{:?}{}", self.timestamp, self.kind, postfix)
    }
}

impl SessionEntry {
    fn file_name(&self) -> String {
        let postfix = if let Some(name) = &self.name {
            format!("-{}", name)
        } else {
            String::new()
        };
        format!("{}{}.log", self.session_id, postfix)
    }
}

#[derive(Debug)] // TODO: Properly implement display trait
#[allow(dead_code)] // Pause and Stop variants are not yet implemented
enum SessionEntryKind {
    Start,
    Pause,
    Stop,
}

#[derive(Error, Debug)]
pub enum StartSessionError {
    #[error("io error")]
    Io(#[from] io::Error),
}

pub fn start(
    working_directory: impl AsRef<Path>,
    options: Option<SessionStartOptions>,
) -> Result<(), StartSessionError> {
    let sessions_directory = working_directory.as_ref().join(SESSIONS_DIRECTORY);
    fs::create_dir_all(&sessions_directory)?;

    let entry = SessionEntry {
        session_id: Uuid::now_v7(),
        timestamp: Utc::now(),
        kind: SessionEntryKind::Start,
        name: options.and_then(|v| v.name),
    };
    let session_path = sessions_directory.join(entry.file_name());

    fs::write(session_path, entry.to_string())?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum PauseSessionError {
    #[error("io error")]
    Io(#[from] io::Error),
}
pub fn pause(_working_directory: impl AsRef<Path>) -> Result<(), PauseSessionError> {
    // TODO: Add pause implementation
    Ok(())
}

#[derive(Error, Debug)]
pub enum StopSessionError {
    #[error("io error")]
    Io(#[from] io::Error),
}
pub fn stop(_working_directory: impl AsRef<Path>) -> Result<(), StopSessionError> {
    // TODO: Add stop implementation
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::uuid;

    #[test]
    fn entry_has_correct_default_filename() {
        const TEST_UUID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let entry = SessionEntry {
            session_id: uuid!(TEST_UUID),
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            kind: SessionEntryKind::Start,
            name: None,
        };
        assert_eq!(entry.file_name(), format!("{}.log", TEST_UUID))
    }

    #[test]
    fn entry_has_correct_named_filename() {
        const TEST_NAME: &str = "entry_has_correct_named_filename";
        const TEST_UUID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let entry = SessionEntry {
            session_id: uuid!(TEST_UUID),
            timestamp: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            kind: SessionEntryKind::Start,
            name: Some(TEST_NAME.into()),
        };
        assert_eq!(
            entry.file_name(),
            format!("{}-{}.log", TEST_UUID, TEST_NAME)
        )
    }

    #[test]
    fn entry_has_correct_default_to_string() {
        const TEST_UUID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let test_timestamp: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let entry = SessionEntry {
            session_id: uuid!(TEST_UUID),
            timestamp: test_timestamp,
            kind: SessionEntryKind::Start,
            name: None,
        };
        assert_eq!(
            entry.to_string(),
            "2025-01-01 12:00:00 UTC,Start".to_string()
        )
    }

    #[test]
    fn entry_has_correct_named_to_string() {
        const TEST_NAME: &str = "entry_has_correct_named_to_string";
        const TEST_UUID: &str = "67e55044-10b1-426f-9247-bb680e5fe0c8";
        let test_timestamp: DateTime<Utc> = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let entry = SessionEntry {
            session_id: uuid!(TEST_UUID),
            timestamp: test_timestamp,
            kind: SessionEntryKind::Start,
            name: Some(TEST_NAME.into()),
        };
        assert_eq!(
            entry.to_string(),
            format!("2025-01-01 12:00:00 UTC,Start,{}", TEST_NAME)
        )
    }
}
