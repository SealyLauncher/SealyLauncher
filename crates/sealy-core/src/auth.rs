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
    /// authlib-injector access token.
    #[serde(default)]
    pub authlib_access_token: Option<String>,
    /// authlib-injector client token.
    #[serde(default)]
    pub authlib_client_token: Option<String>,
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
    match account.kind {
        AccountKind::Offline => return Ok("0".to_owned()),
        AccountKind::Authlib => return authlib_token(store, account),
        AccountKind::Microsoft => {}
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
            ..Default::default()
        },
    );

    Ok(session.access_token)
}

/// Refreshes an authlib-injector session.
fn authlib_token(store: &mut TokenStore, account: &Account) -> Result<String, CoreError> {
    let server = account
        .server
        .clone()
        .ok_or_else(|| CoreError::Network("外置账号缺少服务器地址，请重新登录".to_owned()))?;

    let cached = store.entry(&account.uuid).cloned().unwrap_or_default();
    let Some(access) = cached.authlib_access_token.clone() else {
        return Err(CoreError::Network(
            "外置登录已过期，请在账号页重新登录".to_owned(),
        ));
    };
    let client = cached.authlib_client_token.unwrap_or_default();

    match crate::authlib::refresh(&server, &access, &client) {
        Ok(session) => {
            cached_authlib(store, account, &session);
            Ok(session.access_token)
        }
        Err(_) => Err(CoreError::Network(
            "外置登录已过期，请在账号页重新登录".to_owned(),
        )),
    }
}

/// Caches an authlib session for later launches.
pub fn cached_authlib(
    store: &mut TokenStore,
    account: &Account,
    session: &crate::authlib::Session,
) {
    let mut entry = store.entry(&account.uuid).cloned().unwrap_or_default();
    entry.authlib_access_token = Some(session.access_token.clone());
    entry.authlib_client_token = Some(session.client_token.clone());
    store.store(&account.uuid, entry);
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
            server: None,
        };
        store.store(
            "u",
            StoredTokens {
                refresh_token: Some("r".to_owned()),
                minecraft_token: Some("mc".to_owned()),
                expires_at: now_epoch() + 3_600,
                ..Default::default()
            },
        );
        assert_eq!(
            ensure_minecraft_token(&mut store, &account, "id").unwrap(),
            "mc"
        );
    }
}
