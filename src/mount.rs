use crate::{config::Config, error::ContainerErr};
use libc::{
    __errno_location, c_ulong, MS_ASYNC, MS_BIND, MS_DIRSYNC, MS_I_VERSION, MS_LAZYTIME,
    MS_NOATIME, MS_NODEV, MS_NODIRATIME, MS_NOEXEC, MS_NOSUID, MS_PRIVATE, MS_RDONLY, MS_REC,
    MS_RELATIME, MS_REMOUNT, MS_SHARED, MS_SILENT, MS_SLAVE, MS_STRICTATIME, MS_SYNCHRONOUS,
    MS_UNBINDABLE,
};
use std::ffi::{c_void, CStr};
use std::os::unix::ffi::OsStrExt;
use std::{ffi::CString, path::Path};

pub fn setup_mounts(config: &Config) -> Result<(), ContainerErr> {
    if let Some(mounts) = config.mounts() {
        for mnt in mounts {
            let mut flags = 0;
            let mut fs_opts = Vec::<String>::new();
            let src = if mnt.source.is_some() && mnt.typ.is_none() {
		mnt.source.as_ref().unwrap()
	    } else {
		""
	    };

            if let Some(opts) = &mnt.options {
                flags |= parse_mount_options(opts, &mut fs_opts);
            }

            let fs_opts = CString::new(fs_opts.join(",")).map_err(|e| {
                ContainerErr::Options(format!("could not convert options to cstring: {}", e))
            })?;

	    let t = if let Some(t) = mnt.typ.as_ref() {
		CString::new(t.as_bytes()).map_err(|e| ContainerErr::MountType(format!("mount type cstring conversion failed: {}", e)))?
	    } else {
		CString::new("".as_bytes()).unwrap()
	    };


            mount(
                src,
                &mnt.destination,
                t.as_c_str(),
                flags,
                Some(fs_opts.as_ptr() as *const c_void),
            )
            .map_err(ContainerErr::Mount)?;
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
    data: Option<*const c_void>,
) -> Result<(), MountErr> {
    let src = CString::new(src.as_ref().as_os_str().as_bytes())
        .map_err(|e| MountErr::InvalidPath(format!("{:?}", e)))?;
    let target = CString::new(target.as_ref().as_os_str().as_bytes())
        .map_err(|e| MountErr::InvalidPath(format!("{:?}", e)))?;

    let ptr = data.unwrap_or(std::ptr::null());

    let err = unsafe { libc::mount(src.as_ptr(), target.as_ptr(), fstype.as_ptr(), flags, ptr) };
    if err != 0 {
        return Err(MountErr::Generic(format!(
            "exit code: {}, errno {}",
            err,
            unsafe { *__errno_location() }
        )));
    }
    Ok(())
}

/// Converts mount options from the config into mount(2) flags &
/// filesystem specific options.
fn parse_mount_options(options: &[String], fs_opts: &mut Vec<String>) -> c_ulong {
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
            o => fs_opts.push(o.to_string()),
        }
    }

    flags
}

