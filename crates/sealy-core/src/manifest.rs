//! Minecraft version metadata.
//!
//! Only the manifest is handled here; per-version documents and asset indexes
//! arrive with the downloader.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::SourcePolicy;
use crate::error::CoreError;

/// Mirror that serves Mojang's manifest with Chinese CDN nodes.
const BMCLAPI_ROOT: &str = "https://bmclapi2.bangbang93.com";
/// Mojang's own metadata host.
const MOJANG_ROOT: &str = "https://piston-meta.mojang.com";

/// `version_manifest_v2.json`, trimmed to the fields the launcher needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Newest release and snapshot ids.
    pub latest: LatestVersions,
    /// Every known version, newest first.
    pub versions: Vec<VersionSummary>,
}

/// The two ids surfaced on the downloads page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestVersions {
    /// Newest release id.
    pub release: String,
    /// Newest snapshot id.
    pub snapshot: String,
}

/// One entry of the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionSummary {
    /// Version id such as `1.21.11`.
    pub id: String,
    /// Release channel.
    #[serde(rename = "type")]
    pub kind: VersionKind,
    /// ISO-8601 release timestamp.
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    /// URL of the per-version document.
    pub url: String,
}

/// Release channel of a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VersionKind {
    /// Stable release.
    Release,
    /// Development snapshot.
    Snapshot,
    /// Alpha build.
    OldAlpha,
    /// Beta build.
    OldBeta,
}

impl VersionKind {
    /// Whether this version is a snapshot or older experimental build.
    pub fn is_snapshot(self) -> bool {
        matches!(self, Self::Snapshot | Self::OldAlpha | Self::OldBeta)
    }

    /// Stable label for the UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Release => "release",
            Self::Snapshot => "snapshot",
            Self::OldAlpha => "old_alpha",
            Self::OldBeta => "old_beta",
        }
    }
}

impl VersionManifest {
    /// Releases only, newest first.
    pub fn releases(&self) -> impl Iterator<Item = &VersionSummary> {
        self.versions
            .iter()
            .filter(|v| v.kind == VersionKind::Release)
    }

    /// Snapshots only, newest first.
    pub fn snapshots(&self) -> impl Iterator<Item = &VersionSummary> {
        self.versions.iter().filter(|v| v.kind.is_snapshot())
    }

    /// Human-facing date, e.g. `2026-11-08`.
    pub fn date_of(version: &VersionSummary) -> &str {
        version.release_time.split('T').next().unwrap_or("")
    }
}

/// Fetches the version manifest, trying the configured source first.
pub fn fetch_manifest(policy: SourcePolicy) -> Result<VersionManifest, CoreError> {
    let roots: &[&str] = match policy {
        SourcePolicy::Auto => &[BMCLAPI_ROOT, MOJANG_ROOT],
        SourcePolicy::Bmclapi => &[BMCLAPI_ROOT, MOJANG_ROOT],
        SourcePolicy::Official => &[MOJANG_ROOT],
    };

    let mut last = None;
    for root in roots {
        match fetch_manifest_from(root) {
            Ok(manifest) => return Ok(manifest),
            Err(err) => {
                tracing::warn!(root, error = %err, "version manifest request failed");
                last = Some(err);
            }
        }
    }

    Err(last.unwrap_or_else(|| CoreError::Network("no source configured".to_owned())))
}

/// Fetches the manifest from one mirror root.
pub fn fetch_manifest_from(root: &str) -> Result<VersionManifest, CoreError> {
    let url = format!("{root}/mc/game/version_manifest_v2.json");

    // ureq's shortcut TLS config only exists for rustls, so the native backend
    // (SChannel on Windows, Security.framework on macOS) is wired by hand.
    let connector = ureq::native_tls::TlsConnector::new()
        .map_err(|err| CoreError::Network(format!("tls backend: {err}")))?;

    let agent = ureq::AgentBuilder::new()
        .tls_connector(std::sync::Arc::new(connector))
        .timeout_connect(Duration::from_secs(8))
        .timeout(Duration::from_secs(20))
        .build();

    let response = agent
        .get(&url)
        .call()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;
    let manifest: VersionManifest = response
        .into_json()
        .map_err(|err| CoreError::Network(format!("{url}: {err}")))?;

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "latest": { "release": "1.21.11", "snapshot": "26w01a" },
        "versions": [
            { "id": "1.21.11", "type": "release", "releaseTime": "2026-11-08T10:00:00+00:00", "url": "https://x/1" },
            { "id": "26w01a", "type": "snapshot", "releaseTime": "2026-11-01T10:00:00+00:00", "url": "https://x/2" },
            { "id": "b1.7.3", "type": "old_beta", "releaseTime": "2011-07-08T10:00:00+00:00", "url": "https://x/3" }
        ]
    }"#;

    #[test]
    fn parses_manifest_and_filters() {
        let manifest: VersionManifest = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(manifest.latest.release, "1.21.11");
        assert_eq!(manifest.releases().count(), 1);
        assert_eq!(manifest.snapshots().count(), 2);
        assert_eq!(
            VersionManifest::date_of(&manifest.versions[0]),
            "2026-11-08"
        );
    }
}
