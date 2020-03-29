use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: String) -> io::Result<()> {
  OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(path)?
    .write_fmt(format_args!("{}", content))
}
