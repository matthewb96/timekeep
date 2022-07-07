//! Functionality for the command-line interface.
use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{tasks, CurrentTask, DataFiles};

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
    Add,
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

pub fn view(files: &DataFiles) -> Result<()> {
    let t = CurrentTask::load(files.current_file())?;
    println!("Current task: {}", t);

    Ok(())
}
