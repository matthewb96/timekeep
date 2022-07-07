//! Functionality for the command-line interface.
use anyhow::Result;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
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
        /// Optional task description
        #[clap(short, long)]
        description: Option<String>,
        /// Overwrite current task instead of ending that and starting a new one
        #[clap(short, long)]
        overwrite: bool,
    },
    /// Save and end the current task
    End,
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
}

pub fn start(
    files: &DataFiles,
    project_name: &str,
    description: &Option<String>,
    overwrite: &bool,
) -> Result<()> {
    if !*overwrite {
        // End current task before starting a new one
        match tasks::end_current_task(files.current_file(), files.database_file())? {
            Some(t) => println!("Ended task: {}", t),
            None => (),
        }
    }

    let t = tasks::start_task(project_name, description.as_ref(), files.current_file())?;
    println!("Started task: {}", t);

    Ok(())
}

pub fn end(files: &DataFiles) -> Result<()> {
    match tasks::end_current_task(files.current_file(), files.database_file())? {
        Some(t) => println!("Ended task: {}", t),
        None => println!("No current task to end"),
    };

    Ok(())
}

/// Parse datetime string which doesn't include timezone, use local timezone.
fn parse_local_time(datetime: &str) -> Result<DateTime<Utc>> {
    let datetime = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S")
        .or(NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M"))?;
    Ok(Utc.from_local_datetime(&datetime).unwrap())
}

pub fn add(
    files: &DataFiles,
    project_name: &str,
    start_time: &str,
    end_time: &str,
    description: &Option<String>,
) -> Result<()> {
    let start_time = parse_local_time(&start_time)?;
    let end_time = parse_local_time(&end_time)?;

    let description: Option<String> = match description {
        Some(d) => Some(d.to_string()),
        None => None,
    };

    let task = Task::new(project_name.to_owned(), start_time, end_time, description);

    database::append_task(files.database_file(), &task)?;
    println!("Added to database: {}", &task);

    Ok(())
}

pub fn view(files: &DataFiles) -> Result<()> {
    let t = CurrentTask::load(files.current_file())?;
    println!("Current task: {}", t);

    Ok(())
}
