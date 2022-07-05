use std::{fs, path::Path};

use anyhow::{Ok, Result};
use clap::{Parser, Subcommand};
use directories::BaseDirs;

use timekeep::{tasks, CurrentTask};

const DATA_DIRECTORY: &str = "timekeep";
const CURRENT_ACTIVITY_FILE: &str = "current.json";
const DATABASE_FILE: &str = "timekeep.db";

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    current_file: &Path,
    project_name: &str,
    description: &Option<String>,
    overwrite: &bool,
    database_file: &Path,
) -> Result<()> {
    if !*overwrite {
        // End current task before starting a new one
        match tasks::end_current_task(&current_file, &database_file)? {
            Some(t) => println!("Ended task: {}", t),
            None => (),
        }
    }

    let t = tasks::start_task(project_name, description.as_ref(), &current_file)?;
    println!("Started task: {}", t);

    Ok(())
}

pub fn end(current_file: &Path, database_file: &Path) -> Result<()> {
    match tasks::end_current_task(&current_file, &database_file)? {
        Some(t) => println!("Ended task: {}", t),
        None => println!("No current task to end"),
    };

    Ok(())
}

pub fn view(current_file: &Path, database_file: &Path) -> Result<()> {
    let t = CurrentTask::load(&current_file)?;
    println!("Current task: {}", t);

    Ok(())
}

fn main() -> Result<()> {
    let base_dirs = BaseDirs::new().unwrap();
    let data_folder = base_dirs.data_dir().join(DATA_DIRECTORY);
    if !data_folder.exists() {
        let folder = data_folder.display().to_string();
        fs::create_dir_all(&data_folder).expect(&folder);
    }
    let cli = Cli::parse();

    let current_file = data_folder.join(CURRENT_ACTIVITY_FILE);
    let database_file = data_folder.join(DATABASE_FILE);

    match &cli.command {
        Commands::Start {
            project_name,
            description,
            overwrite,
        } => start(
            &current_file,
            project_name,
            description,
            overwrite,
            &database_file,
        )?,
        Commands::End => end(&current_file, &database_file)?,
        Commands::Add => println!("Not yet implemented"),
        Commands::View => view(&current_file, &database_file)?,
    };

    Ok(())
}
