use std::path::PathBuf;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K.")]
pub struct Args {
  /// The end date of the pattern. Defaults to the current date.
  /// Should be given in the format: "YYYY-MM-DD".
  pub end_date: Option<String>,

  /// The pattern itself.
  #[clap(short = "p", long = "pattern")]
  pub pattern: Option<String>,

  /// Path to file containing the pattern.
  #[clap(short = "f", long = "file")]
  pub pattern_file: Option<PathBuf>,

  /// Width (in weeks, which map to vertical columns) of the pattern.
  #[clap(short = "w", long = "width", default_value = "104")]
  pub pattern_width: usize,

  /// Whether or not to perform a dry run (don't actually make any commits).
  /// TODO: just make a new repository? that way it's non destructive?
  #[clap(short = "d", long = "dry-run")]
  pub dry_run: bool,
}
