//! Host information used by the launcher (physical memory today).
//!
//! The launcher must not pull a heavyweight system-information crate for one
//! number, so each platform uses the cheapest reliable source.

/// Total physical memory in MiB, when it can be detected.
pub fn total_memory_mb() -> Option<u64> {
    #[cfg(target_os = "windows")]
    {
        windows::total_memory_mb()
    }
    #[cfg(target_os = "macos")]
    {
        macos::total_memory_mb()
    }
    #[cfg(target_os = "linux")]
    {
        linux::total_memory_mb()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Heap cap the launcher picks on its own.
///
/// A quarter of the physical memory keeps room for the OS and the rest of the
/// desktop; the result is rounded to a 512 MiB step and clamped to
/// 1024..=8192 MiB so small machines stay usable and 32 GiB machines do not
/// hand the JVM an unsupported heap.
pub fn auto_memory_mb(total_mb: Option<u64>) -> u32 {
    let Some(total) = total_mb.filter(|value| *value > 0) else {
        return 4096;
    };
    let quarter = total / 4;
    let rounded = (quarter / 512).max(1) * 512;
    rounded.clamp(1024, 8192) as u32
}

/// Descriptive summary for the settings page, e.g. `16 GB`.
pub fn memory_label(total_mb: Option<u64>) -> Option<String> {
    let total = total_mb?;
    let gb = total as f64 / 1024.0;
    if gb >= 1.0 {
        Some(format!("{gb:.0} GB"))
    } else {
        Some(format!("{total} MB"))
    }
}

#[cfg(target_os = "windows")]
mod windows {
    #[allow(unsafe_code)]
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[allow(unsafe_code)]
    unsafe extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
    }

    pub(super) fn total_memory_mb() -> Option<u64> {
        let mut status = MemoryStatusEx {
            length: std::mem::size_of::<MemoryStatusEx>() as u32,
            memory_load: 0,
            total_phys: 0,
            avail_phys: 0,
            total_page_file: 0,
            avail_page_file: 0,
            total_virtual: 0,
            avail_virtual: 0,
            avail_extended_virtual: 0,
        };
        #[allow(unsafe_code)]
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
        (ok != 0).then_some(status.total_phys / (1024 * 1024))
    }
}

#[cfg(target_os = "macos")]
mod macos {
    pub(super) fn total_memory_mb() -> Option<u64> {
        let output = std::process::Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&output.stdout);
        let bytes = text.trim().parse::<u64>().ok()?;
        Some(bytes / (1024 * 1024))
    }
}

#[cfg(target_os = "linux")]
mod linux {
    pub(super) fn total_memory_mb() -> Option<u64> {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        let line = text.lines().find(|line| line.starts_with("MemTotal:"))?;
        let kb = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
        Some(kb / 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_memory_follows_a_quarter_of_ram() {
        assert_eq!(auto_memory_mb(Some(16384)), 4096);
        assert_eq!(auto_memory_mb(Some(32768)), 8192);
        assert_eq!(auto_memory_mb(Some(6144)), 1536);
    }

    #[test]
    fn auto_memory_clamps() {
        assert_eq!(auto_memory_mb(Some(2048)), 1024);
        assert_eq!(auto_memory_mb(Some(4096)), 1024);
        assert_eq!(auto_memory_mb(Some(65536)), 8192);
        assert_eq!(auto_memory_mb(None), 4096);
        assert_eq!(auto_memory_mb(Some(0)), 4096);
    }

    #[test]
    fn memory_label_uses_gigabytes() {
        assert_eq!(memory_label(Some(16384)).as_deref(), Some("16 GB"));
        assert_eq!(memory_label(None), None);
    }

    #[test]
    fn detects_host_memory() {
        // Every supported platform should report a positive amount.
        if let Some(total) = total_memory_mb() {
            assert!(total > 256, "detected {total} MiB");
        }
    }
}
