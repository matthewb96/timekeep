//! Functionality for reading / writing to the persistent storage database.
use std::path::Path;

use crate::tasks::Task;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

fn table_exists(file: &Path, table: &str) -> Result<bool> {
    let connection = Connection::open(&file)?;

    let rows: Option<()> = connection
        .query_row(
            &format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}';",
                &table
            ),
            [],
            |_| Ok(()),
        )
        .optional()?;

    match rows {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

fn create_tasks_table(file: &Path) -> Result<()> {
    let connection = Connection::open(&file)?;

    connection.execute(
        "CREATE TABLE tasks (
                project_name    TEXT NOT NULL,
                start_time      TEXT NOT NULL,
                end_time        TEXT NOT NULL,
                description     TEXT
            );",
        [],
    )?;

    Ok(())
}

pub fn append_task(file: &Path, task: &Task) -> Result<()> {
    let connection = Connection::open(&file)?;
    if !table_exists(&file, "tasks")? {
        create_tasks_table(&file)?;
    };

    connection.execute(
        "INSERT INTO tasks VALUES (?1, ?2, ?3, ?4)",
        params![
            task.project_name(),
            task.start_time().to_rfc3339(),
            task.end_time().to_rfc3339(),
            task.description()
        ],
    )?;

    Ok(())
}
