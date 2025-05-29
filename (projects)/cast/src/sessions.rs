use chrono::prelude::*;
use std::fs::{create_dir_all, write};
use std::path::Path;
use uuid::Uuid;

const SESSIONS_DIRECTORY: &str = ".cast/sessions";
const SESSION_START_KEY: &str = "start";

pub fn start(cast_directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let sessions_directory = cast_directory.join(SESSIONS_DIRECTORY);
    create_dir_all(&sessions_directory)?;

    let id = Uuid::now_v7();
    let data = format!("{},{}", Utc::now().to_string(), SESSION_START_KEY);
    write(sessions_directory.join(id.to_string()), data)?;
    Ok(())
}

pub fn pause(_cast_directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub fn stop(_cast_directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
