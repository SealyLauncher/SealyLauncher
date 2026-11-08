//! Installing one version: metadata, client jar and libraries.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use sha1::{Digest, Sha1};

use crate::config::SourcePolicy;
use crate::error::CoreError;
use crate::manifest;
use crate::net;
use crate::version::{self, Task};

/// Snapshot of an install, shared between the worker and the UI.
#[derive(Debug, Clone, Default)]
pub struct Progress {
    /// Files planned.
    pub total: usize,
    /// Files finished (including skipped ones).
    pub done: usize,
    /// File currently being fetched.
    pub current: String,
    /// Bytes of the current file already written.
    pub current_bytes: u64,
    /// Expected size of the current file, zero when unknown.
    pub current_size: u64,
    /// Bytes written so far.
    pub bytes: u64,
    /// Set once the worker stops, successfully or not.
    pub finished: bool,
    /// Failure message.
    pub error: Option<String>,
}

/// Shared handle the worker writes progress into.
pub type ProgressHandle = Arc<Mutex<Progress>>;

/// Cooperative cancellation flag; checked between files.
pub type CancelFlag = Arc<AtomicBool>;

/// What an install produced.
#[derive(Debug, Clone)]
pub struct Installed {
    /// Installed version id.
    pub id: String,
    /// Files fetched or verified.
    pub files: usize,
    /// Bytes fetched.
    pub bytes: u64,
    /// Java component and required major version, when declared.
    pub java: Option<(String, u32)>,
    /// Asset index id and URL, when declared.
    pub assets: Option<(String, String)>,
}

/// Installs `id` under `root`, fetching metadata first.
pub fn install(
    root: &Path,
    id: &str,
    policy: SourcePolicy,
    concurrency: usize,
    progress: ProgressHandle,
    cancel: CancelFlag,
) -> Result<Installed, CoreError> {
    let result = install_inner(root, id, policy, concurrency, &progress, &cancel);
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.finished = true;
    if let Err(err) = &result {
        state.error = Some(err.to_string());
    }
    result
}

fn install_inner(
    root: &Path,
    id: &str,
    policy: SourcePolicy,
    concurrency: usize,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<Installed, CoreError> {
    let manifest = manifest::fetch_manifest(policy)?;
    let summary = manifest
        .versions
        .iter()
        .find(|version| version.id == id)
        .ok_or_else(|| CoreError::Network(format!("version {id} is not in the manifest")))?;

    let body = net::get_text(&summary.url)?;
    let doc: version::VersionDocument = serde_json::from_str(&body)?;

    let version_dir = root.join("versions").join(id);
    fs::create_dir_all(&version_dir)?;
    fs::write(version_dir.join(format!("{id}.json")), &body)?;

    let tasks = doc.plan(id);
    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.total = tasks.len();
    }

    let bytes = run_tasks(root, tasks, concurrency, progress, cancel)?;

    Ok(Installed {
        id: id.to_owned(),
        files: {
            let state = progress.lock().unwrap_or_else(|err| err.into_inner());
            state.done
        },
        bytes,
        java: doc
            .java_version
            .as_ref()
            .filter(|java| java.major_version > 0)
            .map(|java| (java.component.clone(), java.major_version)),
        assets: doc
            .asset_index
            .as_ref()
            .map(|index| (index.id.clone(), index.url.clone())),
    })
}

