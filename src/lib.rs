//! # Timekeep
//! Small command-line tool for tracking time spent on projects and tasks.
pub mod cli;
pub mod database;
pub mod tasks;

pub use tasks::CurrentTask;
pub use tasks::Task;

/// Crate version number.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Data directory name.
const DATA_DIRECTORY: &str = "timekeep";
/// Current activity file name.
const CURRENT_ACTIVITY_FILE: &str = "current.json";
/// Database file name.
const DATABASE_FILE: &str = "timekeep.db";

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Stores file paths for the program's persistent storage.
pub struct DataFiles {
    current_file: PathBuf,
    database_file: PathBuf,
    data_folder: PathBuf,
}

impl DataFiles {
    /// Initialise struct with default sub-folder and file names.
    pub fn new(base_folder: &Path) -> Result<DataFiles> {
        DataFiles::custom(
            base_folder,
            DATA_DIRECTORY,
            CURRENT_ACTIVITY_FILE,
            DATABASE_FILE,
        )
    }

    /// Initialise struct with custom names for data files.
    pub fn custom(
        base_folder: &Path,
        data_directory: &str,
        current_activity_file: &str,
        database_file: &str,
    ) -> Result<DataFiles> {
        if !base_folder.exists() {
            return Err(anyhow!("base folder doesn't exist: {:?}", base_folder));
        }

        let data_folder = base_folder.join(data_directory);
        if !data_folder.exists() {
            let folder = data_folder.display().to_string();
            fs::create_dir_all(&data_folder).expect(&folder);
        }

        Ok(DataFiles {
            current_file: data_folder.join(current_activity_file),
            database_file: data_folder.join(database_file),
            data_folder,
        })
    }

    /// JSON file storing the current running task.
    pub fn current_file(&self) -> &Path {
        &self.current_file
    }

    /// SQLite database storing all completed tasks and projects.
    pub fn database_file(&self) -> &Path {
        &self.database_file
    }

    /// Folder containing all persistent storage for timekeep.
    pub fn data_folder(&self) -> &Path {
        &self.data_folder
    }
}

mod projects {
    struct Project {
        name: String,
        description: Option<String>,
    }
}
