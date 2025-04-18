// todo "learn databases" -> write message into json with meta data. status of job.
// todo -> should list out all incomplete tasks
// todo -r "task title" or "task id" -> remove that task.
// todo -c "task title" or "task id" -> complete a task.
// todo ls -a "list all tasks, complete and incompleted tasks"
// todo ls "list incomplete tasks"
// What should task look like? Struct -> id, title, task, status, date_created, date_completed

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};
use time::UtcDateTime;

#[derive(Debug, Deserialize, Serialize)]
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
    pub id: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Todo {
    pub id: Option<i32>,
    pub title: String,
    pub task: String,
    pub status: Status,
    pub date_created: UtcDateTime,
    pub date_completed: UtcDateTime,
}

pub mod todo {
    use super::*;

    /// Creates a new [`Todo`].
    pub fn new(task: &str, title: &str) -> Todo {
        Todo {
            id: Some(1),
            title: title.to_string(),
            task: task.to_string(),
            status: Status::Incomplete,
            date_created: UtcDateTime::now(),
            date_completed: UtcDateTime::now(),
        }
    }
}

impl Todo {
    fn update_status(&mut self, status: Status) {
        self.status = status;
    }

    fn update_date_completed(&mut self) {
        self.date_completed = UtcDateTime::now();
    }
}

pub fn read_tasks_from_file(cfg: &TodoConfig) -> Vec<Todo> {
    let file = File::open(&cfg.output).expect("file should exist");
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut buffer = String::new();

    let mut todos: Vec<Todo> = Vec::new();
    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(e) => panic!("Could not read from file"),
        };
        buffer.push_str(&line[..]);

        if line.chars().nth(0) == Some('}') {
            let todo: Todo = serde_json::from_str(&buffer).expect("a deserialized todo");
            todos.push(todo);
            buffer.clear();
        }
    }
    todos
}

pub fn write_task_to_file(cfg: &TodoConfig, todo: &Todo) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&cfg.output)
        .expect("a file opened for writing");
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    serde_json::to_writer_pretty(&mut writer, todo)?;
    writeln!(&mut writer, "{}", b'\n').expect("To write nextline character");

    writer.flush()?;

    Ok(())
}
