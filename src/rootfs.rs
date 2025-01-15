use libc::{MS_BIND, MS_PRIVATE, MS_REC, MS_SLAVE};

use crate::mount::mount;
use crate::{config::Config, error::ContainerErr};
use std::{fs, path::Path};

/// Mounts the root filesystem for a container.
pub fn setup_rootfs<P: AsRef<Path>>(config: &Config, bundle_path: P) -> Result<(), ContainerErr> {
    let config_root = bundle_path.as_ref().join(&config.root.path);
    let meta =
        fs::metadata(&config_root).map_err(ContainerErr::IO)?;
    if !meta.is_dir() {
        return Err(ContainerErr::RootFs(format!(
            "rootfs at {} is not a directory.",
            config.root.path
        )));
    }

    // See 'changing the propagation type of an existing mount' here:
    // https://www.man7.org/linux/man-pages/man2/mount.2.html
    mount("", "/", c"", MS_SLAVE | MS_REC, None).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to change propagation type of rootfs: {:?}",
            e
        ))
    })?;

    mount("", "/", c"", MS_PRIVATE, None).map_err(|e| {
        ContainerErr::RootFs(format!(
            "failed to remount container rootfs as private: {:?}",
            e
        ))
    })?;

    mount(
        &config_root,
	"/",
        c"bind",
        MS_BIND | MS_REC,
        None,
    )
    .map_err(|e| ContainerErr::RootFs(format!("failed to mount rootfs: {:?}", e)))?;

    Ok(())
}
