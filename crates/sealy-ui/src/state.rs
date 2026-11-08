//! UI shell state and navigation routes.

use sealy_core::Bootstrap;
use sealy_core::config::{Language, Settings, ThemeName, save_settings};
use sealy_core::paths::LauncherPaths;

use crate::i18n;
use crate::kit::IconName;
use crate::theme::ThemeTokens;

/// Top-level pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Tile-based home.
    Home,
    /// Instance list.
    Instances,
    /// Version browser.
    Downloads,
    /// Account methods.
    Accounts,
    /// Settings.
    Settings,
}

impl Route {
    /// All routes in navigation order.
    pub const ALL: [Self; 5] = [
        Self::Home,
        Self::Instances,
        Self::Downloads,
        Self::Accounts,
        Self::Settings,
    ];

    /// Stable index used for element ids.
    pub fn index(self) -> usize {
        match self {
            Self::Home => 0,
            Self::Instances => 1,
            Self::Downloads => 2,
            Self::Accounts => 3,
            Self::Settings => 4,
        }
    }

    /// One-based index rendered in the page head.
    pub fn ordinal(self) -> &'static str {
        match self {
            Self::Home => "01",
            Self::Instances => "02",
            Self::Downloads => "03",
            Self::Accounts => "04",
            Self::Settings => "05",
        }
    }

    /// Navigation label.
    pub fn label(self, lang: Language) -> &'static str {
        match (self, lang) {
            (Self::Home, Language::ZhCn) => "主页",
            (Self::Home, Language::EnUs) => "Home",
            (Self::Instances, Language::ZhCn) => "实例",
            (Self::Instances, Language::EnUs) => "Instances",
            (Self::Downloads, Language::ZhCn) => "下载",
            (Self::Downloads, Language::EnUs) => "Downloads",
            (Self::Accounts, Language::ZhCn) => "账号",
            (Self::Accounts, Language::EnUs) => "Accounts",
            (Self::Settings, Language::ZhCn) => "设置",
            (Self::Settings, Language::EnUs) => "Settings",
        }
    }

    /// Lucide icon for the tile that opens this route.
    pub fn icon(self) -> IconName {
        match self {
            Self::Home => IconName::LayoutDashboard,
            Self::Instances => IconName::FolderClosed,
            Self::Downloads => IconName::Inbox,
            Self::Accounts => IconName::CircleUser,
            Self::Settings => IconName::Settings2,
        }
    }
}

/// Mutable shell state shared by the title bar and pages.
#[derive(Debug, Clone)]
pub struct ShellState {
    /// Current route.
    pub route: Route,
    /// Persisted settings.
    pub settings: Settings,
    /// Portable paths.
    pub paths: LauncherPaths,
    /// Optional toast.
    pub notice: Option<String>,
}

impl ShellState {
    /// Builds shell state from core bootstrap data.
    pub fn from_bootstrap(bootstrap: Bootstrap) -> Self {
        Self {
            route: Route::Home,
            settings: bootstrap.settings,
            paths: bootstrap.paths,
            notice: None,
        }
    }

    /// Active theme preset name.
    pub fn theme_name(&self) -> ThemeName {
        self.settings.appearance.theme
    }

    /// Resolved token values for the active preset.
    pub fn theme(&self) -> ThemeTokens {
        ThemeTokens::preset(self.theme_name())
    }

    /// Current language.
    pub fn language(&self) -> Language {
        self.settings.language
    }

    /// Switches the theme preset and persists settings.
    pub fn set_theme(&mut self, name: ThemeName) {
        if self.settings.appearance.theme == name {
            return;
        }
        self.settings.appearance.theme = name;
        self.persist();
    }

    /// Changes the language and persists settings.
    pub fn set_language(&mut self, language: Language) {
        self.settings.language = language;
        self.persist();
    }

    fn persist(&mut self) {
        match save_settings(&self.paths, &self.settings) {
            Ok(()) => self.notice = Some(i18n::toast_settings_saved(self.language()).to_owned()),
            Err(err) => {
                tracing::warn!(error = %err, "failed to save settings");
                self.notice = Some(err.to_string());
            }
        }
    }

    /// Shows a transient notice.
    pub fn notify(&mut self, message: impl Into<String>) {
        self.notice = Some(message.into());
    }
}
