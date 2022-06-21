use std::{fs, path::Path};

use chrono::{DateTime, Duration, Utc};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};

const DATA_DIRECTORY: &str = "timekeep";
const CURRENT_ACTIVITY_FILE: &str = "current.json";

struct Task {
    name: String,
    description: Option<String>,
}

/// Activity which started at a certain time but is still ongoing.
///
/// See `Activity` for finished activities.
#[derive(Debug, Serialize, Deserialize)]
struct CurrentActivity {
    task_name: String,
    start: DateTime<Utc>,
    description: Option<String>,
}

impl CurrentActivity {
    fn start(task_name: String, description: Option<String>) -> CurrentActivity {
        CurrentActivity {
            task_name,
            start: Utc::now(),
            description,
        }
    }

    fn end(self) -> Activity {
        Activity::from(self)
    }

    fn save(self, folder: &Path) -> CurrentActivity {
        let path = folder.join(CURRENT_ACTIVITY_FILE);
        let json = serde_json::to_string(&self).expect("cannot convert current activity to json");

        fs::write(path, json).expect("Unable to save current activity");
        self
    }
}

/// Activity which started at a certain time and has already finished.
///
/// Contains an optional description for more details.
#[derive(Debug, Serialize, Deserialize)]
struct Activity {
    task_name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    description: Option<String>,
}

impl Activity {
    fn new(
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

    fn duration(&self) -> Duration {
        self.end - self.start
    }
}

impl From<CurrentActivity> for Activity {
    fn from(activity: CurrentActivity) -> Activity {
        Activity {
            task_name: activity.task_name,
            start: activity.start,
            end: Utc::now(),
            description: activity.description,
        }
    }
}

fn main() {
    let base_dirs = BaseDirs::new().unwrap();
    let data_folder = base_dirs.data_dir().join(DATA_DIRECTORY);
    if !data_folder.exists() {
        let folder = data_folder.display().to_string();
        fs::create_dir_all(&data_folder).expect(&folder);
    }

    let mut activity = CurrentActivity::start(String::from("test"), None);

    dbg!(&activity);
    activity = activity.save(&data_folder);

    let activity = activity.end();
    dbg!(&activity);
    dbg!(activity.duration());

    let today = Utc::today();

    let custom = Activity::new(
        String::from("custom task"),
        today.and_hms(11, 30, 0),
        today.and_hms(12, 14, 12),
        Some(String::from("more info about this activity")),
    );
    dbg!(&custom);
    dbg!(custom.duration());
}
