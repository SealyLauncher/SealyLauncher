//! Application menu bar (macOS), dock menu and the actions they dispatch.
//!
//! Each menu item dispatches a typed GPUI action. The shell root element
//! registers one listener per action and maps it onto [`MenuAction`], so the
//! same entry points are reachable from the menu bar, the dock and keybindings.

use crate::app::SealyShell;
use crate::kit::{App, Context, KeyBinding, Menu, MenuItem, NoAction, OsAction, SystemMenuType};
use crate::state::{Route, SettingsSection};

gpui_kit::actions!(
    sealy,
    [
        OpenHome,
        OpenInstances,
        OpenDownloads,
        OpenSettings,
        NewInstance,
        ImportModpack,
        OpenGameFolder,
        OpenDataFolder,
        OpenLogFolder,
        CheckUpdate,
        AboutLauncher,
        OpenRepository,
        QuitLauncher,
    ]
);

/// What a menu item asks the shell to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    /// Go to the home page.
    Home,
    /// Go to the instance list.
    Instances,
    /// Go to the download page.
    Downloads,
    /// Open the settings page.
    Settings,
    /// Open the settings page on its launcher section.
    About,
    /// Start the new-instance panel.
    NewInstance,
    /// Pick and import a modpack archive.
    ImportModpack,
    /// Reveal the `.minecraft` folder.
    GameFolder,
    /// Reveal the launcher data folder.
    DataFolder,
    /// Reveal the log folder.
    LogFolder,
    /// Query GitHub for a newer release.
    CheckUpdate,
    /// Open the project page in the browser.
    Repository,
    /// Quit the application.
    Quit,
}

/// Applies one menu action and repaints.
pub(crate) fn apply(shell: &mut SealyShell, action: MenuAction, cx: &mut Context<SealyShell>) {
    match action {
        MenuAction::Home => shell.state.route = Route::Home,
        MenuAction::Instances => shell.state.route = Route::Instances,
        MenuAction::Downloads => shell.state.route = Route::Downloads,
        MenuAction::Settings => shell.state.route = Route::Settings,
        MenuAction::About => {
            shell.state.route = Route::Settings;
            shell.state.settings_section = SettingsSection::Launcher;
            let version = sealy_core::version();
            let lang = shell.state.language();
            let line = format!(
                "SealyLauncher v{version} · {}",
                crate::i18n::tr(lang, "便携启动器", "portable launcher")
            );
            shell.state.notify(line);
        }
        MenuAction::NewInstance => {
            shell.state.route = Route::Instances;
            shell.state.begin_new_instance();
        }
        MenuAction::ImportModpack => {
            shell.state.route = Route::Instances;
            crate::install_flow::prompt_modpack(cx);
        }
        MenuAction::GameFolder => shell.state.open_game_folder(),
        MenuAction::DataFolder => shell.state.open_data_folder(),
        MenuAction::LogFolder => shell.state.open_logs_folder(),
        MenuAction::CheckUpdate => {
            shell.state.route = Route::Settings;
            shell.state.settings_section = SettingsSection::Launcher;
            shell.state.start_update_check(cx);
        }
        MenuAction::Repository => {
            let _ = sealy_core::shell::spawn_open_url_helper(
                "https://github.com/SealyLauncher/SealyLauncher",
            );
        }
        MenuAction::Quit => cx.quit(),
    }
    cx.notify();
}

/// Builds the macOS menu bar for one language.
pub fn menus(lang: sealy_core::config::Language) -> Vec<Menu> {
    use crate::i18n::tr;

    let app = Menu::new("SealyLauncher").items([
        MenuItem::action(
            tr(lang, "关于 SealyLauncher", "About SealyLauncher"),
            AboutLauncher,
        ),
        MenuItem::action(tr(lang, "检查更新…", "Check for updates…"), CheckUpdate),
        MenuItem::separator(),
        MenuItem::action(tr(lang, "设置…", "Settings…"), OpenSettings),
        MenuItem::separator(),
        MenuItem::os_submenu(tr(lang, "服务", "Services"), SystemMenuType::Services),
        MenuItem::separator(),
        MenuItem::action(
            tr(lang, "退出 SealyLauncher", "Quit SealyLauncher"),
            QuitLauncher,
        ),
    ]);

    let edit = Menu::new(tr(lang, "编辑", "Edit")).items([
        MenuItem::os_action(tr(lang, "撤销", "Undo"), NoAction, OsAction::Undo),
        MenuItem::os_action(tr(lang, "重做", "Redo"), NoAction, OsAction::Redo),
        MenuItem::separator(),
        MenuItem::os_action(tr(lang, "剪切", "Cut"), NoAction, OsAction::Cut),
        MenuItem::os_action(tr(lang, "复制", "Copy"), NoAction, OsAction::Copy),
        MenuItem::os_action(tr(lang, "粘贴", "Paste"), NoAction, OsAction::Paste),
        MenuItem::os_action(
            tr(lang, "全选", "Select All"),
            NoAction,
            OsAction::SelectAll,
        ),
    ]);

    let file = Menu::new(tr(lang, "文件", "File")).items([
        MenuItem::action(tr(lang, "新建实例", "New instance"), NewInstance),
        MenuItem::action(tr(lang, "导入整合包…", "Import modpack…"), ImportModpack),
        MenuItem::separator(),
        MenuItem::action(tr(lang, "打开游戏目录", "Open game folder"), OpenGameFolder),
        MenuItem::action(tr(lang, "打开数据目录", "Open data folder"), OpenDataFolder),
    ]);

    let view = Menu::new(tr(lang, "视图", "View")).items([
        MenuItem::action(tr(lang, "主页", "Home"), OpenHome),
        MenuItem::action(tr(lang, "实例", "Instances"), OpenInstances),
        MenuItem::action(tr(lang, "下载", "Downloads"), OpenDownloads),
        MenuItem::separator(),
        MenuItem::action(tr(lang, "游戏日志", "Game logs"), OpenLogFolder),
    ]);

    let help = Menu::new(tr(lang, "帮助", "Help")).items([
        MenuItem::action(tr(lang, "GitHub 仓库", "GitHub repository"), OpenRepository),
        MenuItem::action(tr(lang, "打开日志目录", "Open log folder"), OpenLogFolder),
    ]);

    vec![app, file, edit, view, help]
}

/// Installs the menu bar and keybindings.
///
/// The menu bar itself only exists on macOS; the bindings stay harmless
/// elsewhere and keep the same entry points available.
pub fn install(cx: &mut App, lang: sealy_core::config::Language) {
    #[cfg(target_os = "macos")]
    {
        cx.set_menus(menus(lang));
        cx.set_dock_menu(vec![
            MenuItem::action("新建实例", NewInstance),
            MenuItem::action("导入整合包…", ImportModpack),
            MenuItem::separator(),
            MenuItem::action("设置…", OpenSettings),
        ]);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = lang;
    }

    cx.bind_keys([
        KeyBinding::new("cmd-,", OpenSettings, None),
        KeyBinding::new("cmd-n", NewInstance, None),
        KeyBinding::new("cmd-o", ImportModpack, None),
        KeyBinding::new("cmd-1", OpenHome, None),
        KeyBinding::new("cmd-2", OpenInstances, None),
        KeyBinding::new("cmd-3", OpenDownloads, None),
        KeyBinding::new("cmd-q", QuitLauncher, None),
    ]);
}
