use std::path::PathBuf;

use clap::{crate_authors, crate_version};
use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), setting = AppSettings::ColoredHelp)]
pub struct Args {
    /// The destination of the new git directory.
    pub destination: PathBuf,

    /// The end date of the pattern. Defaults to the current date.
    /// Should be given in the format: "YYYY-MM-DD".
    #[clap(short = "e", long = "end")]
    pub end_date: Option<String>,

    /// The template itself.
    #[clap(short = "t", long = "template")]
    pub template: Option<String>,

    /// Path to file containing the template.
    #[clap(short = "f", long = "file")]
    pub template_file: Option<PathBuf>,

    /// How many times the template should repeat.
    #[clap(short = "r", long = "repeat", default_value = "1")]
    pub template_repeat: usize,

    /// Sets "user.name" in the newly created repository.
    #[clap(short = "N", long = "git-user-name")]
    pub git_user_name: Option<String>,

    /// Sets "user.email" in the newly created repository.
    #[clap(short = "E", long = "git-user-email")]
    pub git_user_email: Option<String>,

    /// Whether or not to perform a dry run. This won't create a new repository,
    /// it will just run log out the generated pattern.
    #[clap(short = "d", long = "dry-run")]
    pub dry_run: bool,
}
