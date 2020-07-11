use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn write_str_to_file<P: AsRef<Path>>(path: P, content: impl AsRef<str>) -> io::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?
        .write_all(content.as_ref().as_bytes())?;

    Ok(())
}
