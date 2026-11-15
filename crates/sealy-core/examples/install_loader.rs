//! Manual integration check for the Forge/NeoForge installer path.
//!
//! Usage: cargo run -p sealy-core --example install_loader -- <root> <java> <kind> <game> [loader-version]

fn main() {
    let mut args = std::env::args().skip(1);
    let root = std::path::PathBuf::from(args.next().expect("root"));
    let java = std::path::PathBuf::from(args.next().expect("java"));
    let kind = match args.next().unwrap_or_else(|| "forge".to_owned()).as_str() {
        "neoforge" => sealy_core::loader::LoaderKind::NeoForge,
        "fabric" => sealy_core::loader::LoaderKind::Fabric,
        "quilt" => sealy_core::loader::LoaderKind::Quilt,
        _ => sealy_core::loader::LoaderKind::Forge,
    };
    let game = args.next().unwrap_or_else(|| "1.20.1".to_owned());
    let version = args.next();
    let jar = args.next().map(std::path::PathBuf::from);
    let outcome = match jar {
        Some(jar) => sealy_core::loader::install_from_jar(&root, &game, kind, &jar, &java),
        None => sealy_core::loader::install(&root, kind, &game, version.as_deref(), Some(&java)),
    };
    match outcome {
        Ok(installed) => println!(
            "ok id={} game={} loader={}",
            installed.id, installed.game_version, installed.loader_version
        ),
        Err(err) => println!("err {err}"),
    }
}
