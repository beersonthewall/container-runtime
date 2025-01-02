use crate::{config::Config, error::ContainerErr};

/// Mounts the root filesystem for a container.
pub fn setup_rootfs(config: &Config) -> Result<(), ContainerErr> {
    Ok(())
}
