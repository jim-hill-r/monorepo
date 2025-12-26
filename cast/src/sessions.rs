use chrono::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
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
        write!(f, "{},{}{}", self.timestamp, self.kind, postfix)
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

#[derive(Debug)]
enum SessionEntryKind {
    Start,
    Pause,
    Stop,
}

impl fmt::Display for SessionEntryKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionEntryKind::Start => write!(f, "Start"),
            SessionEntryKind::Pause => write!(f, "Pause"),
            SessionEntryKind::Stop => write!(f, "Stop"),
        }
    }
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

    fs::write(session_path, format!("{}\n", entry))?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum PauseSessionError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("no active session found")]
    NoActiveSession,
}

pub fn pause(working_directory: impl AsRef<Path>) -> Result<(), PauseSessionError> {
    let sessions_directory = working_directory.as_ref().join(SESSIONS_DIRECTORY);
    let latest_session =
        find_latest_session(&sessions_directory).ok_or(PauseSessionError::NoActiveSession)?;

    let entry = SessionEntry {
        session_id: latest_session.session_id,
        timestamp: Utc::now(),
        kind: SessionEntryKind::Pause,
        name: latest_session.name,
    };

    let session_path = sessions_directory.join(entry.file_name());
    let mut file = OpenOptions::new().append(true).open(session_path)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum StopSessionError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("no active session found")]
    NoActiveSession,
}

pub fn stop(working_directory: impl AsRef<Path>) -> Result<(), StopSessionError> {
    let sessions_directory = working_directory.as_ref().join(SESSIONS_DIRECTORY);
    let latest_session =
        find_latest_session(&sessions_directory).ok_or(StopSessionError::NoActiveSession)?;

    let entry = SessionEntry {
        session_id: latest_session.session_id,
        timestamp: Utc::now(),
        kind: SessionEntryKind::Stop,
        name: latest_session.name,
    };

    let session_path = sessions_directory.join(entry.file_name());
    let mut file = OpenOptions::new().append(true).open(session_path)?;
    writeln!(file, "{}", entry)?;
    Ok(())
}

