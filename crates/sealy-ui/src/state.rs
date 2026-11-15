//! UI shell state and navigation routes.

use sealy_core::Bootstrap;
use sealy_core::config::{Language, MemoryMode, Settings, ThemeName, save_settings};
use sealy_core::paths::LauncherPaths;

use crate::i18n;
use crate::kit::{Entity, IconName, SliderState};
use crate::theme::ThemeTokens;

/// Number of first-run wizard steps.
pub const OOBE_STEPS: usize = 4;

/// Wizard step that requires an account.
pub const OOBE_ACCOUNT_STEP: usize = 2;

/// Which versions the downloads page lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VersionFilter {
    /// Release and snapshot builds.
    #[default]
    All,
    /// Stable releases only.
    Release,
    /// Snapshots and older experimental builds.
    Snapshot,
}

impl VersionFilter {
    /// Next filter in the toggle order.
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Release,
            Self::Release => Self::Snapshot,
            Self::Snapshot => Self::All,
        }
    }
}

/// Remote data shown on the downloads page.
#[derive(Debug, Clone, Default)]
pub struct DownloadsState {
    /// Fetched manifest, once available.
    pub manifest: Option<sealy_core::manifest::VersionManifest>,
    /// Last fetch failure, shown instead of the list.
    pub error: Option<String>,
    /// Whether a fetch is in flight.
    pub loading: bool,
    /// Active kind filter.
    pub filter: VersionFilter,
    /// Version picked for a loader install.
    pub choice: Option<String>,
    /// Rows currently materialised by the infinite scroll list.
    pub limit: usize,
}

/// Live view of the running install.
#[derive(Debug, Clone)]
pub struct InstallView {
    /// Version being installed.
    pub id: String,
    /// Files finished.
    pub done: usize,
    /// Files planned; zero until the metadata arrives.
    pub total: usize,
    /// File currently fetching.
    pub current: String,
    /// Whether the worker stopped.
    pub finished: bool,
    /// Failure message.
    pub error: Option<String>,
}

impl InstallView {
    /// Completion in `0.0..=1.0`.
    pub fn ratio(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.done as f32 / self.total as f32).clamp(0.0, 1.0)
        }
    }
}

/// Tone of a message center row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTone {
    /// Work in progress.
    Running,
    /// Finished successfully.
    Done,
    /// Finished with an error.
    Failed,
}

/// One row in the message center.
#[derive(Debug, Clone)]
pub struct Message {
    /// Stable id.
    pub id: u64,
    /// Headline.
    pub title: String,
    /// Secondary line, updated live for tasks.
    pub detail: String,
    /// Progress in `0.0..=1.0` for running tasks.
    pub progress: Option<f32>,
    /// Tone.
    pub tone: MessageTone,
    /// Version id a failed install can retry.
    pub retry: Option<String>,
    /// Cancellation flag of the task behind this message.
    pub cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

/// A transient message shown in the bottom-right corner.
#[derive(Debug, Clone)]
pub struct Toast {
    /// Id used to ignore stale expiry timers.
    pub id: u64,
    /// Text shown to the user.
    pub message: String,
    /// Optional action button.
    pub action: Option<ToastAction>,
}

/// Action attached to a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastAction {
    /// Restore the item that was just deleted.
    Undo,
}

/// Item removed moments ago, kept until the toast expires.
#[derive(Debug, Clone)]
pub enum PendingUndo {
    /// A deleted instance.
    Instance {
        /// The removed entry.
        instance: sealy_core::instances::Instance,
        /// Position it had in the book.
        index: usize,
        /// Whether it was the selected entry.
        selected: bool,
    },
    /// A deleted account.
    Account {
        /// The removed entry.
        account: sealy_core::accounts::Account,
        /// Position it had in the book.
        index: usize,
        /// Whether it was the selected entry.
        selected: bool,
    },
}

/// Handle to a running install so the UI can poll and cancel it.
#[derive(Debug, Clone)]
pub struct InstallTask {
    /// Message row this task reports into.
    pub message_id: u64,
    /// Shared progress written by the workers.
    pub progress: sealy_core::install::ProgressHandle,
    /// Cooperative cancel flag.
    pub cancel: sealy_core::install::CancelFlag,
}

