//! Functionality for the command-line interface.
use anyhow::{anyhow, Result};
use chrono::{
    DateTime, Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc, Weekday,
};
use clap::{Parser, Subcommand, ValueEnum};

use crate::{database, tasks, CurrentTask, DataFiles, Task};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Debug, Copy)]
pub enum ViewFilter {
    Current,
    All,
    Day,
    Week,
    Month,
    Year,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a new task now, ending and saving any currently running tasks
    Start {
        /// Project name for the task
        project_name: String,
        /// Optional start time, if not given then current time is used
        #[clap(short, long)]
        start_time: Option<String>,
        /// Optional task description
        #[clap(short, long)]
        description: Option<String>,
        /// Overwrite current task instead of ending it and starting a new one
        #[clap(short, long)]
        overwrite: bool,
    },
    /// Save and end the current task
    End {
        /// Optional end time, if not given current time is used
        end_time: Option<String>,
        /// End the current task and discard it (do not save it)
        #[clap(short, long)]
        discard: bool,
    },
    /// Add a task with given start and end time
    Add {
        /// Project name for the task
        project_name: String,
        /// Date and time task started
        start_time: String,
        /// Date and time task ended
        end_time: String,
        /// Optional task description
        #[clap(short, long)]
        description: Option<String>,
    },
    /// View current task or a group of tasks based on filtering the task start time
    View {
        /// Shortcut timescale filter, relative to today, for tasks to view
        #[clap(value_enum)]
        filter: Option<ViewFilter>,
        /// Start date / time to get tasks from, if given filter is ignored
        #[clap(short, long)]
        from: Option<String>,
        /// End date / time to get tasks before, if given filter is ignored
        #[clap(short, long)]
        to: Option<String>,
    },
    // TODO Add edit command
}

pub fn start(
    files: &DataFiles,
    project_name: &str,
    start_time: &Option<String>,
    description: &Option<String>,
    overwrite: &bool,
) -> Result<()> {
    if !*overwrite {
        // End current task before starting a new one
        match tasks::end_current_task(files.current_file(), files.database_file(), None, false)? {
            Some(t) => println!("Ended task: {}", t),
            None => (),
        }
    }

    let start_time = match &start_time {
        Some(st) => Some(parse_local_datetime(st)?),
        None => None,
    };

    let t = tasks::start_task(
        project_name,
        start_time,
        description.as_ref(),
        files.current_file(),
    )?;
    println!("Started task: {}", t);

    Ok(())
}

pub fn end(files: &DataFiles, end_time: &Option<String>, discard: &bool) -> Result<()> {
    let end_time = match &end_time {
        Some(et) => Some(parse_local_datetime(et)?),
        None => None,
    };

    match tasks::end_current_task(
        files.current_file(),
        files.database_file(),
        end_time,
        *discard,
    )? {
        Some(t) => println!("Ended task: {}", t),
        None => println!("No current task to end"),
    };

    Ok(())
}

