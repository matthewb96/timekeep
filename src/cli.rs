//! Functionality for the command-line interface.
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeZone, Utc};
use clap::{Parser, Subcommand};

use crate::{database, tasks, CurrentTask, DataFiles, Task};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
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
    /// View current task or a group of tasks based on options given
    View, // TODO Implement options for different views
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
/// If date isn't given then today is used.
fn parse_local_datetime(text: &str) -> Result<DateTime<Utc>> {
    // Attempt to parse datetime and fallback on parsing only time
    let datetime = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S")
        .or(NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M"));

    let datetime = match datetime {
        Ok(dt) => dt,
        Err(_) => {
            // Parse string as time only and use today's date
            let time = NaiveTime::parse_from_str(text, "%H:%M:%S")
                .or(NaiveTime::parse_from_str(text, "%H:%M"))?;

            NaiveDateTime::new(Utc::today().naive_utc(), time)
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

// TODO Add arguments for viewing different results from the database
pub fn view(files: &DataFiles) -> Result<()> {
    let t = CurrentTask::load(files.current_file())?;
    println!("Current task: {}", t);

    Ok(())
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
