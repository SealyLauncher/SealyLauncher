//! Minimal bilingual copy for the Phase 1 shell.
//!
//! Strings stay centralized so Phase 2 can replace this module with a full
//! i18n backend without touching page layout code.

use sealy_core::config::Language;

/// Selects between a Chinese and an English string.
pub fn tr(lang: Language, zh: &'static str, en: &'static str) -> &'static str {
    match lang {
        Language::ZhCn => zh,
        Language::EnUs => en,
    }
}

pub fn home_title(lang: Language) -> &'static str {
    tr(lang, "开始游戏", "Jump back in")
}

pub fn home_status(lang: Language) -> &'static str {
    tr(lang, "未安装任何版本", "No versions installed")
}

pub fn install_version(lang: Language) -> &'static str {
    tr(lang, "安装游戏", "Install a version")
}

pub fn signed_out(lang: Language) -> &'static str {
    tr(lang, "未登录", "Signed out")
}

pub fn sign_in(lang: Language) -> &'static str {
    tr(lang, "登录", "Sign in")
}

pub fn no_instances(lang: Language) -> &'static str {
    tr(lang, "还没有实例", "No instances yet")
}

pub fn search_versions(lang: Language) -> &'static str {
    tr(lang, "搜索版本", "Search versions")
}

pub fn install(lang: Language) -> &'static str {
    tr(lang, "安装", "Install")
}

pub fn release_label(lang: Language) -> &'static str {
    tr(lang, "正式版", "Release")
}

pub fn snapshot_label(lang: Language) -> &'static str {
    tr(lang, "快照", "Snapshot")
}

pub fn add_account(lang: Language) -> &'static str {
    tr(lang, "添加账号", "Add account")
}

pub fn microsoft(lang: Language) -> &'static str {
    tr(lang, "Microsoft 账号", "Microsoft account")
}

pub fn authlib(lang: Language) -> &'static str {
    tr(lang, "authlib-injector", "authlib-injector")
}

pub fn offline(lang: Language) -> &'static str {
    tr(lang, "离线登录", "Offline login")
}

pub fn settings_language(lang: Language) -> &'static str {
    tr(lang, "语言", "Language")
}

pub fn settings_theme(lang: Language) -> &'static str {
    tr(lang, "主题", "Theme")
}

pub fn settings_dark(lang: Language) -> &'static str {
    tr(lang, "深海", "Deep sea")
}

pub fn settings_motion(lang: Language) -> &'static str {
    tr(lang, "动态效果", "Motion")
}

pub fn settings_system(lang: Language) -> &'static str {
    tr(lang, "跟随系统", "System")
}

pub fn settings_source(lang: Language) -> &'static str {
    tr(lang, "下载源", "Download source")
}

pub fn settings_auto(lang: Language) -> &'static str {
    tr(lang, "自动", "Auto")
}

pub fn settings_data_dir(lang: Language) -> &'static str {
    tr(lang, "数据目录", "Data directory")
}

pub fn settings_version(lang: Language) -> &'static str {
    tr(lang, "版本", "Version")
}

pub fn settings_oem(lang: Language) -> &'static str {
    tr(lang, "OEM", "OEM")
}

pub fn settings_oem_off(lang: Language) -> &'static str {
    tr(lang, "未启用", "Not enabled")
}

pub fn toast_install_phase2(lang: Language) -> &'static str {
    tr(
        lang,
        "下载功能将在 Phase 2 接入",
        "Downloads arrive in Phase 2",
    )
}

pub fn toast_login_phase3(lang: Language) -> &'static str {
    tr(
        lang,
        "登录功能将在 Phase 3 接入",
        "Sign-in arrives in Phase 3",
    )
}

pub fn toast_settings_saved(lang: Language) -> &'static str {
    tr(lang, "设置已保存", "Settings saved")
}
