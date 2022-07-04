#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

use chrono::Duration;
pub use tasks::CurrentTask;
pub use tasks::Task;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Divides two integers and rounds result to nearest integer
///
/// # Examples
/// ```
/// use timekeep::rounded_div;
///
/// let tests = [
///     (3, 2, 2),   // 3 / 2 = 1.5    -> 2
///     (2, 3, 1),   // 2 / 3 = 0.66   -> 1
///     (1, 2, 1),   // 1 / 2 = 0.5    -> 1
///     (1, 3, 0),   // 1 / 3 = 0.33   -> 0
///     (-3, 2, -2), // -3 / 2 = -1.5  -> -2
///     (-2, -3, 1), // -2 / -3 = 0.66 -> 1
///     (1, -2, 1),  // 1 / -2 = -0.5  -> -1
///     (-1, 3, 0),  // -1 / 3 = 0.33  -> 0
/// ];
///
/// for (n, d, a) in tests {
///     assert_eq!(rounded_div(n, d), a, "testing: rounded_div({}, {}) == {}", n, d, a);
/// }
/// ```
pub fn rounded_div(numerator: i64, denominator: i64) -> i64 {
    (numerator + (denominator / 2)) / denominator
}

/// Format duration as a human readable string.
///
/// # Examples
/// ```
/// use chrono::Duration;
/// use timekeep::human_duration;
///
/// let durations = [
///     (Duration::milliseconds(947), "947 milliseconds"),
///     (Duration::milliseconds(1947), "2 seconds"),
///     (Duration::seconds(57), "57 seconds"),
///     (Duration::seconds(157), "2 minutes 37 seconds"),
///     (Duration::seconds(4734), "1 hours 19 minutes"),
///     (Duration::seconds(92750), "1 days 2 hours"),
///     (Duration::hours(173), "7 days 5 hours"),
/// ];
///
/// for (d, a) in durations {
///     assert_eq!(human_duration(d), a, "testing: human_duration({}) == {}", d, a);
/// }
/// ```
pub fn human_duration(d: Duration) -> String {
    let milli = d.num_milliseconds();
    if milli < 1000 {
        return format!("{} milliseconds", milli);
    };

    let mut seconds = rounded_div(milli, 1000);
    if seconds < 60 {
        return format!("{} seconds", seconds);
    };

    let mut minutes = seconds / 60;
    seconds -= minutes * 60;
    if minutes < 60 {
        return format!("{} minutes {} seconds", minutes, seconds);
    };

    let mut hours = minutes / 60;
    minutes = minutes - (hours * 60) + rounded_div(seconds, 60);
    if hours < 24 {
        return format!("{} hours {} minutes", hours, minutes);
    };

    let days = hours / 24;
    hours = hours - days * 24 + rounded_div(minutes, 60);
    format!("{} days {} hours", days, hours)
}

pub mod tasks {
    use std::io::Error;
    use std::path::Path;
    use std::{fmt, fs};

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

        pub fn duration(&self) -> Duration {
            Utc::now() - self.start_time
        }
    }

    impl fmt::Display for CurrentTask {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "project: {}, started at: {}, duration {}",
                self.project_name,
                self.start_time.naive_local().format("%R %v").to_string(),
                super::human_duration(self.duration())
            )
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

    impl fmt::Display for Task {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "project: {}, started at: {}, ended at: {}, duration {}",
                self.project_name,
                self.start_time.naive_local().format("%R %v").to_string(),
                self.end_time.naive_local().format("%R %v").to_string(),
                super::human_duration(self.duration())
            )
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
