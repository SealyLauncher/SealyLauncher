//! Mods, resource packs and shader packs stored inside one instance.
//!
//! Files are enabled by their plain name and disabled by appending
//! `.disabled`, which is what Minecraft itself expects (it only scans the
//! original extension).

use std::path::{Path, PathBuf};

use crate::error::CoreError;

/// Suffix used to mark an addon as disabled.
const DISABLED_SUFFIX: &str = ".disabled";

/// Kinds of addon the launcher can manage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddonKind {
    /// `.jar` files in `mods/`.
    Mod,
    /// `.zip` files or folders in `resourcepacks/`.
    ResourcePack,
    /// `.zip` files or folders in `shaderpacks/`.
    Shader,
}

impl AddonKind {
    /// Directory name inside the instance.
    pub fn folder(self) -> &'static str {
        match self {
            Self::Mod => "mods",
            Self::ResourcePack => "resourcepacks",
            Self::Shader => "shaderpacks",
        }
    }

    /// All kinds, in display order.
    pub const ALL: [Self; 3] = [Self::Mod, Self::ResourcePack, Self::Shader];

    /// Whether a file or directory name belongs to this kind.
    pub fn accepts(self, name: &str) -> bool {
        let lowered = name.to_ascii_lowercase();
        match self {
            Self::Mod => lowered.ends_with(".jar"),
            Self::ResourcePack | Self::Shader => {
                lowered.ends_with(".zip") || Path::new(name).extension().is_none()
            }
        }
    }
}

/// One addon found in an instance folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addon {
    /// Name shown to the user, without the `.disabled` suffix.
    pub name: String,
    /// Whether the file is currently active.
    pub enabled: bool,
    /// Size in bytes (`0` for directories).
    pub bytes: u64,
    /// Whether the entry is a directory (common for packs and shaders).
    pub is_dir: bool,
}

impl Addon {
    /// File system path of this entry.
    pub fn path(&self, instance_dir: &Path, kind: AddonKind) -> PathBuf {
        let folder = instance_dir.join(kind.folder());
        if self.enabled {
            folder.join(&self.name)
        } else {
            folder.join(format!("{}{DISABLED_SUFFIX}", self.name))
        }
    }
}

/// Lists addons of one kind, sorted by name.
pub fn list(instance_dir: &Path, kind: AddonKind) -> Vec<Addon> {
    let folder = instance_dir.join(kind.folder());
    let Ok(entries) = std::fs::read_dir(&folder) else {
        return Vec::new();
    };

    let mut found = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(raw) = file_name.to_str() else {
            continue;
        };
        if raw.starts_with('.') {
            continue;
        }

        let (name, enabled) = match raw.strip_suffix(DISABLED_SUFFIX) {
            Some(stripped) => (stripped.to_owned(), false),
            None => (raw.to_owned(), true),
        };
        if !kind.accepts(&name) {
            continue;
        }

        let metadata = entry.metadata().ok();
        found.push(Addon {
            name,
            enabled,
            bytes: metadata.as_ref().map(|meta| meta.len()).unwrap_or(0),
            is_dir: metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false),
        });
    }

    found.sort_by_key(|addon| addon.name.to_lowercase());
    found
}

/// Enables or disables an addon; returns the new enabled state.
pub fn toggle(instance_dir: &Path, kind: AddonKind, name: &str) -> Result<bool, CoreError> {
    let folder = instance_dir.join(kind.folder());
    let enabled_path = folder.join(name);
    let disabled_path = folder.join(format!("{name}{DISABLED_SUFFIX}"));

    if enabled_path.exists() {
        std::fs::rename(&enabled_path, &disabled_path)?;
        return Ok(false);
    }
    if disabled_path.exists() {
        if enabled_path.exists() {
            return Err(CoreError::Invalid(format!(
                "无法启用 {name}：同名文件已存在"
            )));
        }
        std::fs::rename(&disabled_path, &enabled_path)?;
        return Ok(true);
    }

    Err(CoreError::Invalid(format!("找不到附加内容 {name}")))
}

/// Deletes an addon (file or directory).
pub fn remove(instance_dir: &Path, kind: AddonKind, name: &str) -> Result<(), CoreError> {
    let folder = instance_dir.join(kind.folder());
    let candidates = [
        folder.join(name),
        folder.join(format!("{name}{DISABLED_SUFFIX}")),
    ];

    for path in candidates {
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
            return Ok(());
        }
        if path.is_file() {
            std::fs::remove_file(&path)?;
            return Ok(());
        }
    }

    Err(CoreError::Invalid(format!("找不到附加内容 {name}")))
}

