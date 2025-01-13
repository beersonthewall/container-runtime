use crate::mount::mount;
use crate::{config::Config, error::ContainerErr};
use std::{fs, path::Path};

/// Mounts the root filesystem for a container.
pub fn setup_rootfs<P: AsRef<Path>>(config: &Config, bundle_path: P) -> Result<(), ContainerErr> {
    let meta =
        fs::metadata(bundle_path.as_ref().join(&config.root.path)).map_err(ContainerErr::IO)?;
    if !meta.is_dir() {
        return Err(ContainerErr::RootFs(format!(
            "rootfs at {} is not a directory.",
            config.root.path
        )));
    }

    // See 'changing the propagation type of an existing mount' here:
    // https://www.man7.org/linux/man-pages/man2/mount.2.html
    let mount_flags = libc::MS_SLAVE | libc::MS_REC;
    mount("", "/", c"", mount_flags, None).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to change propagation type of rootfs: {:?}",
            e
        ))
    })?;

    let mount_flags = libc::MS_PRIVATE;
    mount("", &config.root.path, c"", mount_flags, None).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to remount container rootfs as private: {:?}",
            e
        ))
    })?;

    let mount_flags = libc::MS_BIND | libc::MS_REC;
    mount(
        &config.root.path,
        &config.root.path,
        c"bind",
        mount_flags,
        None,
    )
    .map_err(|e| ContainerErr::RootFs(format!("failed to mount rootfs: {:?}", e)))?;

    Ok(())
}
