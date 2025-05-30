use std::path::Path;
use std::{fs, io};

pub fn new(
    _working_directory: impl AsRef<Path>,
    _name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Call copy_dir_all on templates/base
    // TODO: Call copy_dir_all on templates/library (overwrite files)
    // TODO: Delete any unnecessary .gitignores
    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
