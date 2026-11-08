//! Instance bookkeeping: which versions the player can launch, and where their
//! game directories live.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// One launchable instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    /// Display name, also the directory name under `instances/`.
    pub name: String,
    /// Version id this instance runs.
    pub version_id: String,
    /// Last launch time, seconds since the Unix epoch.
    #[serde(default)]
    pub last_played: Option<u64>,
}

/// `SealyData/instances.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InstanceBook {
    /// Currently selected instance name.
    #[serde(default)]
    pub selected: Option<String>,
    /// All known instances, oldest first.
    #[serde(default)]
    pub instances: Vec<Instance>,
}

impl InstanceBook {
    /// Loads the book, falling back to an empty one when missing or corrupt.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// Writes the book.
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(path, text)?;
        Ok(())
    }

    /// Creates an instance for `version_id` with an unused default name.
    pub fn create(&mut self, version_id: &str) -> Instance {
        let name = self.next_name(version_id);
        let instance = Instance {
            name: name.clone(),
            version_id: version_id.to_owned(),
            last_played: None,
        };
        self.instances.push(instance.clone());
        self.selected = Some(name);
        instance
    }

    /// Removes an instance by name.
    pub fn remove(&mut self, name: &str) {
        self.instances.retain(|instance| instance.name != name);
        if self.selected.as_deref() == Some(name) {
            self.selected = self.instances.last().map(|instance| instance.name.clone());
        }
    }

    /// Selects an instance by name.
    pub fn select(&mut self, name: &str) {
        if self.instances.iter().any(|instance| instance.name == name) {
            self.selected = Some(name.to_owned());
        }
    }

    /// The currently selected instance.
    pub fn current(&self) -> Option<&Instance> {
        let selected = self.selected.as_deref()?;
        self.instances
            .iter()
            .find(|instance| instance.name == selected)
    }

    /// Records a launch timestamp (epoch seconds).
    pub fn touch(&mut self, name: &str, stamp: u64) {
        if let Some(instance) = self
            .instances
            .iter_mut()
            .find(|instance| instance.name == name)
        {
            instance.last_played = Some(stamp);
        }
    }

    /// A free name derived from the version id.
    fn next_name(&self, version_id: &str) -> String {
        let base = version_id.to_owned();
        if !self.instances.iter().any(|instance| instance.name == base) {
            return base;
        }
        let mut index = 2;
        loop {
            let candidate = format!("{base} ({index})");
            if !self
                .instances
                .iter()
                .any(|instance| instance.name == candidate)
            {
                return candidate;
            }
            index += 1;
        }
    }
}

/// Game directory of one instance.
pub fn directory(game_dir: &Path, name: &str) -> PathBuf {
    game_dir.join("instances").join(sanitize(name))
}

/// Keeps instance names safe as directory names.
fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            other => other,
        })
        .collect();
    let trimmed = cleaned.trim().trim_end_matches('.').trim();
    if trimmed.is_empty() {
        "instance".to_owned()
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_unique_names() {
        let mut book = InstanceBook::default();
        assert_eq!(book.create("1.21.11").name, "1.21.11");
        assert_eq!(book.create("1.21.11").name, "1.21.11 (2)");
        assert_eq!(book.instances.len(), 2);
        assert_eq!(book.selected.as_deref(), Some("1.21.11 (2)"));
    }

    #[test]
    fn removal_moves_selection() {
        let mut book = InstanceBook::default();
        book.create("a");
        book.create("b");
        book.remove("b");
        assert_eq!(
            book.current().map(|instance| instance.name.as_str()),
            Some("a")
        );
    }

    #[test]
    fn sanitizes_directory_names() {
        assert_eq!(sanitize("a/b:c"), "a_b_c");
        assert_eq!(sanitize("   "), "instance");
    }

    #[test]
    fn round_trips_through_json() {
        let mut book = InstanceBook::default();
        book.create("1.20.1");
        let text = serde_json::to_string(&book).unwrap();
        let parsed: InstanceBook = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed.instances, book.instances);
    }
}
