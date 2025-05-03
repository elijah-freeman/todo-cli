use anyhow::{self, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
};
use time::OffsetDateTime;
use uuid::Uuid;

use storage::{atomic_write, load_from, open_or_init};

// Self documenting alias
pub type TimeStamp = OffsetDateTime;

#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    Done,
    Pending,
    Canceled,
    InProgress,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    /** Immutable primary key (unique per task) */
    pub id: Uuid,

    /** Required short summary */
    pub title: String,

    /** Optional long description */
    pub desc: Option<String>,

    /** Workflow state machine */
    pub status: Status,

    /** 1 = highest, 0 = unprioritized */
    pub priority: u8,

    /** Labelling for filtering */
    pub tags: Vec<String>,

    /** Created at UTC time (immutable once set) */
    pub created_at: TimeStamp,

    /** Last time user updated task */
    pub updated_at: Option<TimeStamp>,

    /** When task reached a 'Done' state */
    pub completed_at: Option<TimeStamp>,
}

// --- Zero size markers for the "typed-state" builder ---

/** Compile time flag: title has not been set */
pub struct MissingTitle;

/** Compile time flag: title has been set */
pub struct HasTitle;

// --- Generic Builder struct ---
pub struct TaskBuilder<TitleState> {
    /// Every field mirrors [`Task`] but they are *all* optional
    /// until `.build()` proves invariants are satisfied.
    title: Option<String>,
    desc: Option<String>,
    priority: u8,
    tags: Vec<String>,

    // zero-cost phantom marker to record builder state in type system
    _state: std::marker::PhantomData<TitleState>,
}

// --- Entry-Point: Task::builder() ---
impl Task {
    /// Creates a new builder chain (*without* a title).
    pub fn builder() -> TaskBuilder<MissingTitle> {
        TaskBuilder {
            title: None,
            desc: None,
            priority: 0,
            tags: Vec::new(),
            _state: std::marker::PhantomData,
        }
    }
}

// --- Stage-1: impl: methods available *before* title exists ---
impl TaskBuilder<MissingTitle> {
    pub fn title<S: Into<String>>(mut self, t: S) -> TaskBuilder<HasTitle> {
        self.title = Some(t.into());
        TaskBuilder {
            title: self.title,
            desc: self.desc,
            priority: self.priority,
            tags: self.tags,
            _state: std::marker::phantomData, // Flips to HasTitle marker
        }
    }
}

// --- Stage-2: impl: common (setter) methods available in *either* state ---
impl<TitleState> TaskBuilder<TitleState> {
    /// Optional free-text description.
    pub fn desc<S: Into<String>>(mut self, d: S) -> Self {
        self.desc = Some(d.into());
        self
    }

    /// Optional priority (assert range 0-5).
    pub fn priority(mut self, p: u8) -> Self {
        assert!((0..=5).contains(&p), "Priority must be 0-5.");
        self.priority = p;
        self
    }

    /// Add a single tag.
    pub fn tag<S: Into<String>>(mut self, t: S) -> Self {
        self.tags.push(t.into());
        self
    }
}

// --- Final-Stage: impl: .build() only once title supplied --
impl TaskBuilder<HasTitle> {
    /// Consume builder and return fully-formed [`Task`]
    pub fn build(self) -> Task {
        let now = TimeStamp::now_utc();
        Task {
            id: Uuid::new_v4(),
            title: self.title.unwrap(),
            desc: self.desc,
            status: Status::Pending,
            tags: self.tags,
            created_at: now,
            updated_at: None,
            completed_at: None,
        }
    }
}

pub fn add_task(path: &str, task: Task) -> Result<()> {
    // Open (or create) the storage file, grab exclusive lock.
    let file = open_or_init(path)?;

    // Deserialize current state. Passes &File as Read+Seek.
    let mut data: TodoFile = load_from(&file)?;

    // Mutate in memory
    data.tasks.push(task);

    // Write back atomically *after* we drop the lock.
    atomic_write(path, &data)?;

    anyhow::Ok(())
}

pub fn complete_task(f_name: &str, id: u32) -> Result<()> {
    let file = validate_storage(&f_name)?;
    let mut todo: TodoFile = serde_json::from_reader(&file)?;

    todo.meta.version += 1;
    for task in &mut todo.tasks {
        if task.id.unwrap() == id {
            task.status = Status::Done;
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
