#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub use activities::Activity;
pub use activities::CurrentActivity;

pub mod activities {
    use std::fs;
    use std::io::Error;
    use std::path::Path;

    use chrono::{DateTime, Duration, Utc};
    use serde::{Deserialize, Serialize};

    /// Activity which started at a certain time but is still ongoing.
    ///
    /// See `Activity` for finished activities.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CurrentActivity {
        task_name: String,
        start: DateTime<Utc>,
        description: Option<String>,
    }

    impl CurrentActivity {
        pub fn start(task_name: String, description: Option<String>) -> CurrentActivity {
            CurrentActivity {
                task_name,
                start: Utc::now(),
                description,
            }
        }

        pub fn end(self) -> Activity {
            Activity::from(self)
        }

        pub fn save(self, file: &Path) -> Result<CurrentActivity, Error> {
            let json = serde_json::to_string(&self)?;

            fs::write(file, json)?;
            Ok(self)
        }
    }

    /// Activity which started at a certain time and has already finished.
    ///
    /// Contains an optional description for more details.
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Activity {
        task_name: String,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        description: Option<String>,
    }

    impl Activity {
        pub fn new(
            task_name: String,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
            description: Option<String>,
        ) -> Activity {
            Activity {
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

    impl From<CurrentActivity> for Activity {
        /// Uses current time as the activity end time when converting.
        fn from(activity: CurrentActivity) -> Activity {
            Activity {
                task_name: activity.task_name,
                start: activity.start,
                end: Utc::now(),
                description: activity.description,
            }
        }
    }
}

mod tasks {
    struct Task {
        name: String,
        description: Option<String>,
    }
}
