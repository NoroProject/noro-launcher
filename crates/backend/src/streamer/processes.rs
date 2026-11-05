//! Is a streaming app running?
//!
//! Names only, read straight from the system: a full process snapshot (memory,
//! command lines, users) costs far more than this question needs.

/// Whether a process by this name is a streaming app. `obs` has to match the
/// whole name: as a prefix it would catch Obsidian.
pub fn is_streaming_app(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase();
    let stem = name.strip_suffix(".exe").unwrap_or(&name);
    matches!(stem, "obs" | "obs64" | "obs32" | "obs-studio")
        || stem.starts_with("streamlabs")
        || stem.starts_with("xsplit")
}

pub fn streaming_app_running() -> bool {
    process_names().iter().any(|name| is_streaming_app(name))
}

#[cfg(target_os = "linux")]
fn process_names() -> Vec<String> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .bytes()
                .all(|b| b.is_ascii_digit())
        })
        .filter_map(|e| std::fs::read_to_string(e.path().join("comm")).ok())
        .collect()
}

#[cfg(target_os = "macos")]
fn process_names() -> Vec<String> {
    use std::ffi::c_void;
    // SAFETY: a null buffer asks only for the count.
    let count = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
    if count <= 0 {
        return Vec::new();
    }
    // Room for processes started between the two calls.
    let mut pids = vec![0 as libc::c_int; count as usize + 64];
    let bytes = (pids.len() * std::mem::size_of::<libc::c_int>()) as libc::c_int;
    // SAFETY: the buffer is as long as the size passed with it.
    let filled = unsafe { libc::proc_listallpids(pids.as_mut_ptr() as *mut c_void, bytes) };
    if filled <= 0 {
        return Vec::new();
    }
    pids.truncate(filled as usize);
    let mut names = Vec::with_capacity(pids.len());
    let mut buf = [0u8; 256];
    for pid in pids {
        // SAFETY: the buffer is as long as the size passed with it.
        let len =
            unsafe { libc::proc_name(pid, buf.as_mut_ptr() as *mut c_void, buf.len() as u32) };
        if len > 0 {
            names.push(String::from_utf8_lossy(&buf[..len as usize]).into_owned());
        }
    }
    names
}

#[cfg(windows)]
fn process_names() -> Vec<String> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    // SAFETY: a process snapshot takes no pointers; the handle is closed below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Vec::new();
    }
    let mut names = Vec::new();
    // SAFETY: plain data; the size field is set as the API requires.
    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
    // SAFETY: `entry` lives across the walk and its size is set.
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) } != 0;
    while more {
        let len = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        names.push(String::from_utf16_lossy(&entry.szExeFile[..len]));
        // SAFETY: as above.
        more = unsafe { Process32NextW(snapshot, &mut entry) } != 0;
    }
    // SAFETY: the snapshot handle came from the call above and is closed once.
    unsafe { CloseHandle(snapshot) };
    names
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn process_names() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::is_streaming_app;

    #[test]
    fn streaming_apps_by_their_process_names() {
        for name in [
            "obs64.exe",
            "obs",
            "OBS",
            "obs32.exe",
            "Streamlabs OBS.exe",
            "Streamlabs Desktop",
            "XSplit.Core.exe",
            "XSplit Broadcaster",
            "obs\n",
        ] {
            assert!(is_streaming_app(name), "{name}");
        }
    }

    #[test]
    fn similar_names_are_not_streaming_apps() {
        for name in ["Obsidian", "obsidian.exe", "jobs", "observer", "java"] {
            assert!(!is_streaming_app(name), "{name}");
        }
    }

    /// Reading the list must work on the machine running the tests, whatever
    /// is on it — the answer itself depends on the machine.
    #[test]
    fn the_process_list_is_readable() {
        assert!(!super::process_names().is_empty());
    }
}
