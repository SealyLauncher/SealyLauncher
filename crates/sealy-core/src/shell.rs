//! Opening folders with the platform file manager.
//!
//! The shell call must never run on the UI thread: `ShellExecuteW` pumps the
//! window message loop, which re-enters GPUI's borrowed application state and
//! makes the launcher stop drawing. Each request therefore runs in a short
//! lived helper process (the launcher binary with `--open-folder`).

use std::path::Path;

/// Argument that turns the launcher binary into the folder-opening helper.
pub const OPEN_FOLDER_FLAG: &str = "--open-folder";

/// Reveals a folder with the platform file manager.
pub fn open_folder(path: &Path) -> Result<(), String> {
    let result = open_folder_impl(path);
    trace(path, &result);
    result
}

/// Appends one diagnostic line so field reports can show what the shell did.
fn trace(path: &Path, result: &Result<(), String>) {
    let line = match result {
        Ok(()) => format!("ok {}", path.display()),
        Err(err) => format!("error {}: {err}", path.display()),
    };
    let file = std::env::temp_dir().join("sealy-open-folder.log");
    if let Ok(mut handle) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
    {
        use std::io::Write as _;
        let _ = writeln!(handle, "{line}");
    }
}

fn open_folder_impl(path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return windows_shell::open_folder(path);
    }

    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|err| err.to_string());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Starts the helper process that reveals `path` instead of blocking the UI.
pub fn spawn_open_helper(path: &Path) -> std::io::Result<()> {
    let executable = std::env::current_exe()?;
    std::process::Command::new(executable)
        .arg(OPEN_FOLDER_FLAG)
        .arg(path)
        .spawn()
        .map(|_| ())
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

    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoInitializeEx(reserved: *mut core::ffi::c_void, mode: u32) -> i32;
        fn CoUninitialize();
    }

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

    pub(super) fn open_folder(path: &std::path::Path) -> Result<(), String> {
        // The shell delegates the open verb through COM; without an
        // initialized apartment it reports success but opens nothing.
        // SAFETY: both calls are plain OS apartment calls with no pointers.
        let com = unsafe { CoInitializeEx(std::ptr::null_mut(), 0x2) } >= 0;
        let result = open_folder_inner(path);
        if com {
            // SAFETY: balances the successful CoInitializeEx above.
            unsafe { CoUninitialize() };
        }
        result
    }

    fn open_folder_inner(path: &std::path::Path) -> Result<(), String> {
        // `start` through cmd is the most dependable route on Windows: it keeps
        // working where the shell `open` verb is refused by policy or where
        // explorer is spawned from a process without desktop access.
        let target = path.display().to_string();
        if std::process::Command::new("cmd")
            .args(["/c", "start", "", target.as_str()])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

        if std::process::Command::new("explorer.exe")
            .arg(path)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }

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

        Err(format!(
            "ShellExecuteW failed for {} (code {})",
            path.display(),
            result as isize
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_flag_is_stable() {
        assert_eq!(OPEN_FOLDER_FLAG, "--open-folder");
    }
}
