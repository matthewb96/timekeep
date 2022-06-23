#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub use tasks::Task;
pub use tasks::CurrentTask;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod tasks {
    use std::fs;
    use std::io::Error;
    use std::path::Path;

    use chrono::{DateTime, Duration, Utc};
    use serde::{Deserialize, Serialize};

    /// Task which started at a certain time but is still ongoing.
    ///
    /// See `Task` for finished tasks.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CurrentTask {
        project_name: String,
        start_time: DateTime<Utc>,
        description: Option<String>,
    }

    impl CurrentTask {
        pub fn start(project_name: String, description: Option<String>) -> CurrentTask {
            CurrentTask {
                project_name,
                start_time: Utc::now(),
                description,
            }
        }

        pub fn end(self) -> Task {
            Task::from(self)
        }

        pub fn save(self, file: &Path) -> Result<CurrentTask, Error> {
            let json = serde_json::to_string(&self)?;

            fs::write(file, json)?;
            Ok(self)
        }

        pub fn load(file: &Path) -> Result<CurrentTask, Error> {
            let json = fs::read_to_string(file)?;

            let task: CurrentTask = serde_json::from_str(&json)?;
            Ok(task)
        }
    }

    /// Task which started at a certain time and has already finished.
    ///
    /// Contains an optional description for more details.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Task {
        project_name: String,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        description: Option<String>,
    }

    impl Task {
        pub fn new(
            project_name: String,
            start_time: DateTime<Utc>,
            end_time: DateTime<Utc>,
            description: Option<String>,
        ) -> Task {
            Task {
                project_name,
                start_time,
                end_time,
                description,
            }
        }

        pub fn duration(&self) -> Duration {
            self.end_time - self.start_time
        }
    }

    impl From<CurrentTask> for Task {
        /// Uses current time as the activity end time when converting.
        fn from(activity: CurrentTask) -> Task {
            Task {
                project_name: activity.project_name,
                start_time: activity.start_time,
                end_time: Utc::now(),
                description: activity.description,
            }
        }
    }

    pub fn start_task(
        project_name: &str,
        description: Option<&String>,
        current_file: &Path,
    ) -> Result<CurrentTask, std::io::Error> {
        let description: Option<String> = match description {
            Some(d) => Some(d.to_string()),
            None => None,
        };
        let task = CurrentTask::start(project_name.to_string(), description);
        task.save(&current_file)
    }
    
    pub fn end_current_task(current_file: &Path) -> Result<Option<Task>, std::io::Error> {
        if !current_file.exists() {
            return Ok(None);
        };
    
        let task = CurrentTask::load(&current_file)?;
        let task = task.end();
        fs::remove_file(current_file)?;
        Ok(Some(task))
    }
}

mod projects {
    struct Project {
        name: String,
        description: Option<String>,
    }
}
