use std::cmp;
use std::fs::remove_file;
use std::io::{self, Write};
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
use pattern::Pattern;

const PATTERN_HEIGHT: usize = 7;
const DATE_FORMAT_YMD: &str = "%Y-%m-%d";

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

        write_string_to_file(&temp_file, commit_msg.clone()).expect("Failed to write temp file");
        git::add_file(&args.destination, &temp_file);
        git::commit(
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
    let mut end_date = match args.end_date {
        Some(ref s) => Utc.from_utc_datetime(
            &NaiveDate::parse_from_str(s, DATE_FORMAT_YMD)
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

    let pattern = Pattern::new(&args);

    // The start date is the first Sunday before the start of the pattern.
    let mut start_date = end_date - Duration::days((pattern.width() * PATTERN_HEIGHT) as i64);

    // Shift the start date and end date so it starts on a Sunday.
    while start_date.weekday() != Weekday::Sun {
        let one_day = Duration::days(1);
        start_date = start_date - one_day;
        end_date = end_date - one_day;
    }

    // Prepare the mask.
    let num_days = (end_date - start_date).num_days() as usize;
    let mask = pattern.mask();

    // Prepare repository and create the file which will be used in all commits.
    let temp_file = args.destination.join("temp").to_path_buf();
    if !args.dry_run {
        git::init(&args);
        write_string_to_file(&temp_file, String::from("...")).expect("Failed to write temp file");
        git::add_file(&args.destination, &temp_file);
        git::commit(&args.destination, "lol: start your engines!", start_date);
    }

    println!("Start Date:      {}", start_date.format(DATE_FORMAT_YMD));
    println!("End Date:        {}", end_date.format(DATE_FORMAT_YMD));
    println!("Number of days:  {}", num_days);
    println!("Number of weeks: {}", num_days / PATTERN_HEIGHT);
    println!("Destination:     {}", args.destination.display());
    println!("Using template:\n\n{}\n", pattern.template());
    println!("Final pattern:\n\n{}\n", pattern);

    let mut current_date = start_date.clone();
    let mut max_progress_length = 0;
    while current_date < end_date {
        let index = ((current_date - start_date).num_days()) as usize;
        match (args.dry_run, mask[index]) {
            (false, true) => {
                let progress = format!(
                    "\rCurrent Date: {} (days left: {})          ",
                    current_date.format(DATE_FORMAT_YMD),
                    num_days - index
                );

                max_progress_length = cmp::max(max_progress_length, progress.len());
                print!("{}", progress);
                io::stdout().flush().unwrap();

                do_work(&args, &temp_file, current_date)
            }

            // Dry run or an empty grid.
            _ => (),
        }

        current_date = current_date + Duration::days(1);
    }

    if !args.dry_run {
        remove_file(&temp_file).expect("Failed to remove temp file");
        git::add_file(&args.destination, &temp_file);
        git::commit(&args.destination, "lol: complete!", end_date);
    }

    println!("\r{}", " ".repeat(max_progress_length));
    println!("Done!");

    Ok(())
}
