//! Free space on the volume that holds a path, on every platform.

use std::io;
use std::path::{Path, PathBuf};

/// Bytes this user may still write on the volume holding `path`. The path need
/// not exist yet — a folder the player is about to create is measured on the
/// volume of its nearest existing parent.
pub fn available(path: &Path) -> io::Result<u64> {
    imp(&nearest_existing(path)?)
}

/// `path` itself or the closest of its parents that exists.
pub fn nearest_existing(path: &Path) -> io::Result<PathBuf> {
    path.ancestors()
        .find(|p| !p.as_os_str().is_empty() && p.exists())
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no existing parent"))
}

#[cfg(unix)]
fn imp(path: &Path) -> io::Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c_path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "NUL in path"))?;
    // SAFETY: a valid C string in; statvfs fills the zeroed struct completely.
    unsafe {
        let mut stat: libc::statvfs = std::mem::zeroed();
        if libc::statvfs(c_path.as_ptr(), &mut stat) != 0 {
            return Err(io::Error::last_os_error());
        }
        // `f_bavail` is 32-bit on macOS and 64-bit on Linux, so the cast is
        // required on one and redundant on the other.
        #[allow(clippy::unnecessary_cast)]
        Ok((stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64))
    }
}

#[cfg(windows)]
fn imp(path: &Path) -> io::Result<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    // A UNC path is only accepted with the trailing separator.
    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide
        .last()
        .is_some_and(|c| *c != u16::from(b'\\') && *c != u16::from(b'/'))
    {
        wide.push(u16::from(b'\\'));
    }
    wide.push(0);
    let mut free_to_caller = 0u64;
    // SAFETY: a NUL-terminated wide string in; the two totals are optional
    // and passed as null, the one out-parameter points at a live u64.
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_to_caller,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if ok == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(free_to_caller)
    }
}

#[cfg(not(any(unix, windows)))]
fn imp(_path: &Path) -> io::Result<u64> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "free space"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_folder_still_to_be_created_is_measured_on_its_parent() {
        let tmp = crate::test_http::TempDir::new("disk-space");
        let missing = tmp.path().join("a").join("b");
        assert_eq!(nearest_existing(&missing).unwrap(), tmp.path());
        assert!(available(&missing).unwrap() > 0);
    }
}