/// Top-level pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Tile-based home.
    Home,
    /// Instance list.
    Instances,
    /// Detail page of the selected instance.
    Instance,
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
            Self::Instances | Self::Instance => 1,
            Self::Downloads => 2,
            Self::Accounts => 3,
            Self::Settings => 4,
        }
    }

    /// One-based index rendered in the page head.
    pub fn ordinal(self) -> &'static str {
        match self {
            Self::Home => "01",
            Self::Instances | Self::Instance => "02",
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
            (Self::Instance, Language::ZhCn) => "实例详情",
            (Self::Instance, Language::EnUs) => "Instance",
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
            Self::Instances | Self::Instance => IconName::FolderClosed,
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
    /// Data lives under the per-user directory because the portable one failed.
    pub data_fallback: bool,
    /// macOS is running the bundle from an App Translocation path.
    pub translocated: bool,
    /// Optional toast with an optional undo action.
    pub toast: Option<Toast>,
    /// Id source for toasts.
    pub next_toast_id: u64,
    /// When the visible toast should disappear.
    pub toast_deadline: Option<std::time::Instant>,
    /// Item deleted in the last few seconds, restorable from the toast.
    pub pending_undo: Option<PendingUndo>,
    /// Active first-run wizard step; `None` once the wizard is done.
    pub oobe_step: Option<usize>,
    /// Downloads page data.
    pub downloads: DownloadsState,
    /// Running install, if any.
    pub install: Option<InstallTask>,
    /// Message center rows, newest first.
    pub messages: Vec<Message>,
    /// Whether the message panel is open.
    pub message_open: bool,
    /// Monotonic id for messages.
    pub next_message_id: u64,
    /// Java installations found on this machine.
    pub java: Vec<sealy_core::java::JavaInstall>,
    /// Versions installed under `<game>/versions`.
    pub installed: Vec<sealy_core::version::InstalledVersion>,
    /// Known instances and the current selection.
    pub instances: sealy_core::instances::InstanceBook,
    /// Known accounts and the current selection.
    pub accounts: sealy_core::accounts::AccountBook,
    /// Inline account name buffer while adding an offline account.
    pub account_draft: Option<String>,
    /// Whether the new-instance version picker is open.
    pub new_instance_open: bool,
    /// Choices made in the new-instance picker.
    pub new_instance: NewInstanceState,
    /// Inline instance rename buffer.
    pub rename_draft: Option<RenameDraft>,
    /// authlib-injector login form.
    pub authlib_draft: Option<AuthlibDraft>,
    /// Inline input for adding a Java executable.
    pub java_path_draft: Option<String>,
    /// Cached authentication tokens.
    pub tokens: sealy_core::auth::TokenStore,
    /// Pending Microsoft device-code sign-in.
    pub microsoft_login: Option<MicrosoftLogin>,
    /// Whether the per-instance launch settings panel is open.
    pub instance_settings_open: bool,
    /// Draft values while editing the selected instance's launch settings.
    pub inst_launch_draft: Option<InstanceLaunchDraft>,
    /// Whether the game-log viewer is open.
    pub logs_open: bool,
    /// Header of the log viewer.
    pub logs_title: String,
    /// File currently shown by the log viewer.
    pub logs_path: Option<std::path::PathBuf>,
    /// Last lines read from [`logs_path`](Self::logs_path).
    pub logs_lines: Vec<String>,
    /// Last instance row click, used to detect double clicks.
    pub last_instance_click: Option<(String, std::time::Instant)>,
    /// Whether the delete-instance confirmation is open.
    pub delete_confirm_open: bool,
    /// Sidebar section shown on the instance detail page.
    pub instance_section: InstanceSection,
    /// Selected settings category in the settings sidebar.
    pub settings_section: SettingsSection,
    /// Slider bound to the game memory cap.
    pub memory_slider: Option<Entity<SliderState>>,
    /// Inline numeric editor for the memory cap.
    pub memory_draft: Option<String>,
    /// Physical memory detected at startup, in MiB.
    pub total_ram_mb: Option<u64>,
    /// Inline proxy editor in the settings page.
    pub proxy_draft: Option<String>,
    /// Inline game-directory editor in the settings page.
    pub game_dir_draft: Option<String>,
    /// Whether the crash-analysis panel is open.
    pub crash_open: bool,
    /// Latest crash analysis, when one was requested.
    pub crash_view: Option<sealy_core::crash::CrashReport>,
    /// Result of the last self-update check.
    pub update_check: Option<Result<Option<sealy_core::update::ReleaseInfo>, String>>,
    /// Whether the update archive is being downloaded.
    pub update_downloading: bool,
    /// Live byte progress of the update download.
    pub update_progress: Option<UpdateProgress>,
    /// Cancellation flag for the running update download.
    pub update_cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Last update failure, shown with a retry button.
    pub update_error: Option<String>,
    /// Warning shown once after a previous update did not finish.
    pub update_warning: Option<String>,
}

/// Byte progress of the self-update download.
#[derive(Debug, Clone)]
pub struct UpdateProgress {
    /// Bytes written so far.
    pub done: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Expected size from the release asset.
    pub total: u64,
}

/// Sidebar sections of the instance detail page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InstanceSection {
    /// Overview, launch settings and icon.
    #[default]
    Overview,
    /// Installed mods.
    Mods,
    /// Resource packs.
    ResourcePacks,
    /// Shader packs.
    Shaders,
}

impl InstanceSection {
    /// Every section, in sidebar order.
    pub const ALL: [Self; 4] = [
        Self::Overview,
        Self::Mods,
        Self::ResourcePacks,
        Self::Shaders,
    ];

    /// Sidebar label.
    pub fn label(self, lang: sealy_core::config::Language) -> &'static str {
        use crate::i18n::tr;
        match self {
            Self::Overview => tr(lang, "概览", "Overview"),
            Self::Mods => tr(lang, "模组", "Mods"),
            Self::ResourcePacks => tr(lang, "资源包", "Resource packs"),
            Self::Shaders => tr(lang, "光影", "Shaders"),
        }
    }
}

/// Category shown in the settings sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    /// Theme, language, scale, motion.
    #[default]
    Appearance,
    /// Download sources.
    Downloads,
    /// Java runtimes and the heap cap.
    JavaMemory,
    /// Proxy and game folder.
    Network,
    /// Update, version and maintenance.
    Launcher,
}

impl SettingsSection {
    /// Every section, in sidebar order.
    pub const ALL: [Self; 5] = [
        Self::Appearance,
        Self::Downloads,
        Self::JavaMemory,
        Self::Network,
        Self::Launcher,
    ];

    /// Sidebar label.
    pub fn label(self, lang: sealy_core::config::Language) -> &'static str {
        use crate::i18n::tr;
        match self {
            Self::Appearance => tr(lang, "外观", "Appearance"),
            Self::Downloads => tr(lang, "下载", "Downloads"),
            Self::JavaMemory => tr(lang, "Java 与内存", "Java & memory"),
            Self::Network => tr(lang, "网络与目录", "Network & folders"),
            Self::Launcher => tr(lang, "启动器", "Launcher"),
        }
    }
}
/// Choices made in the new-instance picker.
#[derive(Debug, Clone, Default)]
pub struct NewInstanceState {
    /// Installed version the instance is based on.
    pub game: Option<String>,
    /// Loader to install, or `None` for a vanilla instance.
    pub loader: Option<sealy_core::loader::LoaderKind>,
    /// Loader build picked by the user; `None` means the newest stable one.
    pub loader_version: Option<String>,
    /// Loader builds fetched for the selected game version.
    pub loader_choices: Vec<sealy_core::loader::LoaderChoice>,
    /// Whether the loader build list is being fetched.
    pub loader_loading: bool,
    /// Failure while fetching loader builds.
    pub loader_error: Option<String>,
    /// Local OptiFine installer picked by the user.
    pub optifine_jar: Option<std::path::PathBuf>,
}