/// Copies a file or directory into the instance and returns its display name.
pub fn import(instance_dir: &Path, kind: AddonKind, source: &Path) -> Result<String, CoreError> {
    let Some(file_name) = source.file_name() else {
        return Err(CoreError::Invalid("导入的路径没有文件名".to_owned()));
    };
    let Some(name) = file_name.to_str() else {
        return Err(CoreError::Invalid("导入的文件名不是 UTF-8".to_owned()));
    };
    if !kind.accepts(name) {
        return Err(CoreError::Invalid(format!(
            "{name} 不是 {} 支持的格式",
            kind.folder()
        )));
    }

    let folder = instance_dir.join(kind.folder());
    std::fs::create_dir_all(&folder)?;
    let destination = folder.join(name);
    if destination.exists() || folder.join(format!("{name}{DISABLED_SUFFIX}")).exists() {
        return Err(CoreError::Invalid(format!("{name} 已存在")));
    }

    if source.is_dir() {
        copy_directory(source, &destination)?;
    } else {
        std::fs::copy(source, &destination)?;
    }

    Ok(name.to_owned())
}

/// Recursively copies a directory.
fn copy_directory(from: &Path, to: &Path) -> Result<(), CoreError> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_directory(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join(format!("addons-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn lists_and_toggles_mods() {
        let instance = scratch("mods");
        let mods = instance.join("mods");
        std::fs::create_dir_all(&mods).unwrap();
        std::fs::write(mods.join("sodium.jar"), b"jar").unwrap();
        std::fs::write(mods.join("iris.jar.disabled"), b"jar").unwrap();
        std::fs::write(mods.join("readme.txt"), b"x").unwrap();

        let listed = list(&instance, AddonKind::Mod);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "iris.jar");
        assert!(!listed[0].enabled);
        assert_eq!(listed[1].name, "sodium.jar");
        assert!(listed[1].enabled);

        assert!(!toggle(&instance, AddonKind::Mod, "sodium.jar").unwrap());
        assert!(mods.join("sodium.jar.disabled").is_file());
        assert!(list(&instance, AddonKind::Mod)[1].name == "sodium.jar");
        assert!(!list(&instance, AddonKind::Mod)[1].enabled);

        assert!(toggle(&instance, AddonKind::Mod, "iris.jar").unwrap());
        assert!(mods.join("iris.jar").is_file());

        let _ = std::fs::remove_dir_all(&instance);
    }

    #[test]
    fn imports_and_removes_files_and_folders() {
        let instance = scratch("import");
        let source = scratch("source");
        std::fs::write(source.join("pack.zip"), b"zip").unwrap();
        let folder = source.join("ShaderPack");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("shaders.txt"), b"x").unwrap();

        let imported =
            import(&instance, AddonKind::ResourcePack, &source.join("pack.zip")).unwrap();
        assert_eq!(imported, "pack.zip");
        assert!(instance.join("resourcepacks/pack.zip").is_file());

        let imported = import(&instance, AddonKind::Shader, &folder).unwrap();
        assert_eq!(imported, "ShaderPack");
        assert!(
            instance
                .join("shaderpacks/ShaderPack/shaders.txt")
                .is_file()
        );

        // Duplicates are rejected.
        assert!(import(&instance, AddonKind::ResourcePack, &source.join("pack.zip")).is_err());
        // Unsupported formats are rejected.
        let bogus = source.join("notes.txt");
        std::fs::write(&bogus, b"x").unwrap();
        assert!(import(&instance, AddonKind::Mod, &bogus).is_err());

        remove(&instance, AddonKind::Shader, "ShaderPack").unwrap();
        assert!(!instance.join("shaderpacks/ShaderPack").exists());

        let _ = std::fs::remove_dir_all(&instance);
        let _ = std::fs::remove_dir_all(&source);
    }

    #[test]
    fn accepts_expected_extensions() {
        assert!(AddonKind::Mod.accepts("a.jar"));
        assert!(!AddonKind::Mod.accepts("a.zip"));
        assert!(AddonKind::ResourcePack.accepts("a.zip"));
        assert!(AddonKind::ResourcePack.accepts("Pack"));
        assert!(!AddonKind::ResourcePack.accepts("a.jar"));
    }
}
