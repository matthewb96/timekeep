//! Functionality for reading / writing to the persistent storage database.
use std::path::Path;

use crate::tasks::Task;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
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

/// Parse datetime string in RFC3339 format and convert to UTC.
fn parse_database_datetime(s: String) -> Result<DateTime<Utc>> {
    Ok(DateTime::<Utc>::from(DateTime::parse_from_rfc3339(&s)?))
}

fn extract_tasks_query(file: &Path, query: &str) -> Result<Vec<Task>> {
    let connection = Connection::open(&file)?;

    let mut stmt = connection.prepare(query)?;

    let mut errors = vec![];
    let tasks: Vec<Task> = stmt
        .query_map([], |row| {
            Ok(Task::new(
                row.get(0)?,
                parse_database_datetime(row.get(1)?).unwrap(),
                parse_database_datetime(row.get(2)?).unwrap(),
                row.get(3)?,
            ))
        })?
        .filter_map(|x| x.map_err(|e| errors.push(e)).ok())
        .collect();

    if errors.len() > 0 {
        return Err(anyhow!(
            "error extracting tasks from database: {:#?}",
            errors
        ));
    }

    Ok(tasks)
}

/// Extract all tasks from database.
pub fn extract_all_tasks(file: &Path) -> Result<Vec<Task>> {
    extract_tasks_query(
        file,
        "SELECT project_name, start_time, end_time, description FROM tasks;",
    )
}

/// Extract tasks from database with a start time between `from` and `to`.
pub fn extract_tasks(file: &Path, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<Task>> {
    extract_tasks_query(
        file, 
        &format!(
            "SELECT project_name, start_time, end_time, description FROM tasks WHERE start_time >= '{}' and start_time < '{}';",
            from.to_rfc3339(),
            to.to_rfc3339(),
        )
    )
}
