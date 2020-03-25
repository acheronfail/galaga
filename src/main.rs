use std::env;
use std::fs::{remove_file, OpenOptions};
use std::io::Write;
use std::process::{Command, Output};

use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc, Weekday};
use human_id::id;
use transpose::transpose;

const TEMP_FILE_PATH: &str = "temp";
const PATTERN_WIDTH: usize = 100;
const PATTERN_HEIGHT: usize = 7;
const PATTERN: &str = "
XXXXXX XXXXXX
XXX XX XX XXX
XXX X   X XXX
X X   X   X X
X    X X    X
X  XX   XX  X
X XXXX XXXX X
";

fn git_add_file(path: String) {
    let Output { status, .. } = Command::new("git")
        .args(&["add", &path])
        .output()
        .expect("Failed to run git command");

    if !status.success() {
        panic!(format!("Failed to stage file: {}", path));
    }
}

fn git_commit(message: String, date: DateTime<Utc>) {
    let Output { status, .. } = Command::new("git")
        .args(&["commit", "-a", "-m", &message])
        .env("GIT_AUTHOR_DATE", date.to_rfc2822())
        .env("GIT_COMMITTER_DATE", date.to_rfc2822())
        .output()
        .expect("Failed to run git command");

    if !status.success() {
        panic!(format!("Failed to make commit for {}", date));
    }
}

fn write_to_temp(content: String) {
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(TEMP_FILE_PATH)
        .expect("Failed to open temp file")
        .write_fmt(format_args!("{}", content))
        .expect("Failed to write tmp file");
}

fn do_work(current_date: DateTime<Utc>) {
    let start_of_day = current_date
        .date()
        .and_time(NaiveTime::from_hms(0, 0, 0))
        .expect("...");

    let mut current_datetime = start_of_day.clone();
    while current_datetime < (start_of_day + Duration::days(1)) {
        let date_string = &current_datetime.to_rfc2822();
        let content = format!("{}: {}", date_string, id("-", false));

        println!("{}", content);
        write_to_temp(content.clone());
        git_commit(format!("lol: {}", content), current_datetime);

        current_datetime = current_datetime + Duration::hours(1);
    }
}

fn main() -> Result<(), String> {
    // Check CLI.
    let is_dry_run = env::args().any(|a| a == "--dry");

    // Prepare the mask.
    let mask_pre: Vec<bool> = PATTERN
        .trim()
        .chars()
        .filter_map(|c| match c {
            'X' => Some(true),
            ' ' => Some(false),
            _ => None,
        })
        .collect();

    let mut mask = vec![false; mask_pre.len()];
    transpose(
        &mask_pre,
        &mut mask,
        mask_pre.len() / PATTERN_HEIGHT,
        PATTERN_HEIGHT,
    );

    if mask.len() % PATTERN_HEIGHT != 0 {
        return Err(format!(
            "Pattern length ({}) must be divisible by {}",
            mask.len(),
            PATTERN_HEIGHT
        ));
    }

    // Get start and end dates. End date is the day this is run, start date is
    // the first Sunday before the start of the pattern.
    let end_date = Utc::now();
    let mut start_date = end_date - Duration::days((PATTERN_WIDTH * PATTERN_HEIGHT) as i64);
    while start_date.weekday() != Weekday::Sun {
        start_date = start_date - Duration::days(1);
    }

    // Ensure our mask is long enough for our number of days.
    let num_days = (end_date - start_date).num_days() as usize;
    mask = mask.iter().cycle().take(num_days).map(|r| *r).collect();

    // Create and stage temp file.
    if !is_dry_run {
        write_to_temp(String::from("..."));
        git_add_file(String::from(TEMP_FILE_PATH));
    }

    let mut current_date = start_date.clone();
    while current_date < end_date {
        let index = ((current_date - start_date).num_days()) as usize;
        match (is_dry_run, mask[index]) {
            // Dry run.
            (true, true) => print!("X"),
            (true, false) => print!(" "),

            // Real thing.
            (false, true) => do_work(current_date),
            (false, false) => (),
        }

        if is_dry_run && (index + 1) % PATTERN_HEIGHT == 0 {
            println!("");
        }

        current_date = current_date + Duration::days(1);
    }

    if !is_dry_run {
        remove_file("temp").expect("failed to remove file");
        git_commit(String::from("lol: complete!"), end_date);
    }

    Ok(())
}
