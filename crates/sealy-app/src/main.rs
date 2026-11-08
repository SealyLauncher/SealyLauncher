fn main() {
    if std::env::args().any(|arg| arg == "--version") {
        println!("SealyLauncher {}", sealy_core::version());
        return;
    }

    sealy_ui::app::run();
}
