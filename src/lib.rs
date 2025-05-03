use anyhow::{self, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
};
use time::UtcDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    Complete,
    Incomplete,
    Canceled,
    Removed,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    version: u32,
    current_id: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoFile {
    meta: Meta,
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: Option<u32>,
    pub title: String,
    pub desc: String,
    pub tags: Vec<String>,
    pub status: Status,
    pub date_created: UtcDateTime,
    pub date_completed: UtcDateTime,
    pub priority: Option<u8>,
}

pub struct TaskBuilder(Task);

impl TaskBuilder {
    pub fn id(mut self, id: u32) -> Self {
        self.0.id = Some(id);
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.0.title = title.to_string();
        self
    }

    pub fn desc(mut self, desc: &str) -> Self {
        self.0.desc = desc.to_string();
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.0.tags = tags;
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.0.status = status;
        self
    }

    pub fn priority(mut self, priority: u8) -> Self {
        self.0.priority = Some(priority);
        self
    }

    pub fn build(self) -> Task {
        self.0
    }
}

impl Task {
    /// Creates a new [`Task`].
    pub fn builder() -> TaskBuilder {
        TaskBuilder(Task {
            id: Some(0),
            title: String::from("Task"),
            desc: String::from("No description"),
            tags: vec![],
            status: Status::Incomplete,
            date_created: UtcDateTime::now(),
            date_completed: UtcDateTime::now(),
            priority: Some(0),
        })
    }
}

pub fn add_task(
    f_name: &str,
    desc: &str,
    title: &str,
    priority: Option<u8>,
    tags: Option<Vec<String>>,
) -> Result<()> {
    let mut task = Task::builder().desc(desc).build();

    let file = validate_storage(&f_name)?;
    let mut todo: TodoFile = serde_json::from_reader(&file)?;

    task.id = Some(todo.meta.current_id);
    task.title = title.to_string();
    task.tags = {
        let default = vec![];
        match tags {
            Some(x) => x,
            None => default,
        }
    };
    task.priority = priority;

    todo.meta.version += 1;
    todo.meta.current_id += 1;
    todo.tasks.push(task);

    serde_json::to_writer_pretty(File::create(f_name)?, &todo)?;

    anyhow::Ok(())
}

pub fn complete_task(f_name: &str, id: u32) -> Result<()> {
    let file = validate_storage(&f_name)?;
    let mut todo: TodoFile = serde_json::from_reader(&file)?;

    todo.meta.version += 1;
    for task in &mut todo.tasks {
        if task.id.unwrap() == id {
            task.status = Status::Complete;
            break;
        }
    }

    serde_json::to_writer_pretty(File::create(&f_name)?, &todo)?;

    anyhow::Ok(())
}

pub fn remove_task(f_name: &str, id: u32) -> Result<()> {
    let file = validate_storage(&f_name)?;
    let mut todo: TodoFile = serde_json::from_reader(&file)?;

    todo.meta.version += 1;

    let mut index = 0;
    for task in &todo.tasks {
        if task.id.unwrap() == id {
            break;
        }
        index += 1;
    }
    todo.tasks.remove(index);

    serde_json::to_writer_pretty(File::create(f_name)?, &todo)?;

    anyhow::Ok(())
}

fn print_task(task: &Task) {
    println!(
        "[{}]  {}  {:?}  {}",
        task.id.unwrap_or_default(),
        task.priority.unwrap_or(0),
        task.status,
        task.desc
    );
}

pub fn list_tasks(f_name: &str, priority: Option<u8>, tags: Option<Vec<String>>) -> Result<()> {
    println!("ID  PRIORITY  STATUS   DESC");

    let file = validate_storage(f_name)?;
    let todo: TodoFile = serde_json::from_reader(&file)?;

    let is_filter_applied = priority.is_some() || tags.is_some();

    for task in &todo.tasks {
        if priority.is_some_and(|p| p == task.priority.unwrap_or(0)) {
            print_task(task);
            continue;
        }

        if tags.is_some() && !task.tags.is_empty() {
            'tag_loop: for tag in &tags.clone().unwrap() {
                for task_tags in &task.tags {
                    if task_tags.to_lowercase() == tag.to_lowercase() {
                        print_task(task);
                        break 'tag_loop;
                    }
                }
            }
            continue;
        }

        if !is_filter_applied {
            print_task(task);
        }
    }
    anyhow::Ok(())
}

pub fn validate_storage(f_name: &str) -> Result<File> {
    let result = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .open(f_name);
    let file = match result {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::AlreadyExists => {
                match OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create_new(false)
                    .open(f_name)
                {
                    Ok(file) => return Ok(file),
                    Err(_) => panic!("Could not open file {}.", f_name),
                }
            }
            other_error => panic!("Problem validating file {other_error:?}"),
        },
    };

    let data = TodoFile {
        meta: Meta {
            version: 1,
            current_id: 1,
        },
        tasks: vec![],
    };

    serde_json::to_writer_pretty(&file, &data)?;

    Ok(file)
}
