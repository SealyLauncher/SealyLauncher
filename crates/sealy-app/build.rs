//! Embeds the launcher icon into the Windows executable.
//!
//! `rc.exe` ships with the Windows SDK and is present on CI images; on machines
//! without it the build still succeeds, just without an explorer icon.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../packaging/windows/sealy.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest.join("../../packaging/windows/sealy.ico");
    if !icon.is_file() {
        println!("cargo:warning=icon file missing, skipping executable icon");
        return;
    }

    let Some(rc) = find_rc() else {
        if std::env::var("SEALY_REQUIRE_ICON").is_ok() {
            panic!("rc.exe not found but SEALY_REQUIRE_ICON is set");
        }
        println!("cargo:warning=rc.exe not found, skipping executable icon");
        return;
    };

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let script = out.join("sealy.rc");
    let escaped = icon.display().to_string().replace('\\', "\\\\");
    let contents = format!("IDI_ICON1 ICON \"{escaped}\"\n");
    if std::fs::write(&script, contents).is_err() {
        return;
    }

    let resource = out.join("sealy.res");
    let status = Command::new(rc)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&script)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("cargo:rustc-link-arg={}", resource.display());
        }
        _ => println!("cargo:warning=rc.exe failed, skipping executable icon"),
    }
}

/// Looks for `rc.exe` in the environment, PATH and the Windows SDK.
fn find_rc() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("RC")
        && std::path::Path::new(&path).is_file()
    {
        return Some(PathBuf::from(path));
    }

    for dir in std::env::split_paths(&std::env::var_os("PATH")?) {
        let candidate = dir.join("rc.exe");
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let kits = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin");
    let mut versions: Vec<PathBuf> = std::fs::read_dir(&kits)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    versions.sort();
    versions.reverse();

    for version in versions {
        for arch in ["x64", "x86", "arm64"] {
            let candidate = version.join(arch).join("rc.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}
