use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use crate::Args;

/// A wrapper around `std::process::Command` which simplifies running git commands.
#[derive(Debug)]
struct GitCommand<'a> {
    cmd: Command,
    args: &'a [&'a str],
}

impl<'a> GitCommand<'a> {
    /// Create a new git command with arguments.
    pub fn new(args: &'a [&'a str]) -> GitCommand<'a> {
        GitCommand {
            cmd: Command::new("git"),
            args,
        }
    }

    /// Configures the command to run in this directory.
    pub fn dir(&mut self, dir: impl AsRef<Path>) -> &mut Self {
        self.cmd.current_dir(dir);
        self
    }

    /// Adds the following environment override for the command.
    pub fn env<K: AsRef<OsStr>, V: AsRef<OsStr>>(&mut self, key: K, value: V) -> &mut Self {
        self.cmd.env(key, value);
        self
    }

    /// Runs the git command and returns the output.
    pub fn run(&mut self) -> Result<String> {
        let output = self.cmd.args(self.args).output().unwrap();

        if !output.status.success() {
            let mut message = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if message.is_empty() {
                message = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }

            Err(anyhow!(
                "the command 'git {}' failed with code: {}\n\n{}",
                self.args.join(" "),
                output.status.code().unwrap(),
                message,
            ))
        } else {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
    }

    pub fn set_config<S: AsRef<str>, P: AsRef<Path>>(dir: P, name: S, value: S) {
        GitCommand::new(&["config", name.as_ref(), value.as_ref()])
            .dir(dir.as_ref())
            .run()
            .unwrap();
    }

    fn get_global_config<S: AsRef<str>>(name: S) -> String {
        GitCommand::new(&["config", "--global", name.as_ref()])
            .run()
            .unwrap()
    }
}

/// Initialises a git repository. Panics if the destination already exists.
/// Handles git configured with `user.useConfigOnly` by setting the `user.name`
/// and `user.value` config fields in the new repository.
pub fn init(args: &Args) -> Result<()> {
    if args.destination.exists() {
        panic!("The path {} already exists!", args.destination.display());
    }

    GitCommand::new(&["init", &format!("{}", args.destination.display())]).run()?;

    // Set "user.name" and "user.email" in the new repository.
    GitCommand::set_config(
        &args.destination,
        "user.name",
        &args
            .git_user_name
            .clone()
            .unwrap_or_else(|| GitCommand::get_global_config("user.name")),
    );
    GitCommand::set_config(
        &args.destination,
        "user.email",
        &args
            .git_user_email
            .clone()
            .unwrap_or_else(|| GitCommand::get_global_config("user.email")),
    );

    Ok(())
}

/// Runs `git add <path>` in the destination directory.
pub fn add_file<P: AsRef<Path>>(dir: P, path: P) -> Result<()> {
    let path = path.as_ref();
    let relative_to_git_root_path = path.strip_prefix(&dir).unwrap();
    GitCommand::new(&["add", &format!("{}", relative_to_git_root_path.display())])
        .dir(&dir)
        .run()?;

    Ok(())
}

/// Runs `git commit -m <path>` in the destination directory.
pub fn commit<P: AsRef<Path>, S: AsRef<str>>(
    dir: P,
    message: S,
    date: DateTime<Utc>,
) -> Result<()> {
    GitCommand::new(&["commit", "-m", message.as_ref()])
        .dir(dir)
        .env("GIT_AUTHOR_DATE", date.to_rfc2822())
        .env("GIT_COMMITTER_DATE", date.to_rfc2822())
        .run()?;

    Ok(())
}
