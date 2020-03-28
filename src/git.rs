use std::path::Path;
use std::process::{Command, Output};

use chrono::{DateTime, Utc};

use crate::Args;

const GIT_FAILED_MESSAGE: &str = "Failed to run git";

fn get_global_user_config<S: AsRef<str>>(name: S) -> String {
  let Output { status, stdout, .. } = Command::new("git")
    .args(&["config", "--global", &format!("user.{}", name.as_ref())])
    .output()
    .expect(GIT_FAILED_MESSAGE);

  if !status.success() {
    panic!(format!(
      r#"Failed to get git config "user.{}". If you don't have global git config set, try the --git-user-name and --git-user-email flags."#,
      name.as_ref()
    ));
  }

  String::from_utf8_lossy(&stdout).to_string()
}

fn set_user_config<S: AsRef<str>, P: AsRef<Path>>(dir: P, name: S, value: S) {
  let name = name.as_ref();
  let value = value.as_ref();
  let Output { status, .. } = Command::new("git")
    .args(&["config", &format!("user.{}", name), &value])
    .current_dir(dir)
    .output()
    .expect(GIT_FAILED_MESSAGE);

  if !status.success() {
    panic!(format!(
      "Failed to set git user.{} in new repository: {}",
      name, value
    ));
  }
}

pub fn init(args: &Args) {
  if args.destination.exists() {
    panic!("The path {} already exists!", args.destination.display());
  }

  let Output { status, .. } = Command::new("git")
    .args(&["init", &format!("{}", args.destination.display())])
    .output()
    .expect(GIT_FAILED_MESSAGE);

  if !status.success() {
    panic!(format!(
      "Failed to initialise repository at: {}",
      args.destination.display()
    ));
  }

  // Set "user.name" and "user.email" in the new repository.
  set_user_config(
    &args.destination,
    "name",
    &args
      .git_user_name
      .clone()
      .unwrap_or_else(|| get_global_user_config("name")),
  );
  set_user_config(
    &args.destination,
    "email",
    &args
      .git_user_email
      .clone()
      .unwrap_or_else(|| get_global_user_config("email")),
  );
}

pub fn add_file<P: AsRef<Path>>(dir: P, path: P) {
  let path = path.as_ref();
  let relative_to_git_root_path = path.strip_prefix(&dir).unwrap();
  let output = Command::new("git")
    .args(&["add", &format!("{}", relative_to_git_root_path.display())])
    .current_dir(&dir)
    .output()
    .expect(GIT_FAILED_MESSAGE);

  if !output.status.success() {
    panic!(format!(
      "Failed to stage file: {}.\n{:#?}",
      path.display(),
      output
    ));
  }
}

pub fn commit<P: AsRef<Path>, S: AsRef<str>>(dir: P, message: S, date: DateTime<Utc>) {
  let output = Command::new("git")
    .args(&["commit", "-m", message.as_ref()])
    .current_dir(dir)
    .env("GIT_AUTHOR_DATE", date.to_rfc2822())
    .env("GIT_COMMITTER_DATE", date.to_rfc2822())
    .output()
    .expect(GIT_FAILED_MESSAGE);

  if !output.status.success() {
    panic!(format!("Failed to make commit: {}.\n{:#?}", date, output));
  }
}
