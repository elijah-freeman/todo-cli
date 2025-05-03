use anyhow::{self, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
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

/// Mark task as done.
pub fn complete_task(path: &str, id: Uuid) -> Result<()> {
    // Open and lock storage file.
    let file = open_or_init(path)?;
    let mut data: TodoFile = load_from(&file)?;

    // Find task mutably in place.
    match data.tasks.iter_mut().find(|t| t.id == id) {
        Some(task) => {
            // Only change if task not already done.
            if task.status != Status::Done {
                task.status = Status::Done;
                task.updated_at = Some(TimeStamp::now_utc());
                task.completed_at = task.updated_at;
            }
        }
        None => bail!("task {id} not found"),
    }

    // Release lock on file drop.
    drop(file);
    atomic_write(path, &data).context("Writing updated task list.")
}

/// Remove a task (returns error if id is missing).
pub fn remove_task(path: &str, id: Uuid) -> Result<()> {
    let file = open_or_init(path)?;
    let mut data: TodoFile = load_from(&file)?;

    // `retain` keeps all elements for which the predicate is *true*.
    let before = data.tasks.len();
    data.tasks.retain(|t| t.id != id);
    let after = data.tasks.len();

    if before == after {
        bail!("task {id} is not found");
    }

    drop(file);
    atomic_write(path, &data).context("Persisting after remove")
}

/// List tasks, optionally filtered out by priority and/or tags.
/// Prints to stdout.
pub fn list_tasks(path: &str, priority_filter: Option<u8>, tag_filter: &[String]) -> Result<()> {
    // Pre-lowercase tag filter once, not per task.
    let needle: HashSet<String> = tag_filter.iter().map(|s| s.to_ascii_lowercase()).collect();

    let file = open_or_init(path)?;
    let data: TodoFile = load_from(&file);

    println!("ID                               | Pri | Status      | Title");
    println!("----------------------------------+-----+-------------+----------------");

    data.tasks
        .iter()
        .filter(|t| match priority_filter {
            Some(p) => t.priority == Some(p),
            None => true,
        })
        .filter(|t| {
            if needle.is_empty() {
                true
            } else {
                // Build lowercase set for task once per task.
                let task_tags: HashSet<String> =
                    t.tags.iter().map(|s| s.to_ascii_lowercase()).collect();
                needle.is_subset(&task_tags)
            }
        })
        .for_each(|t| {
            println!(
                "{:<34} | {:<3} | {:<11} | {}",
                t.id,
                t.priority.map_or("--", |p| p.to_string().as_str()),
                format!("{:?}", t.status).to_ascii_lowercase(),
                t.title
            )
        });

    anyhow::Ok(())
}
