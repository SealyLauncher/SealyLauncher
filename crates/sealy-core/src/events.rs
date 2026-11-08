//! Event and task shapes shared with the UI in later phases.

/// A short-lived UI notice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// User-visible message.
    pub message: String,
}

/// State of a long-running task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// Waiting to start.
    Queued,
    /// Currently running.
    Running,
    /// Finished successfully.
    Completed,
    /// Finished with an error.
    Failed,
    /// Cancelled by the user.
    Cancelled,
}

/// Snapshot of a task shown by the progress dock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSnapshot {
    /// Stable task id.
    pub id: u64,
    /// Short label.
    pub label: String,
    /// Lifecycle state.
    pub state: TaskState,
    /// Completed work units.
    pub current: u64,
    /// Total work units, or 0 when unknown.
    pub total: u64,
}

impl TaskSnapshot {
    /// Completion ratio in the range 0..=1.
    pub fn fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f32 / self.total as f32).clamp(0.0, 1.0)
        }
    }
}

/// Events emitted by the core to the UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// Show a transient notice.
    Notice(Notice),
    /// A task snapshot changed.
    Task(TaskSnapshot),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fraction_is_clamped() {
        let task = TaskSnapshot {
            id: 1,
            label: "test".to_owned(),
            state: TaskState::Running,
            current: 5,
            total: 10,
        };
        assert!((task.fraction() - 0.5).abs() < f32::EPSILON);
    }
}
