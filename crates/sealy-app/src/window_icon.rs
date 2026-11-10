//! Gives the Win32 window the launcher icon, for the taskbar and Alt-Tab.
//!
//! This module is the only place in the workspace that talks to Win32
//! directly, so the unsafe allowance is scoped to it.
#![allow(unsafe_code)]

/// The portable icon, embedded so runtime decoration never depends on files.
const ICON: &[u8] = include_bytes!("../../../packaging/windows/sealy.ico");

/// Applies the icon once the first window of this process exists.
pub fn apply() {
    std::thread::spawn(|| {
        for _ in 0..120 {
            if let Some(hwnd) = find_window(std::process::id()) {
                unsafe { set_icon(hwnd) };
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}

#[cfg(windows)]
fn find_window(pid: u32) -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::Foundation::{BOOL, LPARAM, TRUE};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, IsWindowVisible,
    };

    struct Search {
        pid: u32,
        found: Option<windows_sys::Win32::Foundation::HWND>,
    }

    unsafe extern "system" fn callback(
        hwnd: windows_sys::Win32::Foundation::HWND,
        lparam: LPARAM,
    ) -> BOOL {
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut owner = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut owner) };
        if owner == search.pid && unsafe { IsWindowVisible(hwnd) } != 0 {
            search.found = Some(hwnd);
            return 0;
        }
        TRUE
    }

    let mut search = Search { pid, found: None };
    unsafe {
        EnumWindows(Some(callback), &mut search as *mut Search as LPARAM);
    }
    search.found
}

#[cfg(windows)]
unsafe fn set_icon(hwnd: windows_sys::Win32::Foundation::HWND) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateIconFromResourceEx, ICON_BIG, ICON_SMALL, LR_DEFAULTCOLOR, SendMessageW, WM_SETICON,
    };

    let Some(image) = largest_icon(ICON) else {
        return;
    };

    let icon = unsafe {
        CreateIconFromResourceEx(
            image.as_ptr(),
            image.len() as u32,
            1,
            0x0003_0000,
            0,
            0,
            LR_DEFAULTCOLOR,
        )
    };
    if icon.is_null() {
        return;
    }

    unsafe {
        SendMessageW(hwnd, WM_SETICON, ICON_BIG as usize, icon as isize);
        SendMessageW(hwnd, WM_SETICON, ICON_SMALL as usize, icon as isize);
    }
}

/// Picks the biggest image inside an `.ico` container.
#[cfg(windows)]
fn largest_icon(data: &[u8]) -> Option<&[u8]> {
    if data.len() < 6 {
        return None;
    }
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    let mut best: Option<(u32, &[u8])> = None;

    for index in 0..count {
        let entry = 6 + index * 16;
        let bytes = data.get(entry..entry + 16)?;
        let width = if bytes[0] == 0 {
            256u32
        } else {
            bytes[0] as u32
        };
        let length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        let offset = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let end = offset.checked_add(length as usize)?;
        let image = data.get(offset..end)?;
        if best.is_none_or(|(best_width, _)| width > best_width) {
            best = Some((width, image));
        }
    }

    best.map(|(_, image)| image)
}