impl NewInstanceState {
    /// Clears the loader picks; used when the base version changes.
    pub fn reset_loaders(&mut self) {
        self.loader = None;
        self.loader_version = None;
        self.loader_choices.clear();
        self.loader_loading = false;
        self.loader_error = None;
        self.optifine_jar = None;
    }
}
/// Editable copy of one instance's launch overrides.
#[derive(Debug, Clone, Default)]
pub struct InstanceLaunchDraft {
    /// Window width in pixels.
    pub width: String,
    /// Window height in pixels.
    pub height: String,
    /// Maximum heap in MiB.
    pub max_mb: String,
    /// Extra JVM arguments.
    pub jvm_args: String,
    /// Field currently receiving keystrokes.
    pub field: usize,
}

impl InstanceLaunchDraft {
    /// Labels of the editable fields, in order.
    pub const FIELDS: [&'static str; 4] = ["窗口宽度", "窗口高度", "最大内存 (MB)", "JVM 参数"];
}

/// authlib-injector login form state.
#[derive(Debug, Clone, Default)]
pub struct AuthlibDraft {
    /// Yggdrasil API root.
    pub server: String,
    /// Account name.
    pub username: String,
    /// Password (masked in the UI).
    pub password: String,
    /// Focused field: 0 server, 1 user, 2 password.
    pub field: usize,
}

/// Inline rename of one instance.
#[derive(Debug, Clone)]
pub struct RenameDraft {
    /// Current name on disk.
    pub old: String,
    /// Buffer being edited.
    pub buffer: String,
}

/// Device-code sign-in waiting for the user.
#[derive(Debug, Clone)]
pub struct MicrosoftLogin {
    /// Message row this login reports into.
    pub message_id: u64,
    /// Code the user types on microsoft.com/link.
    pub user_code: String,
    /// Page to open.
    pub verification_uri: String,
    /// Cooperative cancel flag for the polling task.
    pub cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Whether the code was copied for the user already.
    pub code_copied: bool,
}

impl ShellState {
    /// Builds shell state from core bootstrap data.
    pub fn from_bootstrap(bootstrap: Bootstrap) -> Self {
        let first_run = !bootstrap.paths.settings_file().exists();
        let data_fallback = bootstrap.data_fallback;
        let translocated = bootstrap.translocated;
        let instances =
            sealy_core::instances::InstanceBook::load(&bootstrap.paths.instances_file());
        let accounts = sealy_core::accounts::AccountBook::load(&bootstrap.paths.accounts_file());
        let tokens = sealy_core::auth::TokenStore::load(&bootstrap.paths.tokens_file());
        let update_warning =
            sealy_core::update::reconcile_pending(&bootstrap.paths.data, sealy_core::version());
        Self {
            route: Route::Home,
            settings: bootstrap.settings,
            paths: bootstrap.paths,
            data_fallback,
            translocated,
            toast: None,
            next_toast_id: 1,
            toast_deadline: None,
            pending_undo: None,
            oobe_step: first_run.then_some(0),
            downloads: DownloadsState {
                limit: 60,
                ..DownloadsState::default()
            },
            install: None,
            messages: Vec::new(),
            message_open: false,
            next_message_id: 1,
            java: Vec::new(),
            installed: Vec::new(),
            instances,
            accounts,
            account_draft: None,
            new_instance_open: false,
            new_instance: NewInstanceState::default(),
            rename_draft: None,
            authlib_draft: None,
            java_path_draft: None,
            tokens,
            microsoft_login: None,
            instance_settings_open: false,
            inst_launch_draft: None,
            logs_open: false,
            logs_title: String::new(),
            logs_path: None,
            logs_lines: Vec::new(),
            last_instance_click: None,
            delete_confirm_open: false,
            instance_section: InstanceSection::default(),
            settings_section: SettingsSection::default(),
            memory_slider: None,
            memory_draft: None,
            total_ram_mb: sealy_core::system::total_memory_mb(),
            proxy_draft: None,
            game_dir_draft: None,
            crash_open: false,
            crash_view: None,
            update_check: None,
            update_downloading: false,
            update_progress: None,
            update_cancel: None,
            update_error: None,
            update_warning,
        }
    }

    /// Whether the first-run wizard is showing.
    pub fn in_oobe(&self) -> bool {
        self.oobe_step.is_some()
    }

    /// Opens the new-instance picker with a clean slate.
    pub fn begin_new_instance(&mut self) {
        self.new_instance = NewInstanceState::default();
        self.new_instance_open = true;
    }

    /// Picks the Minecraft version an instance is based on.
    pub fn pick_new_instance_game(&mut self, game: &str) {
        if self.new_instance.game.as_deref() == Some(game) {
            return;
        }
        self.new_instance.game = Some(game.to_owned());
        self.new_instance.reset_loaders();
    }

    /// Picks the loader to install, keeping any previous version choice.
    pub fn set_new_instance_loader(&mut self, loader: Option<sealy_core::loader::LoaderKind>) {
        if self.new_instance.loader == loader {
            return;
        }
        self.new_instance.loader = loader;
        self.new_instance.loader_version = None;
        self.new_instance.loader_error = None;
    }
    /// Persists the instance book.
    pub fn save_instances(&mut self) {
        if let Err(err) = self.instances.save(&self.paths.instances_file()) {
            tracing::warn!(error = %err, "failed to save instances");
        }
    }

    /// Persists the account book.
    pub fn save_accounts(&mut self) {
        if let Err(err) = self.accounts.save(&self.paths.accounts_file()) {
            tracing::warn!(error = %err, "failed to save accounts");
        }
    }

    /// Persists cached tokens.
    pub fn save_tokens(&mut self) {
        if let Err(err) = self.tokens.save(&self.paths.tokens_file()) {
            tracing::warn!(error = %err, "failed to save tokens");
        }
    }

    /// Opens the inline name editor for a new offline account.
    pub fn begin_account(&mut self) {
        self.account_draft = Some(String::new());
    }

    /// Feeds one typed character into the draft.
    pub fn push_account_char(&mut self, ch: char) {
        if let Some(draft) = &mut self.account_draft
            && draft.chars().count() < 16
            && (ch.is_alphanumeric() || ch == '_')
        {
            draft.push(ch);
        }
    }

    /// Removes the last character of the draft.
    pub fn pop_account_char(&mut self) {
        if let Some(draft) = &mut self.account_draft {
            draft.pop();
        }
    }

