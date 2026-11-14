// Release builds are windowed apps: no console window behind the launcher.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod window_icon;

fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("SealyLauncher {}", sealy_core::version());
        return;
    }

    let bootstrap = match sealy_core::bootstrap() {
        Ok(bootstrap) => bootstrap,
        Err(err) => {
            report_fatal(&format!("SealyLauncher 启动失败: {err}"));
            std::process::exit(1);
        }
    };

    let _log_guard =
        match sealy_core::logging::init(&bootstrap.paths, bootstrap.settings.debug.verbose_logging)
        {
            Ok(guard) => Some(guard),
            Err(err) => {
                eprintln!("日志初始化失败，将继续启动: {err}");
                None
            }
        };

    let _lock = match sealy_core::single_instance::acquire(&bootstrap.paths.lock_file()) {
        Ok(lock) => lock,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    #[cfg(windows)]
    window_icon::apply();

    sealy_ui::app::run(bootstrap);
}

/// Reports a fatal startup error where the user can actually see it.
///
/// A windowed macOS `.app` has no terminal, so a silent `eprintln!` looks like
/// "the app does nothing"; show a native alert and leave a file behind.
fn report_fatal(message: &str) {
    eprintln!("{message}");

    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(parent) = exe.parent()
    {
        roots.push(parent.to_path_buf());
    }
    roots.push(std::env::temp_dir());

    for root in roots {
        let path = root.join("SealyLauncher-error.log");
        if std::fs::write(&path, format!("{message}\n")).is_ok() {
            eprintln!("详细信息已写入 {}", path.display());
            break;
        }
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "display dialog \"{escaped}\" buttons {{\"OK\"}} default button \"OK\" \
             with title \"SealyLauncher\" with icon caution"
        );
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status();
    }
}
