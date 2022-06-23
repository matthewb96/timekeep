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
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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
        task_name: String,
        start: DateTime<Utc>,
        description: Option<String>,
    }

    impl CurrentTask {
        pub fn start(task_name: String, description: Option<String>) -> CurrentTask {
            CurrentTask {
                task_name,
                start: Utc::now(),
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
    }

    /// Task which started at a certain time and has already finished.
    ///
    /// Contains an optional description for more details.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Task {
        task_name: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        description: Option<String>,
    }

    impl Task {
        pub fn new(
            task_name: String,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
            description: Option<String>,
        ) -> Task {
            Task {
                task_name,
                start,
                end,
                description,
            }
        }

        pub fn duration(&self) -> Duration {
            self.end - self.start
        }
    }

    impl From<CurrentTask> for Task {
        /// Uses current time as the activity end time when converting.
        fn from(activity: CurrentTask) -> Task {
            Task {
                task_name: activity.task_name,
                start: activity.start,
                end: Utc::now(),
                description: activity.description,
            }
        }
    }
}

mod projects {
    struct Project {
        name: String,
        description: Option<String>,
    }
}
