use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
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
pub struct Task {
    pub id: Option<i32>,
    pub title: String,
    pub desc: String,
    pub status: Status,
    pub date_created: UtcDateTime,
    pub date_completed: UtcDateTime,
    pub priority: Option<u8>,
}

pub struct TaskBuilder(Task);

impl TaskBuilder {
    pub fn id(mut self, output: &str) -> Self {
        let tasks = read_tasks(output);
        let mut id = 1;
        for task in tasks {
            id = task.id.unwrap();
        }
        id += 1;
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
            id: Some(-1),
            title: String::from("Task"),
            desc: String::from("No description"),
            status: Status::Incomplete,
            date_created: UtcDateTime::now(),
            date_completed: UtcDateTime::now(),
            priority: None,
        })
    }

    fn _new(desc: &str) -> Self {
        Task::builder().desc(desc).build()
    }

    fn _status(&mut self, status: Status) {
        self.status = status;
    }

    fn _date_completed(&mut self) {
        self.date_completed = UtcDateTime::now();
    }
}

pub fn add_task(output: &str, desc: &str) -> Result<()> {
    //let task = Task::new(desc);
    let task = Task::builder().id(output).desc(desc).build();
    write_task(output, &task)?;
    Ok(())
}

pub fn complete_task(output: &str, id: i32) -> Result<()> {
    let mut tasks = read_tasks(output);
    for task in &mut tasks {
        if task.id.unwrap() == id {
            task.status = Status::Complete;
            break;
        }
    }
    write_tasks(output, tasks)?;
    Ok(())
}

pub fn remove_task(output: &str, id: i32) -> Result<()> {
    let mut tasks = read_tasks(output);
    let mut i = 0;
    for task in &tasks {
        if task.id.unwrap() == id {
            break;
        }
        i += 1;
    }

    for task in &tasks {
        println!("{:?}", task);
    }

    if i < tasks.len() {
        tasks.remove(i);
    }
    println!("--");

    for task in &tasks {
        println!("{:?}", task);
    }

    write_tasks(output, tasks)?;
    Ok(())
}

pub fn list_tasks(output: &str) -> Result<()> {
    let tasks = read_tasks(output);
    for task in tasks {
        println!(
            "[{}]  {:?}  {}",
            task.id.unwrap_or_default(),
            task.status,
            task.desc
        );
    }
    Ok(())
}

fn read_tasks(f_name: &str) -> Vec<Task> {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .open(f_name)
        .unwrap();
    //let file = File::open(f_name).expect("file should exist");
    let reader = BufReader::with_capacity(64 * 1024, file);
    let mut buffer = String::new();

    let mut todos: Vec<Task> = Vec::new();

    //match reader.read_line(&mut buffer) {
    //    Ok(l) => l,
    //    Err(_e) => panic!("Could not read count"),
    //}

    for line_result in reader.lines() {
        let line = line_result.expect("failed to parse line");

        //let line = match line_result {
        //    Ok(l) => (),
        //    Err(_e) => panic!("Could not read from file"),
        //};

        buffer.push_str(&line[..]);

        if line.chars().nth(0) == Some('}') {
            let todo: Task = serde_json::from_str(&buffer).expect("a deserialized todo");
            todos.push(todo);
            buffer.clear();
        }
    }
    todos
}

fn write_task(f_name: &str, todo: &Task) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(f_name)
        .expect("a file opened for appending");
    let mut writer = BufWriter::with_capacity(64 * 1024, file);
    serde_json::to_writer_pretty(&mut writer, todo)?;
    writeln!(&mut writer, "\n").expect("To write nextline character");

    writer.flush()?;
    Ok(())
}

fn write_tasks(f_name: &str, tasks: Vec<Task>) -> Result<()> {
    println!("--write_tasks--");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(f_name)
        .expect("a file opened for writing.");

    let mut writer = BufWriter::with_capacity(64 * 1024, file);

    for task in tasks {
        println!("{:?}", &task);
        serde_json::to_writer_pretty(&mut writer, &task)?;
        writeln!(&mut writer, "\n").expect("To write nextline character");
    }
    Ok(())
}
