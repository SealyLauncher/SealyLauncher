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
            eprintln!("SealyLauncher 启动失败: {err}");
            std::process::exit(1);
        }
    };

    let _log_guard = match sealy_core::logging::init(&bootstrap.paths) {
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
