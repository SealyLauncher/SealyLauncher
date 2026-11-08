//! UI shell state and navigation routes.

use sealy_core::Bootstrap;
use sealy_core::config::{Language, Settings, ThemeName, save_settings};
use sealy_core::paths::LauncherPaths;

use crate::i18n;
use crate::kit::IconName;
use crate::theme::ThemeTokens;

/// Number of first-run wizard steps.
pub const OOBE_STEPS: usize = 3;

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
    /// Active first-run wizard step; `None` once the wizard is done.
    pub oobe_step: Option<usize>,
}

impl ShellState {
    /// Builds shell state from core bootstrap data.
    pub fn from_bootstrap(bootstrap: Bootstrap) -> Self {
        let first_run = !bootstrap.paths.settings_file().exists();
        Self {
            route: Route::Home,
            settings: bootstrap.settings,
            paths: bootstrap.paths,
            notice: None,
            oobe_step: first_run.then_some(0),
        }
    }

    /// Whether the first-run wizard is showing.
    pub fn in_oobe(&self) -> bool {
        self.oobe_step.is_some()
    }

    /// Shows the wizard from the first step.
    pub fn start_oobe(&mut self) {
        self.oobe_step = Some(0);
    }

    /// Advances one step, finishing the wizard after the last one.
    pub fn oobe_next(&mut self) {
        match self.oobe_step {
            Some(step) if step + 1 < OOBE_STEPS => self.oobe_step = Some(step + 1),
            Some(_) => {
                self.oobe_step = None;
                self.route = Route::Home;
                self.persist();
            }
            None => {}
        }
    }

    /// Returns to the previous step, if any.
    pub fn oobe_back(&mut self) {
        if let Some(step) = self.oobe_step {
            self.oobe_step = (step > 0).then(|| step - 1);
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

    /// Toggles background music and persists settings.
    pub fn set_audio(&mut self, enabled: bool) {
        self.settings.appearance.background_music = enabled;
        self.persist();
    }

    /// Changes the download source policy and persists settings.
    pub fn set_source(&mut self, policy: sealy_core::config::SourcePolicy) {
        self.settings.download.metadata_source = policy;
        self.settings.download.file_source = policy;
        self.persist();
    }

    /// Changes the maximum heap and persists settings.
    pub fn set_max_memory(&mut self, mb: u32) {
        self.settings.java.memory.max_mb = mb;
        self.persist();
    }

    /// Changes the motion preference and persists settings.
    pub fn set_motion(&mut self, motion: sealy_core::config::MotionSetting) {
        self.settings.motion = motion;
        self.persist();
    }

    fn persist(&mut self) {
        // Inside the wizard nothing is written until the user finishes, so an
        // interrupted first run still shows the wizard next time.
        if self.in_oobe() {
            return;
        }
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
