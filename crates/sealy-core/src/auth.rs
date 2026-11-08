//! Cached authentication state and token refresh.
//!
//! Refresh tokens are sensitive: they live in `SealyData/tokens.json` for now,
//! with DPAPI/Keychain wrapping planned before a stable release.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::accounts::{Account, AccountKind};
use crate::error::CoreError;
use crate::microsoft;

/// Tokens cached for one account, keyed by dashed UUID.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoredTokens {
    /// MSA refresh token.
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// Minecraft access token.
    #[serde(default)]
    pub minecraft_token: Option<String>,
    /// Expiry of the Minecraft token, seconds since the Unix epoch.
    #[serde(default)]
    pub expires_at: u64,
}

/// `SealyData/tokens.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenStore {
    /// Per-account tokens.
    #[serde(default)]
    pub entries: BTreeMap<String, StoredTokens>,
}

impl TokenStore {
    /// Loads the store, ignoring unreadable files.
    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// Writes the store.
    pub fn save(&self, path: &Path) -> Result<(), CoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Cached entry for an account.
    pub fn entry(&self, uuid: &str) -> Option<&StoredTokens> {
        self.entries.get(uuid)
    }

    /// Stores fresh tokens for an account.
    pub fn store(&mut self, uuid: &str, tokens: StoredTokens) {
        self.entries.insert(uuid.to_owned(), tokens);
    }

    /// Drops an account's tokens.
    pub fn forget(&mut self, uuid: &str) {
        self.entries.remove(uuid);
    }
}

/// Seconds since the Unix epoch.
pub fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|delta| delta.as_secs())
        .unwrap_or(0)
}

/// Returns a usable Minecraft access token for `account`, refreshing if needed.
///
/// Offline accounts never reach Minecraft services, so they get the dummy token
/// the game expects.
pub fn ensure_minecraft_token(
    store: &mut TokenStore,
    account: &Account,
    client_id: &str,
) -> Result<String, CoreError> {
    if account.kind != AccountKind::Microsoft {
        return Ok("0".to_owned());
    }

    let now = now_epoch();

    if let Some(cached) = store.entry(&account.uuid)
        && let Some(token) = &cached.minecraft_token
        && cached.expires_at > now + 60
    {
        return Ok(token.clone());
    }

    let refresh_token = store
        .entry(&account.uuid)
        .and_then(|entry| entry.refresh_token.clone())
        .ok_or_else(|| CoreError::Network("登录已过期，请重新登录 Microsoft".to_owned()))?;

    let msa = microsoft::refresh(client_id, &refresh_token)?;
    let (_, session) = microsoft::complete_login(&msa)?;

    store.store(
        &account.uuid,
        StoredTokens {
            refresh_token: msa.refresh_token.or(Some(refresh_token)),
            minecraft_token: Some(session.access_token.clone()),
            expires_at: now + session.expires_in,
        },
    );

    Ok(session.access_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_accounts_get_dummy_token() {
        let mut store = TokenStore::default();
        let account = Account::offline("Steve");
        assert_eq!(
            ensure_minecraft_token(&mut store, &account, "id").unwrap(),
            "0"
        );
    }

    #[test]
    fn cached_token_is_reused() {
        let mut store = TokenStore::default();
        let account = Account {
            name: "Alex".to_owned(),
            uuid: "u".to_owned(),
            kind: AccountKind::Microsoft,
        };
        store.store(
            "u",
            StoredTokens {
                refresh_token: Some("r".to_owned()),
                minecraft_token: Some("mc".to_owned()),
                expires_at: now_epoch() + 3_600,
            },
        );
        assert_eq!(
            ensure_minecraft_token(&mut store, &account, "id").unwrap(),
            "mc"
        );
    }
}
