//! # Timekeep
//! Small command-line tool for tracking time spent on projects and tasks.
pub mod database;
pub mod tasks;

pub use tasks::CurrentTask;
pub use tasks::Task;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

mod projects {
    struct Project {
        name: String,
        description: Option<String>,
    }
}
