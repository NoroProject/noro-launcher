//! Facts about the machine the launcher runs on.

/// Physical memory in megabytes, or `None` where it can't be read.
pub fn total_memory_mb() -> Option<u32> {
    let bytes = physical_memory_bytes()?;
    u32::try_from(bytes / (1024 * 1024)).ok()
}

#[cfg(unix)]
fn physical_memory_bytes() -> Option<u64> {
    // SAFETY: sysconf only reads system configuration.
    let (pages, page_size) = unsafe {
        (
            libc::sysconf(libc::_SC_PHYS_PAGES),
            libc::sysconf(libc::_SC_PAGESIZE),
        )
    };
    if pages <= 0 || page_size <= 0 {
        return None;
    }
    Some(pages as u64 * page_size as u64)
}

#[cfg(windows)]
fn physical_memory_bytes() -> Option<u64> {
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    // SAFETY: the struct is plain data; its length field is set as the API
    // requires before the call.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return None;
    }
    Some(status.ullTotalPhys)
}

#[cfg(not(any(unix, windows)))]
fn physical_memory_bytes() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn this_machine_has_some_memory() {
        let mb = super::total_memory_mb().expect("readable here");
        assert!(mb > 256, "{mb} MB");
    }
}
