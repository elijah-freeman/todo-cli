// todo "learn databases" -> write message into json with meta data. status of job.
// todo -> should list out all incomplete tasks
// todo -r "task title" or "task index" -> remove that task.
// todo -c "task title" or "task index" -> complete a task.
// todo ls -a "list all tasks, complete and incompleted tasks"
// todo ls "list incomplete tasks"
// What should task look like? Struct -> index, title, task, status, date_created, date_completed

use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
};
use time::UtcDateTime;

#[derive(Debug)]
pub enum Status {
    Complete,
    Incomplete,
    Canceled,
    Removed,
}

pub struct TodoConfig {
    pub title: String,
    pub task: String,
    pub output: PathBuf,
    pub index: Option<i32>,
}

#[derive(Debug)]
pub struct Todo {
    pub index: Option<i32>,
    pub title: String,
    pub task: String,
    pub status: Status,
    pub date_created: UtcDateTime,
    pub date_completed: UtcDateTime,
}

pub mod todo {
    use super::*;

    pub fn new(task: &str, title: &str) -> Todo {
        Todo {
            index: Some(1),
            title: title.to_string(),
            task: task.to_string(),
            status: Status::Incomplete,
            date_created: UtcDateTime::now(),
            date_completed: UtcDateTime::now(),
        }
    }
}

impl Todo {
    /// Creates a new [`Todo`].

    fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    fn update_date_completed(&mut self) {
        self.date_completed = UtcDateTime::now();
    }
}

pub fn write_task_to_file(cfg: &TodoConfig, todo: &Todo) -> Result<()> {
    let file = File::create(&cfg.output).with_context(|| format!("Error creating file"))?;
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    writeln!(writer, "{}", format!("Task at hand {}", todo.task));
    writer.flush()?;
    Ok(())
}