/// Parse datetime string which doesn't include timezone, use local timezone.
///
/// If date isn't given then today is used, if time isn't given then 00:00:00
/// is used.
fn parse_local_datetime(text: &str) -> Result<DateTime<Utc>> {
    // Attempt to parse datetime and fallback on parsing only date or time
    let datetime = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
        .or(NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M"));

    let datetime = match datetime {
        Ok(dt) => dt,
        Err(_) => {
            // Parse string as date only
            let date = NaiveDate::parse_from_str(text, "%Y-%m-%d");

            if date.is_ok() {
                date.unwrap().and_hms(0, 0, 0)
            } else {
                // Parse string as time only and use today's date
                let time = NaiveTime::parse_from_str(text, "%H:%M:%S")
                    .or(NaiveTime::parse_from_str(text, "%H:%M"))?;

                NaiveDateTime::new(Utc::today().naive_utc(), time)
            }
        }
    };

    Ok(Utc.from_local_datetime(&datetime).unwrap())
}

pub fn add(
    files: &DataFiles,
    project_name: &str,
    start_time: &str,
    end_time: &str,
    description: &Option<String>,
) -> Result<()> {
    let start_time = parse_local_datetime(&start_time)?;
    let end_time = parse_local_datetime(&end_time)?;

    let description: Option<String> = match description {
        Some(d) => Some(d.to_string()),
        None => None,
    };

    let task = Task::new(project_name.to_owned(), start_time, end_time, description);

    database::append_task(files.database_file(), &task)?;
    println!("Added to database: {}", &task);

    Ok(())
}

fn view_filter_shortcut(files: &DataFiles, filter: ViewFilter) -> Result<Vec<Task>> {
    let today: DateTime<Utc> = Utc::today().and_hms(0, 0, 0);

    match filter {
        ViewFilter::Current => Err(anyhow!("cannot view current task in table")),

        ViewFilter::All => database::extract_all_tasks(&files.database_file),

        ViewFilter::Day => {
            database::extract_tasks(&files.database_file, today, today + Duration::days(1))
        }

        ViewFilter::Week => {
            let mon = NaiveDate::from_isoywd(today.year(), today.iso_week().week(), Weekday::Mon)
                .and_hms(0, 0, 0);
            let mon = Utc.from_utc_datetime(&mon);

            database::extract_tasks(&files.database_file, mon, mon + Duration::days(7))
        }

        ViewFilter::Month => {
            let first = today
                .with_day(1)
                .expect("always valid because all months have a day 1");

            let last = match first.month() {
                12 => first // First day of the next year
                    .with_year(today.year() + 1)
                    .expect("invalid year")
                    .with_month(1)
                    .expect("hardcoded valid month"),

                m => first // First day of the next month
                    .with_month(m + 1)
                    .expect("m + 1 will always be a valid month"),
            };

            database::extract_tasks(&files.database_file, first, last)
        }

        ViewFilter::Year => {
            let first = today
                .with_month(1)
                .expect("hardcoded valid month")
                .with_day(1)
                .expect("hardcoded valid day");
            let last = first.with_year(first.year() + 1).expect("invalid year");

            database::extract_tasks(&files.database_file, first, last)
        }
    }
}

/// View task, or group of tasks, based on start time filtering
pub fn view(
    files: &DataFiles,
    filter: Option<ViewFilter>,
    from: &Option<String>,
    to: &Option<String>,
) -> Result<()> {
    let tasks = if from.is_none() & to.is_none() {
        // Use filter if after or before aren't given
        let filter = filter.unwrap_or(ViewFilter::Current);

        if matches!(filter, ViewFilter::Current) {
            let t = CurrentTask::load(files.current_file())?;
            println!("Current task: {}", t);
            return Ok(());
        }

        view_filter_shortcut(files, filter)?
    } else {
        let from = match from {
            Some(s) => parse_local_datetime(s)?,
            None => Utc.ymd(0, 1, 1).and_hms(0, 0, 0),
        };
        let to = match to {
            Some(s) => parse_local_datetime(s)?,
            None => Utc.ymd(9999, 1, 1).and_hms(0, 0, 0),
        };

        println!(
            "Showing results from {} - {}",
            from.to_rfc2822(),
            to.to_rfc2822()
        );

        if from > to {
            return Err(anyhow!(
                "from should be less than to, not {} and {}",
                from.to_rfc2822(),
                to.to_rfc2822()
            ));
        }

        database::extract_tasks(&files.database_file, from, to)?
    };

    display_tasks(&tasks);

    Ok(())
}

/// Print tasks to screen in a simple table structure.
fn display_tasks(tasks: &Vec<Task>) {
    println!("Found {} tasks", tasks.len());
    println!(
        "| {: <17} | {: <17} | {: <15} | {: <25} | {:0.50}",
        "From", "To", "Duration", "Project Name", "Description"
    );
    for t in tasks {
        println!("{}", t);
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone, Utc};

    /// Test parsing text with date and time.
    #[test]
    fn datetime_parse_valid() {
        let mut tests = Vec::new();
        tests.push((
            "2022-02-01 13:14:15",
            NaiveDate::from_ymd(2022, 2, 1).and_hms(13, 14, 15),
        ));
        tests.push((
            "2022-02-01 01:02",
            NaiveDate::from_ymd(2022, 2, 1).and_hms(1, 2, 0),
        ));
        tests.push((
            "2022-2-1 1:2:3",
            NaiveDate::from_ymd(2022, 2, 1).and_hms(1, 2, 3),
        ));
        tests.push((
            "2022-2-1 1:2",
            NaiveDate::from_ymd(2022, 2, 1).and_hms(1, 2, 0),
        ));
        tests.push(("2022-2-1", NaiveDate::from_ymd(2022, 2, 1).and_hms(0, 0, 0)));

        for (s, t) in tests {
            let t = Utc.from_local_datetime(&t).unwrap();

            assert_eq!(
                super::parse_local_datetime(s).unwrap(),
                t,
                "testing: parse_local_datetime({}) == {:?}",
                s,
                t
            );
        }
    }

    /// Test parsing text with time only.
    #[test]
    fn datetime_parse_time() {
        let mut tests = Vec::new();
        tests.push(("11:12:1", Utc::today().and_hms(11, 12, 1).naive_utc()));
        tests.push(("11:12", Utc::today().and_hms(11, 12, 0).naive_utc()));

        for (s, t) in tests {
            let t = Utc.from_local_datetime(&t).unwrap();

            assert_eq!(
                super::parse_local_datetime(s).unwrap(),
                t,
                "testing: parse_local_datetime({}) == {:?}",
                s,
                t
            );
        }
    }
}
