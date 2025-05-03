// --- Atomic JSON persistence helpers ---

use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::{Context, Result};
use fs4::FileExt;
use serde::{Serialize, de::DeserializeOwned};
use tempfile::NamedTempFile; // For atomic writes

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    version: u32,
    current_id: u32,
    generated_at: TimeStamp,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TodoFile {
    meta: Meta,
    tasks: Vec<Task>,
}

impl TodoFile {
    pub fn new() -> Self {
        Self {
            meta: Meta {
                version: 1,
                current_id: 1,
                generated_at: TimeStamp::now_utc(),
            },
            tasks: Vec::new(),
        }
    }
}

/// Open storage file *with* a shared lock (read/write),
/// auto-creating and seeding if missing.
pub fn open_or_init(path: impl AsRef<Path>) -> Result<File> {
    let path = path.as_ref();

    // Try opening for read-write; create if file *doesn't exist*.
    match OpenOptions::new().read(true).write(true).open(path) {
        Ok(file) => {
            lock_file(&file)?;
            Ok(file)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true) // fail if raced
                .open(path)
                .with_context(|| format!("creating {}", path.display()))?;

            // Seed file with valid JSON.
            serde_json::to_writer_pretty(&mut file, &TodoFile::new())
                .context("seeding new JSON file.")?;

            // Ensure bytes hit kernel.
            file.flush()?;
            lock_file(&file)?;

            // Rewind so first caller can read immediately.
            file.seek(SeekFrom::Start(0))?;

            Ok(file)
        }
        Err(e) => Err(e).with_context(|| format!("opening {}", path.display())),
    }
}

/// Atomically write *any* serializable value to disk, replacing previous file contents
/// only when the entire payload is safely persisted.
pub fn atomic_write<T>(path: impl AsRef<Path>, value: &T) -> Result<()>
where
    T: Serialize,
{
    let path = path.as_ref();

    // Write into a temp file in the *same* directory.
    let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))
        .context("create temp file")?;

    serde_json::to_writer_pretty(&mut tmp, value).context("serializing JSON")?;

    // push os buffers
    tmp.flush()?;

    // fsync tempfile *and* containing directory
    tmp.as_file().sync_all()?;

    // atomic rename the tmp file with final path on POSIX, safe fallback on Windows
    tmp.persist(path)
        .with_context(|| format!("persist {}", path.display()))?;

    Ok(())
}

/// Read JSON from an already-opened & locked file into any deserializable type.
pub fn load_from<R, T>(mut file: R) -> Result<T>
where
    R: std::io::Read + Seek,
    T: DeserializeOwned,
{
    // Rewind since caller may have written.
    file.seek(SeekFrom::Start(0))?;

    let data = serde_json::from_reader(&file).context("JSON parse")?;

    Ok(data)
}

// --- Internal Helper: advisory locking ---
fn lock_file(file: &File) -> Result<()> {
    file.lock_exclusive()
        .with_context("Another process is already using the todo file.")?;
    Ok(())
}