    /// Commits the draft as an offline account.
    pub fn commit_account(&mut self) {
        let Some(draft) = self.account_draft.take() else {
            return;
        };
        let name = draft.trim().to_owned();
        if name.is_empty() {
            return;
        }
        let mut chars = name.chars();
        let display = match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => name,
        };
        self.accounts
            .add(sealy_core::accounts::Account::offline(&display));
        self.save_accounts();
    }

    /// Closes the draft without creating anything.
    pub fn cancel_account(&mut self) {
        self.account_draft = None;
    }

    /// Removes the named account.
    pub fn remove_account(&mut self, name: &str) -> Option<PendingUndo> {
        let index = self
            .accounts
            .accounts
            .iter()
            .position(|item| item.name == name)?;
        let was_selected = self.accounts.selected.as_deref() == Some(name);
        let account = self.accounts.accounts.remove(index);
        if was_selected {
            self.accounts.selected = None;
        }
        self.save_accounts();
        Some(PendingUndo::Account {
            account,
            index,
            selected: was_selected,
        })
    }

    /// Selects the named account.
    pub fn select_account(&mut self, name: &str) {
        self.accounts.select(name);
        self.save_accounts();
    }

    /// Creates an instance for `version_id` and selects it.
    pub fn create_instance(&mut self, version_id: &str) -> String {
        let instance = self.instances.create(version_id);
        let dir = sealy_core::instances::directory(&self.paths.game, &instance.name);
        let _ = std::fs::create_dir_all(&dir);
        self.save_instances();
        instance.name
    }

    /// Removes the named instance.
    pub fn remove_instance(&mut self, name: &str) -> Option<PendingUndo> {
        let index = self
            .instances
            .instances
            .iter()
            .position(|item| item.name == name)?;
        let was_selected = self.instances.selected.as_deref() == Some(name);
        let instance = self.instances.instances.remove(index);
        if was_selected {
            self.instances.selected = None;
        }
        self.save_instances();
        Some(PendingUndo::Instance {
            instance,
            index,
            selected: was_selected,
        })
    }

    /// Selects the named instance.
    pub fn select_instance(&mut self, name: &str) {
        self.instances.select(name);
        self.save_instances();
    }

    /// Opens the rename editor for the current instance.
    pub fn begin_rename(&mut self) {
        if let Some(instance) = self.instances.current() {
            self.rename_draft = Some(RenameDraft {
                old: instance.name.clone(),
                buffer: instance.name.clone(),
            });
        }
    }

    /// Feeds one character into the rename buffer.
    pub fn push_rename_char(&mut self, ch: char) {
        if let Some(draft) = &mut self.rename_draft
            && draft.buffer.chars().count() < 48
        {
            draft.buffer.push(ch);
        }
    }

    /// Removes the last rename character.
    pub fn pop_rename_char(&mut self) {
        if let Some(draft) = &mut self.rename_draft {
            draft.buffer.pop();
        }
    }

    /// Commits the rename, moving the instance directory too.
    pub fn commit_rename(&mut self) {
        let Some(draft) = self.rename_draft.take() else {
            return;
        };
        let new_name = draft.buffer.trim().to_owned();
        if new_name.is_empty() || new_name == draft.old {
            return;
        }
        if !self.instances.rename(&draft.old, &new_name) {
            return;
        }

        let root = self.paths.game.clone();
        let from = sealy_core::instances::directory(&root, &draft.old);
        let to = sealy_core::instances::directory(&root, &new_name);
        if from.is_dir() && !to.exists() {
            let _ = std::fs::rename(&from, &to);
        }
        self.save_instances();
    }

    /// Feeds one character into the focused authlib field.
    /// Adds a probed Java install, replacing an entry with the same path.
    pub fn add_java(&mut self, install: sealy_core::java::JavaInstall) {
        self.java.retain(|existing| existing.path != install.path);
        self.java.push(install);
        self.java.sort_by_key(|java| std::cmp::Reverse(java.major));
    }

    /// Sets (or clears) the user-selected default Java.
    pub fn set_default_java(&mut self, path: Option<&std::path::Path>) {
        self.settings.java.default_path = path.map(|path| path.display().to_string());
        self.persist();
    }

    /// Opens the inline "add Java" input.
    pub fn begin_add_java(&mut self) {
        self.java_path_draft = Some(String::new());
    }

    /// Opens the launch settings of the selected instance.
    pub fn begin_instance_settings(&mut self) {
        let Some(instance) = self.instances.current() else {
            return;
        };
        let launch = instance.launch.clone();
        self.inst_launch_draft = Some(InstanceLaunchDraft {
            width: launch
                .width
                .map(|value| value.to_string())
                .unwrap_or_default(),
            height: launch
                .height
                .map(|value| value.to_string())
                .unwrap_or_default(),
            max_mb: launch
                .max_mb
                .map(|value| value.to_string())
                .unwrap_or_default(),
            jvm_args: launch.jvm_args,
            field: 0,
        });
        self.instance_settings_open = true;
    }

    /// Appends a character to the focused instance-settings field.
    pub fn inst_draft_push_char(&mut self, ch: char) {
        let Some(draft) = &mut self.inst_launch_draft else {
            return;
        };
        let field = match draft.field {
            0 => &mut draft.width,
            1 => &mut draft.height,
            2 => &mut draft.max_mb,
            _ => &mut draft.jvm_args,
        };
        if field.chars().count() < 400 {
            field.push(ch);
        }
    }

    /// Removes the last character from the focused field.
    pub fn inst_draft_pop_char(&mut self) {
        let Some(draft) = &mut self.inst_launch_draft else {
            return;
        };
        let field = match draft.field {
            0 => &mut draft.width,
            1 => &mut draft.height,
            2 => &mut draft.max_mb,
            _ => &mut draft.jvm_args,
        };
        field.pop();
    }

    /// Moves to the next instance-settings field.
    pub fn inst_draft_next_field(&mut self) {
        if let Some(draft) = &mut self.inst_launch_draft {
            draft.field = (draft.field + 1) % InstanceLaunchDraft::FIELDS.len();
        }
    }

