use crate::{config::Config, error::ContainerErr};
use libc::{__errno_location, c_int, c_ulong, MS_ASYNC, MS_BIND, MS_DIRSYNC, MS_I_VERSION, MS_KERNMOUNT, MS_LAZYTIME, MS_MANDLOCK, MS_MOVE, MS_NOATIME, MS_NODEV, MS_NODIRATIME, MS_NOEXEC, MS_NOSUID, MS_NOUSER, MS_PRIVATE, MS_RDONLY, MS_REC, MS_RELATIME, MS_REMOUNT, MS_SHARED, MS_SILENT, MS_SLAVE, MS_STRICTATIME, MS_SYNC, MS_SYNCHRONOUS, MS_UNBINDABLE };

use std::ffi::{c_void, CStr};
use std::os::unix::ffi::OsStrExt;
use std::{ffi::CString, path::Path};

pub fn setup_mounts(config: &Config) -> Result<(), ContainerErr> {
    if let Some(mounts) = config.mounts() {
	for mnt in mounts {
	    let mut flags = 0;
	    if let Some(opts) = &mnt.options {
		flags |= parse_mount_options(&opts);
	    }
	    let src = if let Some(src) = &mnt.source {
		src
	    } else {
		""
	    };

	    mount(&src, &mnt.destination, c"", flags).map_err(ContainerErr::Mount)?;
	}
    }
    Ok(())
}

#[derive(Debug)]
pub enum MountErr {
    InvalidPath(String),
    Generic(String),
}

pub fn mount<S: AsRef<Path>, T: AsRef<Path>>(
    src: S,
    target: T,
    fstype: &CStr,
    flags: c_ulong,
) -> Result<(), MountErr> {
    let src = CString::new(src.as_ref().as_os_str().as_bytes())
        .map_err(|e| MountErr::InvalidPath(format!("{:?}", e)))?;
    let target = CString::new(target.as_ref().as_os_str().as_bytes())
        .map_err(|e| MountErr::InvalidPath(format!("{:?}", e)))?;
    let err = unsafe {
        libc::mount(
            src.as_ptr(),
            target.as_ptr(),
            fstype.as_ptr(),
            flags,
            std::ptr::null() as *const c_void,
        )
    };
    if err > 0 {
        return Err(MountErr::Generic(format!(
            "exit code: {}, errno {}",
            err,
            unsafe { *__errno_location() }
        )));
    }
    Ok(())
}

/// Turns mount options from the config into mount(2) flags
fn parse_mount_options(options: &[String]) -> c_ulong {
    let mut flags: c_ulong = 0;
    for opt in options {
        match opt.as_str() {
            "async" => flags |= MS_ASYNC as c_ulong,
            "atime" => flags ^= MS_NOATIME,
            "bind" => flags |= MS_BIND,
            "defaults" => flags |= 0,
            "dev" => flags |= MS_NODEV,
            "diratime" => flags ^= MS_NODIRATIME,
            "dirsync" => flags |= MS_DIRSYNC,
            "exec" => flags ^= MS_NOEXEC,
            "iversion" => flags |= MS_I_VERSION,
            "lazytime" => flags |= MS_LAZYTIME,
            "loud" => flags ^= MS_SILENT,
            "noatime" => flags |= MS_NOATIME,
            "nodev" => flags |= MS_NODEV,
            "nodiratime" => flags |= MS_NODIRATIME,
            "noexec" => flags |= MS_NOEXEC,
            "noiversion" => flags ^= MS_I_VERSION,
            "nolazytime" => flags ^= MS_LAZYTIME,
            "norelatime" => flags ^= MS_RELATIME,
            "nostrictatime" => flags ^= MS_STRICTATIME,
            "nosuid" => flags |= MS_NOSUID,
            "private" => flags |= MS_PRIVATE,
            "rbind" => flags |= MS_BIND | MS_REC,
            "relatime" => flags |= MS_RELATIME,
            "remount" => flags |= MS_REMOUNT,
            "ro" => flags |= MS_RDONLY,
            "rprivate" => flags |= MS_PRIVATE,
            "rshared" => flags |= MS_SHARED,
            "rslave" => flags |= MS_SLAVE,
            "runbindable" => flags |= MS_UNBINDABLE,
            "rw" => flags ^= MS_RDONLY,
            "shared" => flags |= MS_SHARED,
            "silent" => flags ^= MS_SILENT,
            "slave" => flags |= MS_SLAVE,
            "strictatime" => flags |= MS_STRICTATIME,
            "suid" => flags ^= MS_NOSUID,
            "sync" => flags |= MS_SYNCHRONOUS,
            "unbindable" => flags |= MS_UNBINDABLE,
            _ => {},
        }
    }
    flags
}

