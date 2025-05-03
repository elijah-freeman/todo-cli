use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// Self documenting alias
pub type TimeStamp = OffsetDateTime;

// --- Task Status ---
#[derive(Debug, Deserialize, Serialize)]
pub enum Status {
    Done,
    Pending,
    Canceled,
    InProgress,
}

// --- Task Object ---
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
pub struct MissingTitle;
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

// --- Entry-Point: Task::builder(). impl domain methods on `Task` ---
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

    /// Mark task as done
    pub fn mark_done(&mut self) {
        if self.status != Status::Done {
            self.status = Status::Done;
            let now = TimeStamp::now_utc();
            self.completed_at = Some(now);
            self.updated_at = Some(now);
        }
    }

    /// Change priority & update timestamp
    pub fn set_priority(&mut self, p: Option<u8>) {
        if self.priority != p {
            if let Some(val) = p {
                assert!((0..=5).contains(&val), "priority must be 0-5");
            }
            self.priority = p;
            self.updated_at = Some(TimeStamp::now_utc());
        }
    }

    /// Add a tag (case insensitive & prevents duplicates)
    pub fn add_tag<S: Into<String>>(&mut self, t: S) {
        let tag = t.into();
        if !self.tags.iter().any(|s| s.eq_ignore_ascii_case(&tag)) {
            self.tags.push(tag);
            self.updated_at(Some(TimeStamp::now_utc));
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

// --- File-level metadata ---
#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    version: u32,
    current_id: u32,
    generated_at: TimeStamp,
}

// -- Top level container ---
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
