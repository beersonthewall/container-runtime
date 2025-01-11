use crate::ctx::Ctx;
use crate::{config::Config, error::ContainerErr};
use libc::{__errno_location, c_ulong};
use std::ffi::{c_void, CStr};
use std::os::unix::ffi::OsStrExt;
use std::{ffi::CString, fs, path::Path};

/// Mounts the root filesystem for a container.
pub fn setup_rootfs<P: AsRef<Path>>(config: &Config, bundle_path: P) -> Result<(), ContainerErr> {
    let meta = fs::metadata(bundle_path.as_ref().join(&config.root.path)).map_err(ContainerErr::IO)?;
    if !meta.is_dir() {
        return Err(ContainerErr::RootFs(format!(
            "rootfs at {} is not a directory.",
            config.root.path
        )));
    }

    // See 'changing the propagation type of an existing mount' here:
    // https://www.man7.org/linux/man-pages/man2/mount.2.html
    let mount_flags = libc::MS_SLAVE | libc::MS_REC;
    mount("", "/", c"", mount_flags).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to change propagation type of rootfs: {:?}",
            e
        ))
    })?;

    let mount_flags = libc::MS_PRIVATE;
    mount("", &config.root.path, c"", mount_flags).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to remount container rootfs as private: {:?}",
            e
        ))
    })?;

    let mount_flags = libc::MS_BIND | libc::MS_REC;
    mount(&config.root.path, &config.root.path, c"bind", mount_flags)
        .map_err(|e| ContainerErr::RootFs(format!("failed to mount rootfs: {:?}", e)))?;

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