/// Downloads every task across a small worker pool.
///
/// Files whose SHA-1 already matches are skipped, so re-installing is cheap.
pub fn run_tasks(
    root: &Path,
    tasks: Vec<Task>,
    concurrency: usize,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<u64, CoreError> {
    let queue = Arc::new(Mutex::new(tasks));
    let bytes = Arc::new(AtomicU64::new(0));
    let failure: Arc<Mutex<Option<CoreError>>> = Arc::new(Mutex::new(None));
    let workers = concurrency
        .clamp(1, 16)
        .min(queue.lock().unwrap().len().max(1));

    std::thread::scope(|scope| {
        for _ in 0..workers {
            let queue = Arc::clone(&queue);
            let bytes = Arc::clone(&bytes);
            let failure = Arc::clone(&failure);

            scope.spawn(move || {
                let Ok(agent) = net::agent() else {
                    if let Ok(mut slot) = failure.lock()
                        && slot.is_none()
                    {
                        *slot = Some(CoreError::Network("tls backend unavailable".to_owned()));
                    }
                    cancel.store(true, Ordering::Relaxed);
                    return;
                };

                loop {
                    if cancel.load(Ordering::Relaxed) {
                        break;
                    }

                    let Some(task) = queue.lock().unwrap().pop() else {
                        break;
                    };

                    match fetch_one(&agent, root, &task, progress) {
                        Ok(written) => {
                            bytes.fetch_add(written, Ordering::Relaxed);
                            bump(progress, &task.path, written);
                        }
                        Err(err) => {
                            if let Ok(mut slot) = failure.lock()
                                && slot.is_none()
                            {
                                *slot = Some(err);
                            }
                            cancel.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
            });
        }
    });

    if let Some(err) = failure.lock().unwrap_or_else(|e| e.into_inner()).take() {
        return Err(err);
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(CoreError::Network("install cancelled".to_owned()));
    }

    Ok(bytes.load(Ordering::Relaxed))
}

fn fetch_one(
    agent: &ureq::Agent,
    root: &Path,
    task: &Task,
    progress: &ProgressHandle,
) -> Result<u64, CoreError> {
    let destination = root.join(&task.path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    if is_current(&destination, task) {
        return Ok(0);
    }

    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.current = task.path.clone();
        state.current_bytes = 0;
        state.current_size = task.size;
    }

    let mut file = fs::File::create(&destination)?;
    let written = match net::download_to(agent, &task.url, &mut file, &mut |written| {
        if let Ok(mut state) = progress.lock() {
            state.current_bytes = written;
        }
    }) {
        Ok(bytes) => bytes,
        Err(err) => {
            drop(file);
            let _ = fs::remove_file(&destination);
            return Err(err);
        }
    };
    drop(file);

    if !task.sha1.is_empty() {
        let actual = sha1_of(&destination)?;
        if !actual.eq_ignore_ascii_case(&task.sha1) {
            let _ = fs::remove_file(&destination);
            return Err(CoreError::Network(format!(
                "checksum mismatch for {}",
                task.path
            )));
        }
    }

    Ok(written)
}

fn bump(progress: &ProgressHandle, path: &str, bytes: u64) {
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.done += 1;
    state.current = path.to_owned();
    state.bytes += bytes;
}

fn is_current(destination: &PathBuf, task: &Task) -> bool {
    let Ok(metadata) = fs::metadata(destination) else {
        return false;
    };
    if task.size > 0 && metadata.len() != task.size {
        return false;
    }
    if task.sha1.is_empty() {
        return false;
    }
    match sha1_of(destination) {
        Ok(actual) => actual.eq_ignore_ascii_case(&task.sha1),
        Err(_) => false,
    }
}

/// Hex SHA-1 of a file.
pub fn sha1_of(path: &Path) -> Result<String, CoreError> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scratch directory under `target/`, since the system temp dir is not
    /// always writable inside the build sandbox.
    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/test-scratch")
            .join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn hashes_known_content() {
        let dir = scratch("sha1");
        let file = dir.join("a.txt");
        fs::write(&file, b"abc").unwrap();
        assert_eq!(
            sha1_of(&file).unwrap(),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn skips_matching_files() {
        let dir = scratch("skip");
        let file = dir.join("b.txt");
        fs::write(&file, b"abc").unwrap();
        let task = Task {
            url: "https://example.invalid/b.txt".to_owned(),
            path: "b.txt".to_owned(),
            sha1: "a9993e364706816aba3e25717850c26c9cd0d89d".to_owned(),
            size: 3,
        };
        assert!(is_current(&file, &task));

        let progress = Arc::new(Mutex::new(Progress::default()));
        let cancel = Arc::new(AtomicBool::new(false));
        let bytes = run_tasks(&dir, vec![task], 2, &progress, &cancel).unwrap();
        assert_eq!(bytes, 0);
        assert_eq!(progress.lock().unwrap().done, 1);
    }
}
