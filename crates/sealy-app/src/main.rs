// Release builds are windowed apps: no console window behind the launcher.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod window_icon;

fn main() {
    if let Some(url) = open_url_argument() {
        match sealy_core::shell::open_url(&url) {
            Ok(()) => std::process::exit(0),
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }

    if let Some(path) = open_folder_argument() {
        match sealy_core::shell::open_folder(std::path::Path::new(&path)) {
            Ok(()) => std::process::exit(0),
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
    }

    if std::env::args().any(|arg| arg == "--check-update") {
        match sealy_core::update::check(sealy_core::version()) {
            Ok(Some(release)) => println!(
                "update {}{}",
                release.version,
                release
                    .asset
                    .map(|asset| format!(" · {}", asset.name))
                    .unwrap_or_default()
            ),
            Ok(None) => println!("up-to-date {}", sealy_core::version()),
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        }
        return;
    }

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

    if let Some(path) = modpack_argument() {
        let code = import_modpack_headless(&bootstrap, std::path::Path::new(&path));
        std::process::exit(code);
    }
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

/// Reads the `--import-modpack <path>` headless import invocation.
fn modpack_argument() -> Option<String> {
    argument_after("--import-modpack")
}

/// Imports a modpack without opening the window; used by automation and CI.
fn import_modpack_headless(bootstrap: &sealy_core::Bootstrap, archive: &std::path::Path) -> i32 {
    let root = bootstrap.paths.game.clone();
    let plan = match sealy_core::modpack::inspect(archive) {
        Ok(plan) => plan,
        Err(err) => {
            eprintln!("{err}");
            return 1;
        }
    };
    println!(
        "pack: {} {} · {} · {:?}",
        plan.name, plan.version, plan.game_version, plan.format
    );

    let policy = bootstrap.settings.download.metadata_source;
    let concurrency = usize::from(bootstrap.settings.download.concurrency).max(1);
    let progress = std::sync::Arc::new(std::sync::Mutex::new(
        sealy_core::install::Progress::default(),
    ));
    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let version_id = match &plan.loader {
        Some(spec) => {
            let java = sealy_core::java::runtime_java_candidates(&root)
                .into_iter()
                .next();
            let loader = match sealy_core::loader::install(
                &root,
                spec.kind,
                &plan.game_version,
                spec.version.as_deref(),
                java.as_deref(),
            ) {
                Ok(loader) => loader,
                Err(err) => {
                    eprintln!("{err}");
                    return 1;
                }
            };
            println!("loader: {} {}", spec.kind.label(), loader.id);
            if let Err(err) = sealy_core::install::install(
                &root,
                &loader.id,
                policy,
                concurrency,
                progress.clone(),
                cancel.clone(),
            ) {
                eprintln!("{err}");
                return 1;
            }
            loader.id
        }
        None => {
            if let Err(err) = sealy_core::install::install(
                &root,
                &plan.game_version,
                policy,
                concurrency,
                progress.clone(),
                cancel.clone(),
            ) {
                eprintln!("{err}");
                return 1;
            }
            plan.game_version.clone()
        }
    };

    let mut book = sealy_core::instances::InstanceBook::load(&bootstrap.paths.instances_file());
    let instance = book.create_named(&plan.name, &version_id);
    if let Err(err) = book.save(&bootstrap.paths.instances_file()) {
        eprintln!("{err}");
        return 1;
    }
    let instance_dir = sealy_core::instances::directory(&root, &instance.name);
    let summary = match sealy_core::modpack::import_files(
        &instance_dir,
        archive,
        &plan,
        &progress,
        &cancel,
    ) {
        Ok(summary) => summary,
        Err(err) => {
            // A failed download must not leave an empty instance in the list.
            sealy_core::instances::discard(
                &root,
                &mut book,
                &bootstrap.paths.instances_file(),
                &instance.name,
            );
            eprintln!("{err}");
            return 1;
        }
    };
    println!(
        "instance: {} · {} files · {} overrides · {} skipped",
        instance.name, summary.files, summary.overrides, summary.skipped
    );
    0
}
/// Reads the --open-folder <path> helper invocation, if any.
fn open_folder_argument() -> Option<String> {
    argument_after(sealy_core::shell::OPEN_FOLDER_FLAG)
}

/// Reads the --open-url <url> helper invocation, if any.
fn open_url_argument() -> Option<String> {
    argument_after(sealy_core::shell::OPEN_URL_FLAG)
}

/// Returns the argument following lag.
fn argument_after(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }
    None
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
