//! Asset index handling: objects live under `<game>/assets/objects`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use serde::Deserialize;

use crate::error::CoreError;
use crate::install::{self, CancelFlag, ProgressHandle};
use crate::net;
use crate::version::Task;

/// Mojang's CDN prefix for asset objects.
const RESOURCE_ROOT: &str = "https://resources.download.minecraft.net";

/// `assets/indexes/<id>.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    /// Logical name to object mapping.
    #[serde(default)]
    pub objects: BTreeMap<String, AssetObject>,
    /// Present for pre-1.7 layouts.
    #[serde(default)]
    pub map_to_resources: bool,
    /// Present for pre-1.7 layouts.
    #[serde(default)]
    pub r#virtual: bool,
}

/// One asset object.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    /// Hex SHA-1, also the on-disk name.
    pub hash: String,
    /// Size in bytes.
    #[serde(default)]
    pub size: u64,
}

impl AssetObject {
    /// Relative path under `assets/objects`.
    pub fn path(&self) -> String {
        format!(
            "assets/objects/{}/{}",
            &self.hash[..2.min(self.hash.len())],
            self.hash
        )
    }
}

/// Downloads the asset index for `id` and every object it references.
pub fn install(
    game_dir: &Path,
    index_url: &str,
    id: &str,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<usize, CoreError> {
    let result = install_inner(game_dir, index_url, id, progress, cancel);

    // Callers poll `finished`; set it on every exit path.
    let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
    state.finished = true;
    if let Err(err) = &result {
        state.error = Some(err.to_string());
    }

    result
}

fn install_inner(
    game_dir: &Path,
    index_url: &str,
    id: &str,
    progress: &ProgressHandle,
    cancel: &CancelFlag,
) -> Result<usize, CoreError> {
    let text = net::get_text(index_url)?;
    let index: AssetIndex = serde_json::from_str(&text)?;

    let indexes = game_dir.join("assets").join("indexes");
    fs::create_dir_all(&indexes)?;
    fs::write(indexes.join(format!("{id}.json")), &text)?;

    let mut tasks = Vec::new();
    for object in index.objects.values() {
        let path = object.path();
        if game_dir.join(&path).is_file() {
            continue;
        }
        tasks.push(Task {
            url: format!(
                "{RESOURCE_ROOT}/{}/{}",
                &object.hash[..2.min(object.hash.len())],
                object.hash
            ),
            path,
            sha1: object.hash.clone(),
            size: object.size,
        });
    }

    {
        let mut state = progress.lock().unwrap_or_else(|err| err.into_inner());
        state.total = tasks.len();
        state.current = format!("assets {id}");
    }

    if tasks.is_empty() {
        return Ok(0);
    }

    let count = {
        // Objects are small and numerous; a wider pool pays off.
        let before = progress
            .lock()
            .map(|progress| progress.done)
            .unwrap_or_default();
        install::run_tasks(game_dir, tasks, 8, progress, cancel)?;
        let after = progress
            .lock()
            .map(|progress| progress.done)
            .unwrap_or_default();
        after - before
    };

    Ok(count)
}

/// Reads an already downloaded index from disk.
pub fn load_index(game_dir: &Path, id: &str) -> Result<AssetIndex, CoreError> {
    let path = game_dir
        .join("assets")
        .join("indexes")
        .join(format!("{id}.json"));
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

/// Number of objects present on disk for this index.
pub fn present(game_dir: &Path, id: &str) -> usize {
    let Ok(index) = load_index(game_dir, id) else {
        return 0;
    };
    index
        .objects
        .values()
        .filter(|object| game_dir.join(object.path()).is_file())
        .count()
}

/// Missing object count for this index.
pub fn missing(game_dir: &Path, id: &str) -> usize {
    let Ok(index) = load_index(game_dir, id) else {
        return 0;
    };
    let present = index
        .objects
        .values()
        .filter(|object| game_dir.join(object.path()).is_file())
        .count();
    index.objects.len().saturating_sub(present)
}

/// Shared progress helper used by tests.
#[allow(dead_code)]
fn default_progress() -> ProgressHandle {
    std::sync::Arc::new(Mutex::new(crate::install::Progress::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_paths_use_hash_prefix() {
        let object = AssetObject {
            hash: "abcdef0123".to_owned(),
            size: 12,
        };
        assert_eq!(object.path(), "assets/objects/ab/abcdef0123");
    }

    #[test]
    fn parses_index_with_legacy_flags() {
        let text = r#"{"virtual":true,"objects":{"a/b":{"hash":"1234","size":3}}}"#;
        let index: AssetIndex = serde_json::from_str(text).unwrap();
        assert!(index.r#virtual);
        assert_eq!(index.objects.len(), 1);
    }
}
