//! Fabric and Quilt installation from their metadata services.
//!
//! Both loaders publish a "profile" JSON that only contains the loader main
//! class and libraries, plus `inheritsFrom: <game version>`. Writing it under
//! `versions/<id>/<id>.json` is enough for the normal install flow to fetch the
//! loader libraries and to launch through version inheritance.

use std::fs;
use std::path::Path;

use serde_json::json;

use crate::error::CoreError;
use crate::net;

/// Which loader to install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderKind {
    /// Fabric.
    Fabric,
    /// Quilt.
    Quilt,
}

impl LoaderKind {
    /// Display name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Fabric => "Fabric",
            Self::Quilt => "Quilt",
        }
    }

    /// Lowercase id used in version ids.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Fabric => "fabric",
            Self::Quilt => "quilt",
        }
    }

    fn meta(self) -> &'static str {
        match self {
            Self::Fabric => "https://meta.fabricmc.net/v2",
            Self::Quilt => "https://meta.quiltmc.org/v3",
        }
    }
}

/// Result of a loader install.
#[derive(Debug, Clone)]
pub struct LoaderInstall {
    /// Version id written to disk.
    pub id: String,
    /// Minecraft version it extends.
    pub game_version: String,
    /// Loader version.
    pub loader_version: String,
}

/// Installs `kind` for `game` and returns the new version id.
///
/// The caller still has to run the normal install flow for that id to fetch the
/// loader libraries.
pub fn install(root: &Path, kind: LoaderKind, game: &str) -> Result<LoaderInstall, CoreError> {
    let loader_version = latest_loader(kind, game)?;
    let url = format!(
        "{}/versions/loader/{game}/{loader_version}/profile/json",
        kind.meta()
    );
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Err(CoreError::Network(format!(
            "{} 还没有 {} 的安装配置（HTTP {}）",
            game,
            kind.label(),
            response.status
        )));
    }

    let mut profile = response.body;
    let id = profile["id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}-loader-{loader_version}-{game}", kind.slug()));
    profile["id"] = json!(id);
    profile["inheritsFrom"] = json!(game);

    if profile["libraries"].as_array().is_none_or(Vec::is_empty) {
        return Err(CoreError::Network(format!(
            "{} 的配置里没有库文件，已中止",
            kind.label()
        )));
    }

    let directory = root.join("versions").join(&id);
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join(format!("{id}.json")),
        serde_json::to_string_pretty(&profile)?,
    )?;

    Ok(LoaderInstall {
        id,
        game_version: game.to_owned(),
        loader_version,
    })
}

/// Newest loader version for `game`.
fn latest_loader(kind: LoaderKind, game: &str) -> Result<String, CoreError> {
    let url = format!("{}/versions/loader/{game}", kind.meta());
    let response = net::get_json(&url, None)?;
    if !response.is_ok() {
        return Err(CoreError::Network(format!(
            "{} 还没有 {} 的构建（HTTP {}）",
            game,
            kind.label(),
            response.status
        )));
    }

    let list = response
        .body
        .as_array()
        .ok_or_else(|| CoreError::Network(format!("{url}: 返回的不是数组")))?;

    // Fabric nests the version under `loader`, Quilt puts it at the top level.
    let version_of = |entry: &serde_json::Value| {
        entry
            .pointer("/loader/version")
            .and_then(|value| value.as_str())
            .or_else(|| entry.get("version").and_then(|value| value.as_str()))
            .map(str::to_owned)
    };

    let stable = list.iter().find(|entry| {
        version_of(entry).is_some()
            && entry
                .pointer("/loader/stable")
                .and_then(|value| value.as_bool())
                .unwrap_or(true)
    });

    stable
        .or_else(|| list.iter().find(|entry| version_of(entry).is_some()))
        .and_then(version_of)
        .ok_or_else(|| CoreError::Network(format!("{} 没有可用的版本列表", kind.label())))
}

#[cfg(test)]
mod tests {

    #[test]
    fn fabric_list_shape() {
        let body = serde_json::json!([
            { "loader": { "version": "0.16.9", "stable": false } },
            { "loader": { "version": "0.16.5", "stable": true } }
        ]);
        let list = body.as_array().unwrap();
        let version = list
            .iter()
            .find(|entry| {
                entry
                    .pointer("/loader/stable")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(true)
            })
            .and_then(|entry| entry.pointer("/loader/version"))
            .and_then(|value| value.as_str());
        assert_eq!(version, Some("0.16.5"));
    }

    #[test]
    fn quilt_list_shape() {
        let body = serde_json::json!([{ "version": "0.24.0", "build": 3 }]);
        let version = body
            .as_array()
            .unwrap()
            .first()
            .and_then(|entry| entry.get("version"))
            .and_then(|value| value.as_str());
        assert_eq!(version, Some("0.24.0"));
    }
}