    /// Validates and stores the edited launch settings.
    pub fn commit_instance_settings(&mut self) -> Result<(), String> {
        let Some(draft) = self.inst_launch_draft.take() else {
            return Ok(());
        };
        self.instance_settings_open = false;

        let parse = |raw: &str, label: &str| -> Result<Option<u32>, String> {
            let text = raw.trim();
            if text.is_empty() {
                return Ok(None);
            }
            text.parse::<u32>()
                .map(Some)
                .map_err(|_| format!("{label}不是有效数字"))
        };
        let width = parse(&draft.width, "窗口宽度")?;
        let height = parse(&draft.height, "窗口高度")?;
        let max_mb = parse(&draft.max_mb, "最大内存")?;

        if let Some(value) = width
            && !(320..=7680).contains(&value)
        {
            return Err("窗口宽度需在 320–7680 之间".to_owned());
        }
        if let Some(value) = height
            && !(240..=4320).contains(&value)
        {
            return Err("窗口高度需在 240–4320 之间".to_owned());
        }
        if let Some(value) = max_mb
            && !(256..=65536).contains(&value)
        {
            return Err("最大内存需在 256–65536 MB 之间".to_owned());
        }

        let Some(name) = self
            .instances
            .current()
            .map(|instance| instance.name.clone())
        else {
            return Ok(());
        };
        self.instances.set_launch(
            &name,
            sealy_core::instances::LaunchOverrides {
                width,
                height,
                max_mb,
                jvm_args: draft.jvm_args.trim().to_owned(),
            },
        );
        self.save_instances();
        self.show_toast("实例启动设置已保存", None);
        Ok(())
    }

    /// Directory of the selected instance inside the game folder.
    pub fn current_instance_dir(&self) -> Option<std::path::PathBuf> {
        self.instances
            .current()
            .map(|instance| sealy_core::instances::directory(&self.paths.game, &instance.name))
    }

    /// Sets or clears the icon key of the selected instance.
    pub fn set_instance_icon(&mut self, key: Option<&str>) {
        let Some(name) = self
            .instances
            .current()
            .map(|instance| instance.name.clone())
        else {
            return;
        };
        self.instances.set_icon(&name, key.map(str::to_owned));
        self.save_instances();
    }

    /// Deletes the selected instance, optionally removing its game directory.
    pub fn delete_selected_instance(&mut self, delete_files: bool) {
        self.delete_confirm_open = false;
        let Some(instance) = self.instances.current().cloned() else {
            return;
        };
        let name = instance.name.clone();

        if delete_files {
            let dir = sealy_core::instances::directory(&self.paths.game, &name);
            if let Err(err) = std::fs::remove_dir_all(&dir) {
                tracing::warn!(path = %dir.display(), error = %err, "could not remove instance folder");
            }
        }

        let removed = self.remove_instance(&name);
        if delete_files {
            // Nothing is left to restore, so no undo offer.
            self.pending_undo = None;
            self.show_toast(format!("已删除实例 {name} 及其文件"), None);
        } else if let Some(pending) = removed {
            self.pending_undo = Some(pending);
            let lang = self.language();
            self.show_toast(
                format!(
                    "{} {name}",
                    crate::i18n::tr(lang, "已删除实例", "Deleted instance")
                ),
                Some(ToastAction::Undo),
            );
        }
    }

    /// Imports files or folders into one addon folder of the selected instance.
    pub fn import_addons(
        &mut self,
        kind: sealy_core::addons::AddonKind,
        paths: &[std::path::PathBuf],
    ) -> Result<usize, String> {
        let dir = self
            .current_instance_dir()
            .ok_or_else(|| "没有选中的实例".to_owned())?;
        let mut imported = 0_usize;
        let mut last_error = None;
        for path in paths {
            match sealy_core::addons::import(&dir, kind, path) {
                Ok(_) => imported += 1,
                Err(err) => last_error = Some(err.to_string()),
            }
        }
        if imported == 0 {
            return Err(last_error.unwrap_or_else(|| "没有可导入的文件".to_owned()));
        }
        Ok(imported)
    }
    /// Enables or disables one addon.
    pub fn toggle_addon(
        &mut self,
        kind: sealy_core::addons::AddonKind,
        name: &str,
    ) -> Result<bool, String> {
        let dir = self
            .current_instance_dir()
            .ok_or_else(|| "没有选中的实例".to_owned())?;
        sealy_core::addons::toggle(&dir, kind, name).map_err(|err| err.to_string())
    }

    /// Deletes one addon from the instance.
    pub fn remove_addon(
        &mut self,
        kind: sealy_core::addons::AddonKind,
        name: &str,
    ) -> Result<(), String> {
        let dir = self
            .current_instance_dir()
            .ok_or_else(|| "没有选中的实例".to_owned())?;
        sealy_core::addons::remove(&dir, kind, name).map_err(|err| err.to_string())
    }

