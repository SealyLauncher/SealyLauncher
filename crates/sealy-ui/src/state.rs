//! UI shell state and navigation routes.

use sealy_core::Bootstrap;
use sealy_core::config::{Language, Settings, ThemeName, save_settings};
use sealy_core::paths::LauncherPaths;

use crate::i18n;
use crate::kit::IconName;
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
            downloads: DownloadsState::default(),
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
        }
    }

    /// Whether the first-run wizard is showing.
    pub fn in_oobe(&self) -> bool {
        self.oobe_step.is_some()
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

    /// Opens the detail page for the selected instance.
    pub fn open_instance_detail(&mut self) {
        if self.instances.current().is_some() {
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

/// Per-platform launcher for the file manager.
fn reveal_inner(path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // `explorer.exe <path>` and PowerShell-based helpers silently fail for
        // some users; the shell "open" verb is the reliable route.
        return windows_shell::open_folder(path).map_err(std::io::Error::other);
    }

    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Minimal `ShellExecuteW` binding used to reveal folders on Windows.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
mod windows_shell {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    /// `ShellExecuteW` returns an `HINSTANCE`; values above 32 mean success.
    const SUCCESS_THRESHOLD: isize = 32;
    /// `SW_SHOWNORMAL`.
    const SW_SHOWNORMAL: i32 = 1;

    #[link(name = "shell32")]
    unsafe extern "system" {
        fn ShellExecuteW(
            hwnd: *mut core::ffi::c_void,
            operation: *const u16,
            file: *const u16,
            parameters: *const u16,
            directory: *const u16,
            show_cmd: i32,
        ) -> *mut core::ffi::c_void;
    }

    fn wide(value: &OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }

    /// Opens a folder with the shell's default verb.
    pub(super) fn open_folder(path: &std::path::Path) -> Result<(), String> {
        let operation = wide(OsStr::new("open"));
        let file = wide(path.as_os_str());
        // SAFETY: every pointer refers to a null-terminated UTF-16 buffer that
        // outlives the call; the remaining arguments are plain OS API values.
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                operation.as_ptr(),
                file.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            )
        };

        if result as isize > SUCCESS_THRESHOLD {
            return Ok(());
        }

        // Fall back to spawning Explorer directly: some shells refuse the
        // `open` verb for folders inside a job object.
        if std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        Err(format!(
            "ShellExecuteW failed for {} (code {})",
            path.display(),
            result as isize
        ))
    }
}
