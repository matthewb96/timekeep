use std::{fs, path::Path};

use clap::{Parser, Subcommand};
use directories::BaseDirs;
use timekeep::{tasks, CurrentTask, Task};

const DATA_DIRECTORY: &str = "timekeep";
const CURRENT_ACTIVITY_FILE: &str = "current.json";

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
        #[clap(short, long, action)]
        overwrite: bool,
    },
    /// Save and end the current task
    End,
    /// Add a task with given start and end time
    Add,
}

fn main() {
    let base_dirs = BaseDirs::new().unwrap();
    let data_folder = base_dirs.data_dir().join(DATA_DIRECTORY);
    if !data_folder.exists() {
        let folder = data_folder.display().to_string();
        fs::create_dir_all(&data_folder).expect(&folder);
    }
    let cli = Cli::parse();

    let current_file = data_folder.join(CURRENT_ACTIVITY_FILE);

    match &cli.command {
        Commands::Start {
            project_name,
            description,
            overwrite,
        } => {
            if *overwrite {
                // End current task before starting a new one
                match tasks::end_current_task(&current_file) {
                    Ok(Some(task)) => println!("Ended task: {:?}", task),
                    Ok(None) => (),
                    Err(err) => eprintln!("Error when attempting to end current task: {}", err),
                }
            }

            let task = tasks::start_task(project_name, description.as_ref(), &current_file)
                .expect("oh crap");
            println!("Started task: {:?}", task);
        }
        Commands::End => match tasks::end_current_task(&current_file) {
            Ok(Some(task)) => println!("Ended task: {:?}", task),
            Ok(None) => println!("No current task to end"),
            Err(err) => eprintln!("Error when attempting to end current task: {}", err),
        },
        Commands::Add => println!("Not yet implemented"),
    };
}
