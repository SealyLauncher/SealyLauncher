//! Accounts: offline profiles first, online providers later.

use std::path::Path;

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// How an account authenticates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    /// Local profile without a login.
    #[default]
    Offline,
    /// Microsoft account.
    Microsoft,
    /// Third-party authlib-injector account.
    Authlib,
}

/// One account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    /// Player name sent to the game.
    pub name: String,
    /// Player UUID, dashed lowercase.
    pub uuid: String,
    /// Authentication kind.
    #[serde(default)]
    pub kind: AccountKind,
    /// Yggdrasil API root, for authlib accounts.
    #[serde(default)]
    pub server: Option<String>,
}

impl Account {
    /// Builds an offline account with the derived UUID.
    pub fn offline(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            uuid: offline_uuid(name),
            kind: AccountKind::Offline,
            server: None,
        }
    }
}

/// `SealyData/accounts.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountBook {
    /// Selected account name.
    #[serde(default)]
    pub selected: Option<String>,
    /// Known accounts.
    #[serde(default)]
    pub accounts: Vec<Account>,
}

impl AccountBook {
    /// Loads the book, falling back to an empty one.
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
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Adds an account, replacing one with the same name.
    pub fn add(&mut self, account: Account) {
        self.accounts
            .retain(|existing| existing.name != account.name);
        self.selected = Some(account.name.clone());
        self.accounts.push(account);
    }

    /// Puts a removed account back at index, selecting it again if asked.
    pub fn restore(&mut self, account: Account, index: usize, select: bool) {
        let index = index.min(self.accounts.len());
        let name = account.name.clone();
        self.accounts.insert(index, account);
        if select {
            self.selected = Some(name);
        }
    }

    /// Removes an account by name.
    pub fn remove(&mut self, name: &str) {
        self.accounts.retain(|account| account.name != name);
        if self.selected.as_deref() == Some(name) {
            self.selected = self.accounts.first().map(|account| account.name.clone());
        }
    }

    /// Selects an account by name.
    pub fn select(&mut self, name: &str) {
        if self.accounts.iter().any(|account| account.name == name) {
            self.selected = Some(name.to_owned());
        }
    }

    /// The selected account.
    pub fn current(&self) -> Option<&Account> {
        let selected = self.selected.as_deref()?;
        self.accounts
            .iter()
            .find(|account| account.name == selected)
    }
}

/// Offline UUID derived from the player name, matching vanilla servers.
pub fn offline_uuid(name: &str) -> String {
    let digest = Md5::digest(format!("OfflinePlayer:{name}").as_bytes());

    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_keeps_position_and_selection() {
        let mut book = AccountBook::default();
        book.add(Account::offline("a"));
        book.add(Account::offline("b"));
        let removed = book.accounts[0].clone();
        book.remove("a");
        book.restore(removed, 0, true);
        assert_eq!(book.accounts[0].name, "a");
        assert_eq!(book.selected.as_deref(), Some("a"));
    }

    #[test]
    fn offline_uuid_is_stable_and_dashed() {
        let a = offline_uuid("Notch");
        let b = offline_uuid("Notch");
        assert_eq!(a, b);
        assert_eq!(a.len(), 36);
        assert_eq!(a.matches('-').count(), 4);

        // Documented vanilla vector for offline "Notch".
        assert_eq!(a, "b50ad385-829d-3141-a216-7e7d7539ba7f");
    }

    #[test]
    fn adding_replaces_same_name() {
        let mut book = AccountBook::default();
        book.add(Account::offline("Steve"));
        book.add(Account::offline("Steve"));
        assert_eq!(book.accounts.len(), 1);
        assert_eq!(book.selected.as_deref(), Some("Steve"));
    }

    #[test]
    fn removal_moves_selection() {
        let mut book = AccountBook::default();
        book.add(Account::offline("a"));
        book.add(Account::offline("b"));
        book.remove("b");
        assert_eq!(book.current().map(|a| a.name.as_str()), Some("a"));
    }
}
