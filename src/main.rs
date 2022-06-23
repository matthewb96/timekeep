use std::fs;

use chrono::Utc;
use directories::BaseDirs;
use timekeep::{CurrentTask, Task};

const DATA_DIRECTORY: &str = "timekeep";
const CURRENT_ACTIVITY_FILE: &str = "current.json";

fn main() {
    let base_dirs = BaseDirs::new().unwrap();
    let data_folder = base_dirs.data_dir().join(DATA_DIRECTORY);
    if !data_folder.exists() {
        let folder = data_folder.display().to_string();
        fs::create_dir_all(&data_folder).expect(&folder);
    }
    let current_file = data_folder.join(CURRENT_ACTIVITY_FILE);

    let mut activity = CurrentTask::start(String::from("test"), None);

    dbg!(&activity);
    activity = activity
        .save(&current_file)
        .expect("Error saving current activity");

    let activity = activity.end();
    dbg!(&activity);
    dbg!(activity.duration());

    let today = Utc::today();

    let custom = Task::new(
        String::from("custom task"),
        today.and_hms(11, 30, 0),
        today.and_hms(12, 14, 12),
        Some(String::from("more info about this activity")),
    );
    dbg!(&custom);
    dbg!(custom.duration());
}