/// Find the most recent session by UUID v7 timestamp
fn find_latest_session(sessions_directory: &Path) -> Option<SessionEntry> {
    if !sessions_directory.exists() {
        return None;
    }

    let entries = fs::read_dir(sessions_directory).ok()?;
    let mut latest: Option<(Uuid, PathBuf)> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }

        let filename = match path.file_stem().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };

        // Extract UUID and optional name from filename
        // Format is either "uuid.log" or "uuid-name.log" where name doesn't contain dashes from the UUID
        // UUID v7 format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx (8-4-4-4-12 hex digits)
        // We need to extract the first 36 characters as the UUID
        let uuid_str = if filename.len() >= 36 {
            &filename[..36]
        } else {
            filename
        };

        if let Ok(uuid) = Uuid::parse_str(uuid_str) {
            if let Some((latest_uuid, _)) = latest {
                // UUID v7 embeds timestamp, so we can compare directly
                if uuid > latest_uuid {
                    latest = Some((uuid, path.clone()));
                }
            } else {
                latest = Some((uuid, path.clone()));
            }
        }
    }

    if let Some((uuid, path)) = latest {
        // Extract the name from the filename if present
        let filename = path.file_stem()?.to_str()?;
        // Name starts after the UUID and the separator dash
        let name = if filename.len() > 37 {
            // 36 chars for UUID + 1 char for dash
            Some(filename[37..].to_string())
        } else {
            None
        };

        Some(SessionEntry {
            session_id: uuid,
            timestamp: Utc::now(), // This will be overwritten when creating new entry
            kind: SessionEntryKind::Start, // Placeholder, actual kind doesn't matter for finding session
            name,
        })
    } else {
        None
    }
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

    #[test]
    fn session_entry_kind_start_displays_correctly() {
        let kind = SessionEntryKind::Start;
        assert_eq!(kind.to_string(), "Start");
    }

    #[test]
    fn session_entry_kind_pause_displays_correctly() {
        let kind = SessionEntryKind::Pause;
        assert_eq!(kind.to_string(), "Pause");
    }

    #[test]
    fn session_entry_kind_stop_displays_correctly() {
        let kind = SessionEntryKind::Stop;
        assert_eq!(kind.to_string(), "Stop");
    }

    #[test]
    fn pause_session_appends_to_log() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();

        // Start a session first
        start(tmp_dir.path(), None).unwrap();

        // Pause the session
        pause(tmp_dir.path()).unwrap();

        // Verify the session file exists and contains both Start and Pause entries
        let sessions_dir = tmp_dir.path().join(SESSIONS_DIRECTORY);
        let entries: Vec<_> = fs::read_dir(sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 1, "Expected exactly one session file");

        let session_file = &entries[0].path();
        let content = fs::read_to_string(session_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2, "Expected two lines (Start and Pause)");
        assert!(
            lines[0].contains("Start"),
            "First line should contain Start"
        );
        assert!(
            lines[1].contains("Pause"),
            "Second line should contain Pause"
        );
    }

    #[test]
    fn stop_session_appends_to_log() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();

        // Start a session first
        start(tmp_dir.path(), None).unwrap();

        // Stop the session
        stop(tmp_dir.path()).unwrap();

        // Verify the session file exists and contains both Start and Stop entries
        let sessions_dir = tmp_dir.path().join(SESSIONS_DIRECTORY);
        let entries: Vec<_> = fs::read_dir(sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 1, "Expected exactly one session file");

        let session_file = &entries[0].path();
        let content = fs::read_to_string(session_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2, "Expected two lines (Start and Stop)");
        assert!(
            lines[0].contains("Start"),
            "First line should contain Start"
        );
        assert!(lines[1].contains("Stop"), "Second line should contain Stop");
    }

    #[test]
    fn pause_without_active_session_returns_error() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();

        // Try to pause without starting a session
        let result = pause(tmp_dir.path());
        assert!(
            result.is_err(),
            "Expected error when pausing without active session"
        );

        if let Err(e) = result {
            assert_eq!(e.to_string(), "no active session found");
        }
    }

    #[test]
    fn stop_without_active_session_returns_error() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();

        // Try to stop without starting a session
        let result = stop(tmp_dir.path());
        assert!(
            result.is_err(),
            "Expected error when stopping without active session"
        );

        if let Err(e) = result {
            assert_eq!(e.to_string(), "no active session found");
        }
    }

    #[test]
    fn pause_and_stop_with_named_session() {
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();
        let session_name = "test-session";

        // Start a named session
        start(
            tmp_dir.path(),
            Some(SessionStartOptions {
                name: Some(session_name.to_string()),
            }),
        )
        .unwrap();

        // Pause the session
        pause(tmp_dir.path()).unwrap();

        // Stop the session
        stop(tmp_dir.path()).unwrap();

        // Verify the session file exists with the correct name
        let sessions_dir = tmp_dir.path().join(SESSIONS_DIRECTORY);
        let entries: Vec<_> = fs::read_dir(sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 1, "Expected exactly one session file");

        let session_file = &entries[0].path();
        let filename = session_file.file_name().unwrap().to_str().unwrap();
        assert!(
            filename.contains(session_name),
            "Filename should contain session name"
        );

        let content = fs::read_to_string(session_file).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 3, "Expected three lines (Start, Pause, Stop)");
        assert!(
            lines[0].contains("Start"),
            "First line should contain Start"
        );
        assert!(
            lines[0].contains(session_name),
            "First line should contain session name"
        );
        assert!(
            lines[1].contains("Pause"),
            "Second line should contain Pause"
        );
        assert!(
            lines[1].contains(session_name),
            "Second line should contain session name"
        );
        assert!(lines[2].contains("Stop"), "Third line should contain Stop");
        assert!(
            lines[2].contains(session_name),
            "Third line should contain session name"
        );
    }

    #[test]
    fn multiple_sessions_pause_affects_latest() {
        use std::thread;
        use std::time::Duration;
        use tempdir::TempDir;

        let tmp_dir = TempDir::new("test").unwrap();

        // Start first session
        start(
            tmp_dir.path(),
            Some(SessionStartOptions {
                name: Some("first".to_string()),
            }),
        )
        .unwrap();

        // Wait a bit to ensure different UUIDs
        thread::sleep(Duration::from_millis(10));

        // Start second session
        start(
            tmp_dir.path(),
            Some(SessionStartOptions {
                name: Some("second".to_string()),
            }),
        )
        .unwrap();

        // Pause should affect the latest (second) session
        pause(tmp_dir.path()).unwrap();

        // Verify two session files exist
        let sessions_dir = tmp_dir.path().join(SESSIONS_DIRECTORY);
        let entries: Vec<_> = fs::read_dir(&sessions_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(entries.len(), 2, "Expected two session files");

        // Find the second session file (should contain "second")
        let second_session_file = entries
            .iter()
            .find(|e| e.file_name().to_str().unwrap().contains("second"))
            .expect("Should find second session file");

        let content = fs::read_to_string(second_session_file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 2, "Second session should have Start and Pause");
        assert!(
            lines[1].contains("Pause"),
            "Second session should have Pause entry"
        );
    }
}
