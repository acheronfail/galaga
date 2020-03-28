use std::fs::remove_file;
use std::path::Path;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use clap::Clap;
use human_id::id;

mod cli;
mod fs;
mod git;
mod pattern;

use cli::Args;
use fs::write_string_to_file;
use git::{add_file, commit, init};
use pattern::prepare_mask;

const PATTERN_HEIGHT: usize = 7;

fn do_work<P: AsRef<Path>>(args: &Args, temp_file: P, current_date: DateTime<Utc>) {
    let temp_file = temp_file.as_ref().to_path_buf();
    let start_of_day = current_date
        .date()
        .and_time(NaiveTime::from_hms(0, 0, 0))
        .expect("...");

    let mut current_datetime = start_of_day.clone();
    while current_datetime < (start_of_day + Duration::days(1)) {
        let date_string = &current_datetime.to_rfc2822();
        let commit_msg = format!("{}: {}", date_string, id("-", false));

        println!("{}", commit_msg);
        write_string_to_file(&temp_file, commit_msg.clone()).expect("Failed to write temp file");
        add_file(&args.destination, &temp_file);
        commit(
            &args.destination,
            format!("lol: {}", commit_msg),
            current_datetime,
        );

        current_datetime = current_datetime + Duration::hours(1);
    }
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    // Parse end date or default to today.
    let end_date = match args.end_date {
        Some(ref s) => Utc.from_utc_datetime(
            &NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .expect(&format!("Failed to parse end date: {}", s))
                .and_hms(0, 0, 0),
        ),
        None => {
            let today = Utc::today();
            Utc.from_utc_datetime(
                &NaiveDate::from_ymd(today.year(), today.month(), today.day()).and_hms(0, 0, 0),
            )
        }
    };

    // The start date is the first Sunday before the start of the pattern.
    let mut start_date = end_date - Duration::days((args.pattern_width * PATTERN_HEIGHT) as i64);
    while start_date.weekday() != Weekday::Sun {
        start_date = start_date - Duration::days(1);
    }

    // Prepare the mask.
    let num_days = (end_date - start_date).num_days() as usize;
    let mask = prepare_mask(&args, num_days)?;

    // Prepare repository.
    init(&args);

    // Prepare file which will be used in all commits.
    let temp_file = args.destination.join("temp").to_path_buf();
    write_string_to_file(&temp_file, String::from("...")).expect("Failed to write temp file");
    add_file(&args.destination, &temp_file);
    commit(&args.destination, "lol: start your engines!", start_date);

    let mut current_date = start_date.clone();
    while current_date < end_date {
        let index = ((current_date - start_date).num_days()) as usize;
        match (args.dry_run, mask[index]) {
            // Dry run.
            (true, true) => print!("X"),
            (true, false) => print!(" "),

            // Real thing.
            (false, true) => do_work(&args, &temp_file, current_date),
            (false, false) => (),
        }

        if args.dry_run && (index + 1) % PATTERN_HEIGHT == 0 {
            println!("");
        }

        current_date = current_date + Duration::days(1);
    }

    remove_file(&temp_file).expect("Failed to remove temp file");
    commit(&args.destination, "lol: complete!", end_date);

    Ok(())
}
