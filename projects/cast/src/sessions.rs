use chrono::prelude::*;
use std::fs::{create_dir_all, write};
use std::path::Path;
use uuid::Uuid;

const SESSIONS_DIRECTORY: &str = ".cast/sessions";
const SESSION_START_KEY: &str = "start";

pub fn start(working_directory: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let sessions_directory = working_directory.as_ref().join(SESSIONS_DIRECTORY);
    create_dir_all(&sessions_directory)?;

    let id = Uuid::now_v7();
    let data = format!("{},{}", Utc::now().to_string(), SESSION_START_KEY);
    write(sessions_directory.join(id.to_string()), data)?;
    Ok(())
}

pub fn pause(_working_directory: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Add pause implementation
    Ok(())
}

pub fn stop(_working_directory: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Add stop implementation
    Ok(())
}

// TODO: Add tests
