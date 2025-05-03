use anyhow::{self, Context, Result, bail};
use std::collections::HashSet;

use uuid::Uuid;

use storage::{atomic_write, load_from, open_or_init};

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