    /// Downloads, verifies and applies a release update.
    pub(crate) fn download_update(
        &mut self,
        release: &sealy_core::update::ReleaseInfo,
        cx: &mut crate::kit::Context<crate::app::SealyShell>,
    ) {
        if self.update_downloading {
            return;
        }
        let Some(asset) = release.asset.clone() else {
            let lang = self.language();
            self.notify(crate::i18n::tr(
                lang,
                "这个版本没有适合本平台的安装包",
                "No asset for this platform",
            ));
            return;
        };
        self.update_downloading = true;
        self.update_error = None;
        self.update_check = Some(Ok(Some(release.clone())));
        let destination = sealy_core::update::cache_path(&self.paths.data, &asset);
        let done = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        self.update_progress = Some(UpdateProgress {
            done: done.clone(),
            total: asset.size,
        });
        self.update_cancel = Some(cancel.clone());

        let data_dir = self.paths.data.clone();
        let install_dir = self.paths.root.clone();
        let version = release.version.clone();
        let checksums = release.checksums.clone();
        let executable = std::env::current_exe().ok();

        cx.spawn(async move |this, cx| {
            let outcome = cx
                .background_executor()
                .spawn({
                    let asset = asset.clone();
                    let destination = destination.clone();
                    let done = done.clone();
                    let cancel = cancel.clone();
                    let data_dir = data_dir.clone();
                    let version = version.clone();
                    async move {
                        let Some(exe) = executable.as_deref() else {
                            return Err(sealy_core::CoreError::Invalid(
                                "无法定位当前可执行文件".to_owned(),
                            ));
                        };
                        sealy_core::update::install_release(sealy_core::update::UpdateRequest {
                            asset: &asset,
                            destination: &destination,
                            checksums: checksums.as_ref(),
                            downloaded: &done,
                            cancel: &cancel,
                            data_dir: &data_dir,
                            version: &version,
                            install_dir: &install_dir,
                            exe,
                        })
                    }
                })
                .await;

            let succeeded = outcome.is_ok();
            let cancelled = matches!(outcome, Err(sealy_core::CoreError::Cancelled));
            let error = outcome.err().map(|err| err.to_string());
            let _ = this.update(cx, |shell, cx| {
                shell.state.update_downloading = false;
                shell.state.update_progress = None;
                shell.state.update_cancel = None;
                let lang = shell.state.language();
                if succeeded {
                    shell.state.update_error = None;
                    shell.state.notify(crate::i18n::tr(
                        lang,
                        "更新已就绪，正在重启完成安装",
                        "Update ready; restarting to finish",
                    ));
                } else if cancelled {
                    shell
                        .state
                        .notify(crate::i18n::tr(lang, "已取消更新下载", "Update cancelled"));
                } else if let Some(error) = error {
                    shell.state.update_error = Some(error.clone());
                    shell.state.notify(error);
                }
                cx.notify();
            });

            if succeeded {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(1200))
                    .await;
                std::process::exit(0);
            }
        })
        .detach();
    }

    /// Requests cancellation of the running update download.
    pub(crate) fn cancel_update(&mut self) {
        if let Some(cancel) = &self.update_cancel {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Enables or disables the startup update check.
    pub(crate) fn set_auto_check_update(&mut self, enabled: bool) {
        self.settings.update.auto_check = enabled;
        self.persist();
    }
    /// Quietly checks for a release after startup.
    pub(crate) fn spawn_auto_update_check(
        &mut self,
        cx: &mut crate::kit::Context<crate::app::SealyShell>,
    ) {
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { sealy_core::update::check(sealy_core::version()) })
                .await;
            let _ = this.update(cx, |shell, cx| {
                match result {
                    Ok(Some(release)) => {
                        let lang = shell.state.language();
                        let line = format!(
                            "{} v{}",
                            crate::i18n::tr(lang, "发现新版本", "New version available"),
                            release.version
                        );
                        shell.state.notify(line);
                        shell.state.update_check = Some(Ok(Some(release)));
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Analyses the newest crash report and opens the panel.
    pub fn open_crash_report(&mut self) {
        self.crash_view = Some(sealy_core::crash::analyze(&self.paths.game));
        self.crash_open = true;
    }

    /// Re-runs the analysis while the panel stays open.
    pub fn refresh_crash_report(&mut self) {
        self.crash_view = Some(sealy_core::crash::analyze(&self.paths.game));
    }

    /// Closes the crash-analysis panel.
    pub fn close_crash_report(&mut self) {
        self.crash_open = false;
        self.crash_view = None;
    }
    /// Changes the interface scale and persists it.
    pub fn set_ui_scale_pct(&mut self, pct: u16) {
        let clamped = pct.clamp(70, 160);
        self.settings.appearance.ui_scale_pct = clamped;
        crate::kit::set_ui_scale(f32::from(clamped) / 100.0);
        self.persist();
    }
    /// Saves the HTTP proxy address and applies it to new requests.
    pub fn commit_proxy(&mut self, value: String) {
        let trimmed = value.trim().to_owned();
        if !trimmed.is_empty() && sealy_core::net::split_host_port(&trimmed).is_none() {
            let lang = self.language();
            self.notify(crate::i18n::tr(
                lang,
                "代理格式应为 host:port",
                "Use host:port for the proxy",
            ));
            return;
        }
        if let Some(draft) = &mut self.proxy_draft {
            draft.clear();
        }
        self.proxy_draft = None;
        self.settings.network.proxy = trimmed.clone();
        sealy_core::net::set_proxy(Some(trimmed.clone()));
        self.persist();
        let lang = self.language();
        let message = if trimmed.is_empty() {
            crate::i18n::tr(lang, "已关闭代理", "Proxy disabled").to_owned()
        } else {
            format!(
                "{} {trimmed}",
                crate::i18n::tr(lang, "代理已启用", "Proxy enabled")
            )
        };
        self.notify(message);
    }

    /// Switches the game directory and reloads installed versions/Java.
    pub fn commit_game_dir(&mut self, value: String) {
        let trimmed = value.trim().to_owned();
        self.game_dir_draft = None;
        self.settings.game_dir = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.clone())
        };
        let mut paths = self.paths.clone();
        if let Err(err) = sealy_core::apply_runtime_settings(&mut paths, &self.settings) {
            let lang = self.language();
            self.notify(format!(
                "{} {err}",
                crate::i18n::tr(lang, "无法切换目录:", "Cannot switch:")
            ));
            return;
        }
        self.paths = paths;
        self.persist();
        self.installed = sealy_core::version::installed(&self.paths.game);
        self.java = sealy_core::java::detect(&self.paths.game);
        let lang = self.language();
        let line = self.paths.game.display().to_string();
        self.notify(format!(
            "{} {line}",
            crate::i18n::tr(lang, "游戏目录已切换", "Game folder")
        ));
    }
    /// Minecraft version an installed version id resolves to.
    pub fn version_game_version(&self, version_id: &str) -> String {
        let document = self
            .paths
            .game
            .join("versions")
            .join(version_id)
            .join(format!("{version_id}.json"));
        let Ok(text) = std::fs::read_to_string(document) else {
            return version_id.to_owned();
        };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
            return version_id.to_owned();
        };
        json["inheritsFrom"]
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| version_id.to_owned())
    }
    /// Opens the detail page for the selected instance.
    pub fn open_instance_detail(&mut self) {
        if self.instances.current().is_some() {
            self.instance_section = InstanceSection::Overview;
            self.route = Route::Instance;
        }
    }

    /// Returns to the instance list.
    pub fn close_instance_detail(&mut self) {
        self.route = Route::Instances;
    }

    /// Records a row click and reports whether it completes a double click.
    pub fn register_instance_click(&mut self, name: &str) -> bool {
        let now = std::time::Instant::now();
        let double = self.last_instance_click.as_ref().is_some_and(|(last, at)| {
            last == name && now.duration_since(*at) < std::time::Duration::from_millis(450)
        });
        self.last_instance_click = if double {
            None
        } else {
            Some((name.to_owned(), now))
        };
        double
    }

    /// Opens the log viewer for the selected instance.
    pub fn open_logs(&mut self) {
        let Some(instance) = self.instances.current() else {
            return;
        };
        self.logs_title = format!("{} · {}", instance.name, instance.version_id);
        self.logs_path = Some(sealy_core::launch::log_path(
            &self.paths.game,
            &instance.version_id,
        ));
        self.logs_open = true;
        self.refresh_logs();
    }

    /// Re-reads the tail of the log file.
    pub fn refresh_logs(&mut self) {
        let Some(path) = &self.logs_path else {
            return;
        };
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = text
            .lines()
            .rev()
            .take(400)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        lines.reverse();
        self.logs_lines = lines;
    }

    /// Closes the log viewer.
    pub fn close_logs(&mut self) {
        self.logs_open = false;
    }

    /// Reveals the game log directory in the platform file manager.
    pub fn open_log_folder(&mut self) {
        let dir = self.paths.game.join("logs");
        let _ = std::fs::create_dir_all(&dir);
        self.reveal_with_feedback(&dir.clone());
    }

    /// Appends a character to the Java path input.
    pub fn java_path_push_char(&mut self, ch: char) {
        if let Some(draft) = &mut self.java_path_draft
            && draft.chars().count() < 300
        {
            draft.push(ch);
        }
    }

    /// Removes the last character from the Java path input.
    pub fn java_path_pop_char(&mut self) {
        if let Some(draft) = &mut self.java_path_draft {
            draft.pop();
        }
    }

    /// Takes the typed path, closing the input.
    pub fn take_java_path(&mut self) -> Option<std::path::PathBuf> {
        let raw = self.java_path_draft.take()?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(trimmed))
        }
    }

    pub fn authlib_push_char(&mut self, ch: char) {
        if let Some(draft) = &mut self.authlib_draft {
            let field = match draft.field {
                0 => &mut draft.server,
                1 => &mut draft.username,
                _ => &mut draft.password,
            };
            if field.chars().count() < 120 {
                field.push(ch);
            }
        }
    }

    /// Removes the last character of the focused authlib field.
    pub fn authlib_pop_char(&mut self) {
        if let Some(draft) = &mut self.authlib_draft {
            let field = match draft.field {
                0 => &mut draft.server,
                1 => &mut draft.username,
                _ => &mut draft.password,
            };
            field.pop();
        }
    }

    /// Moves focus to the next authlib field.
    pub fn authlib_next_field(&mut self) {
        if let Some(draft) = &mut self.authlib_draft {
            draft.field = (draft.field + 1).min(2);
        }
    }

    /// Whether the password field is focused.
    pub fn authlib_last_field(&self) -> bool {
        self.authlib_draft
            .as_ref()
            .is_some_and(|draft| draft.field == 2)
    }

    /// Discards the rename buffer.
    pub fn cancel_rename(&mut self) {
        self.rename_draft = None;
    }

    /// Duplicates the current instance and copies its game directory.
    pub fn duplicate_instance(&mut self) {
        let Some(current) = self.instances.current().map(|i| i.name.clone()) else {
            return;
        };
        let Some(copy) = self.instances.duplicate(&current) else {
            return;
        };

        let root = self.paths.game.clone();
        let from = sealy_core::instances::directory(&root, &current);
        let to = sealy_core::instances::directory(&root, &copy.name);
        if from.is_dir() && !to.exists() {
            let _ = copy_directory(&from, &to);
        }
        self.save_instances();
    }

    /// Reveals the current instance's game directory in the file manager.
    pub fn open_instance_folder(&mut self) {
        let Some(current) = self.instances.current().map(|i| i.name.clone()) else {
            return;
        };
        let dir = sealy_core::instances::directory(&self.paths.game, &current);
        let _ = std::fs::create_dir_all(&dir);
        self.reveal_with_feedback(&dir.clone());
    }

    /// Adds a message and returns its id.
    pub fn push_message(&mut self, title: String, detail: String, tone: MessageTone) -> u64 {
        let id = self.next_message_id;
        self.next_message_id += 1;
        self.messages.insert(
            0,
            Message {
                id,
                title,
                detail,
                progress: None,
                tone,
                retry: None,
                cancel: None,
            },
        );
        self.messages.truncate(40);
        id
    }

    /// Mutates one message by id.
    pub fn update_message(&mut self, id: u64, update: impl FnOnce(&mut Message)) {
        if let Some(message) = self.messages.iter_mut().find(|message| message.id == id) {
            update(message);
        }
    }

    /// Number of messages still running.
    pub fn running_messages(&self) -> usize {
        self.messages
            .iter()
            .filter(|message| message.tone == MessageTone::Running)
            .count()
    }

    /// Toggles the message panel.
    pub fn toggle_messages(&mut self) {
        self.message_open = !self.message_open;
    }

    /// Shows the wizard from the first step.
    pub fn start_oobe(&mut self) {
        self.oobe_step = Some(0);
    }

    /// Advances one step, finishing the wizard after the last one.
    pub fn oobe_next(&mut self) {
        if self.oobe_step == Some(OOBE_ACCOUNT_STEP) && self.accounts.accounts.is_empty() {
            return;
        }
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
        self.settings.java.memory_mode = MemoryMode::Manual;
        self.settings.java.memory.max_mb = mb.clamp(512, 32768);
        self.persist();
    }

    /// Applies a value dragged on the memory slider; `commit` persists it.
    pub fn apply_slider_memory(&mut self, mb: u32, commit: bool) {
        self.settings.java.memory_mode = MemoryMode::Manual;
        self.settings.java.memory.max_mb = mb.clamp(512, 32768);
        if commit {
            self.persist();
        }
    }

    /// Switches between the manual cap and the launcher-chosen one.
    pub fn set_memory_mode(&mut self, mode: MemoryMode) {
        if self.settings.java.memory_mode == mode {
            return;
        }
        self.settings.java.memory_mode = mode;
        self.persist();
    }

    /// Heap cap actually used on launch.
    pub fn effective_memory_mb(&self) -> u32 {
        match self.settings.java.memory_mode {
            MemoryMode::Auto => sealy_core::system::auto_memory_mb(self.total_ram_mb),
            MemoryMode::Manual => self.settings.java.memory.max_mb.clamp(512, 32768),
        }
    }

    /// Opens the inline numeric memory editor with the current value.
    pub fn begin_memory_draft(&mut self) {
        // Start empty so the first keystroke replaces the current cap.
        self.memory_draft = Some(String::new());
    }

    /// Parses and applies the inline numeric memory editor.
    pub fn commit_memory_draft(&mut self) {
        let Some(text) = self.memory_draft.take() else {
            return;
        };
        let Ok(value) = text.trim().parse::<u32>() else {
            let lang = self.language();
            self.notify(crate::i18n::tr(
                lang,
                "请输入 512-32768 之间的整数（MB）",
                "Enter an integer between 512 and 32768 (MB)",
            ));
            return;
        };
        self.set_max_memory(value);
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
            Ok(()) => {
                self.show_toast(i18n::toast_settings_saved(self.language()).to_owned(), None);
            }
            Err(err) => {
                tracing::warn!(error = %err, "failed to save settings");
                self.show_toast(err.to_string(), None);
            }
        }
    }

    /// Shows a transient message without an action.
    pub fn notify(&mut self, message: impl Into<String>) {
        self.show_toast(message, None);
    }

    /// Shows a toast, replacing any previous one, and returns its id.
    pub fn show_toast(&mut self, message: impl Into<String>, action: Option<ToastAction>) -> u64 {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toast = Some(Toast {
            id,
            message: message.into(),
            action,
        });
        self.toast_deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(5));
        id
    }

    /// Clears the toast if `id` is still the visible one.
    pub fn expire_toast(&mut self, id: u64) {
        if self.toast.as_ref().is_some_and(|toast| toast.id == id) {
            self.toast = None;
            self.pending_undo = None;
            self.toast_deadline = None;
        }
    }

    /// Clears the toast once its deadline passed; returns whether it did.
    pub fn expire_due_toast(&mut self) -> bool {
        let Some(deadline) = self.toast_deadline else {
            return false;
        };
        if std::time::Instant::now() < deadline {
            return false;
        }
        self.toast = None;
        self.pending_undo = None;
        self.toast_deadline = None;
        true
    }

    /// Turns verbose file logging on or off and persists the choice.
    pub fn set_verbose_logging(&mut self, on: bool) {
        self.settings.debug.verbose_logging = on;
        sealy_core::logging::set_verbose(on);
        self.persist();
    }

    /// Reveals the data directory in the platform file manager.
    /// Reveals the shared `.minecraft` folder.
    pub fn open_game_folder(&mut self) {
        if let Err(err) = sealy_core::shell::spawn_open_helper(&self.paths.game) {
            self.notify(err.to_string());
        }
    }

    /// Starts the self-update query and stores its result for the settings page.
    pub(crate) fn start_update_check(
        &mut self,
        cx: &mut crate::kit::Context<crate::app::SealyShell>,
    ) {
        self.update_check = None;
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { sealy_core::update::check(sealy_core::version()) })
                .await
                .map_err(|err| err.to_string());
            let _ = this.update(cx, |shell, cx| {
                shell.state.update_check = Some(result);
                cx.notify();
            });
        })
        .detach();
    }
    pub fn open_data_folder(&mut self) {
        let _ = std::fs::create_dir_all(&self.paths.data);
        let dir = self.paths.data.clone();
        self.reveal_with_feedback(&dir);
    }

    /// Reveals SealyData/logs in the platform file manager.
    pub fn open_logs_folder(&mut self) {
        let _ = std::fs::create_dir_all(&self.paths.logs);
        let dir = self.paths.logs.clone();
        self.reveal_with_feedback(&dir);
    }

    /// Reveals a folder and reports the outcome in a toast.
    pub fn reveal_with_feedback(&mut self, path: &std::path::Path) {
        match reveal_inner(path) {
            Ok(()) => self.notify(format!("已打开 {}", path.display())),
            Err(err) => {
                tracing::warn!(path = %path.display(), error = %err, "could not open folder");
                self.notify(format!("无法打开目录：{err}"));
            }
        }
    }

    /// Dismisses the toast immediately.
    pub fn dismiss_toast(&mut self) {
        self.toast = None;
        self.pending_undo = None;
        self.toast_deadline = None;
    }

    /// Restores the last deleted item.
    pub fn undo(&mut self) {
        let Some(pending) = self.pending_undo.take() else {
            return;
        };

        match pending {
            PendingUndo::Instance {
                instance,
                index,
                selected,
            } => {
                let name = instance.name.clone();
                self.instances.restore(instance, index, selected);
                self.save_instances();
                self.instances.select(&name);
                self.show_toast(
                    crate::i18n::tr(self.language(), "已恢复实例", "Instance restored"),
                    None,
                );
            }
            PendingUndo::Account {
                account,
                index,
                selected,
            } => {
                self.accounts.restore(account, index, selected);
                self.save_accounts();
                self.show_toast(
                    crate::i18n::tr(self.language(), "已恢复账号", "Account restored"),
                    None,
                );
            }
        }
    }
}

/// Copies a directory tree, ignoring failures on individual entries.
fn copy_directory(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            let _ = copy_directory(&entry.path(), &target);
        } else {
            let _ = std::fs::copy(entry.path(), &target);
        }
    }
    Ok(())
}
/// Hands the folder to the platform file manager through a helper process.
///
/// The shell call pumps a message loop and would re-enter GPUI's borrowed
/// application state if it ran on the UI thread, so it runs in a separate
/// short lived launcher process instead.
fn reveal_inner(path: &std::path::Path) -> std::io::Result<()> {
    sealy_core::shell::spawn_open_helper(path)
}
